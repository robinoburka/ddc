use std::collections::BTreeSet;

use humansize::{DECIMAL, format_size};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
};
use ratatui::{Frame, crossterm::event::KeyCode};

use crate::browse_tui::component::{Component, Navigable};
use crate::browse_tui::helpers::{last_update_cell, now, size_cell};
use crate::browse_tui::message::{AppMessage, SortBy, SortDirection};
use crate::vcs_postprocess::EnrichedVcsResult;

#[derive(Debug)]
pub struct VcsTab {
    results: Vec<EnrichedVcsResult>,
    preprocessed_filter_paths: Vec<String>,
    view: Vec<usize>,
    sum: u64,
    state: TableState,
    scroll_state: ScrollbarState,
    page_size: u16,
    sort_by: Option<SortBy>,
    sort_direction: SortDirection,
    active_filter: Option<String>,
}

impl VcsTab {
    const SORT_OPTIONS: [SortBy; 3] = [SortBy::Project, SortBy::Size, SortBy::LastUpdate];

    pub fn new(results: Vec<EnrichedVcsResult>) -> Self {
        let filter_paths = results
            .iter()
            .map(|r| r.path.to_string_lossy().to_ascii_lowercase())
            .collect();

        Self {
            state: {
                let mut state = TableState::default();
                state.select(Some(0));
                state
            },
            scroll_state: ScrollbarState::new(results.len()),
            sum: results.iter().map(|r| r.size).sum(),
            view: (0..results.len()).collect(),
            preprocessed_filter_paths: filter_paths,
            results,
            page_size: 0,
            sort_by: None,
            sort_direction: SortDirection::default(),
            active_filter: None,
        }
    }

    pub fn apply_filter(&mut self, filter: Option<String>) {
        let normalized = filter
            .map(|raw| raw.trim().to_ascii_lowercase())
            .filter(|value| !value.is_empty());

        if self.active_filter == normalized {
            return;
        }

        self.active_filter = normalized;
        self.refresh_view();
    }

    fn selected_result(&self) -> Option<&EnrichedVcsResult> {
        self.state
            .selected()
            .and_then(|visible_idx| self.view.get(visible_idx))
            .and_then(|idx| self.results.get(*idx))
    }

    fn enter(&mut self) -> Option<AppMessage> {
        self.selected_result()
            .map(|res| res.path.clone())
            .map(AppMessage::EnterBrowser)
    }

    fn request_sort(&mut self) -> Option<AppMessage> {
        Some(AppMessage::OpenSort(&Self::SORT_OPTIONS))
    }

    fn apply_sort(&mut self, sort_by: SortBy) -> Option<AppMessage> {
        if self.sort_by == Some(sort_by) {
            self.sort_direction = match self.sort_direction {
                SortDirection::Ascending => SortDirection::Descending,
                SortDirection::Descending => SortDirection::Ascending,
            };
        } else {
            self.sort_by = Some(sort_by);
            self.sort_direction = sort_by.default_direction();
        }

        self.refresh_view();

        None
    }

    fn start_filter(&mut self) -> Option<AppMessage> {
        Some(AppMessage::StartFilter)
    }

    fn sync_scroll(&mut self) {
        let selected = self.state.selected().unwrap_or(0);
        self.scroll_state = ScrollbarState::new(self.view.len()).position(selected);
    }

    fn refresh_view(&mut self) {
        self.view = match self.active_filter.as_ref() {
            Some(filter) => self
                .preprocessed_filter_paths
                .iter()
                .enumerate()
                .filter_map(|(idx, path)| path.contains(filter).then_some(idx))
                .collect(),
            None => (0..self.results.len()).collect(),
        };

        if let Some(sort_by) = self.sort_by {
            match sort_by {
                SortBy::Project => self.view.sort_by(|a_idx, b_idx| {
                    self.results[*a_idx].path.cmp(&self.results[*b_idx].path)
                }),
                SortBy::Size => self.view.sort_by_key(|idx| self.results[*idx].size),
                SortBy::LastUpdate => self
                    .view
                    .sort_by_key(|idx| newest_project_update(&self.results[*idx])),
            }

            if self.sort_direction == SortDirection::Descending {
                self.view.reverse();
            }
        }

        self.sum = self.view.iter().map(|&idx| self.results[idx].size).sum();
        self.adjust_selection_to_view();
        self.sync_scroll();
    }

    fn adjust_selection_to_view(&mut self) {
        if self.view.is_empty() {
            self.state.select(None);
            return;
        }

        let selected = self.state.selected().unwrap_or(0);
        let last = self.view.len().saturating_sub(1);
        self.state.select(Some(selected.min(last)));
    }
}

#[derive(Debug)]
pub enum VcsTabMessage {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    RequestSort,
    ApplySort(SortBy),
    StartFilter,
}

impl Component for VcsTab {
    type Message = VcsTabMessage;

    fn update(&mut self, message: Self::Message) -> Option<AppMessage> {
        match message {
            VcsTabMessage::MoveUp => self.move_up(),
            VcsTabMessage::MoveDown => self.move_down(),
            VcsTabMessage::PageUp => self.page_up(),
            VcsTabMessage::PageDown => self.page_down(),
            VcsTabMessage::Home => self.home(),
            VcsTabMessage::End => self.end(),
            VcsTabMessage::Enter => {
                return self.enter();
            }
            VcsTabMessage::RequestSort => {
                return self.request_sort();
            }
            VcsTabMessage::ApplySort(sort_by) => {
                return self.apply_sort(sort_by);
            }
            VcsTabMessage::StartFilter => {
                return self.start_filter();
            }
        }
        None
    }

    fn handle_key(&mut self, key: KeyCode) -> Option<Self::Message> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => Some(VcsTabMessage::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(VcsTabMessage::MoveDown),
            KeyCode::Right | KeyCode::Char('l') => Some(VcsTabMessage::Enter),
            KeyCode::PageDown => Some(VcsTabMessage::PageDown),
            KeyCode::PageUp => Some(VcsTabMessage::PageUp),
            KeyCode::Home => Some(VcsTabMessage::Home),
            KeyCode::End => Some(VcsTabMessage::End),
            KeyCode::Char('s') => Some(VcsTabMessage::RequestSort),
            KeyCode::Char('/') => Some(VcsTabMessage::StartFilter),
            _ => None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.page_size = area.height.saturating_sub(3);

        let rows: Vec<_> = self
            .view
            .iter()
            .filter_map(|idx| self.results.get(*idx))
            .map(create_row)
            .collect();
        let human_size = format_size(self.sum, DECIMAL);

        let table = Table::new(
            rows,
            &[
                Constraint::Length(3),
                Constraint::Percentage(60),
                Constraint::Length(10),
                Constraint::Length(20),
                Constraint::Length(12),
                Constraint::Percentage(10),
            ],
        )
        .header(
            Row::new(vec![
                "",
                "Path",
                "Size",
                "Last project update",
                "VCS dir size",
                "Detections",
            ])
            .style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .footer(Row::new(vec![
            Cell::from(""),
            Cell::from(""),
            Cell::from(human_size.as_str()).style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Version controlled ")
                .title_style(Style::default().fg(Color::LightYellow))
                .border_style(Style::default().fg(Color::LightYellow)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

        frame.render_stateful_widget(table, area, &mut self.state);

        let needs_scroll = self.view.len() > self.page_size as usize;
        if needs_scroll {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"));

            frame.render_stateful_widget(scrollbar, area, &mut self.scroll_state);
        }
    }
}

fn create_row<'a>(result: &'a EnrichedVcsResult) -> Row<'a> {
    Row::new(vec![
        Cell::from(""),
        Cell::from(Line::from(result.path.display().to_string())),
        size_cell(result.size),
        last_update_cell(now(), newest_project_update(result)),
        size_cell(result.vcs_size),
        Cell::from(unique_detections(result)),
    ])
}

fn newest_project_update(result: &EnrichedVcsResult) -> Option<std::time::SystemTime> {
    result
        .matched_projects
        .iter()
        .filter_map(|p| p.last_update)
        .max()
}

fn unique_detections(result: &EnrichedVcsResult) -> String {
    let mut icons = BTreeSet::new();

    for project in &result.matched_projects {
        icons.insert(format!("{}", project.lang));
    }

    icons.into_iter().collect::<Vec<_>>().join(" ")
}

impl Navigable for VcsTab {
    fn move_up(&mut self) {
        self.state.select_previous();
        self.sync_scroll();
    }

    fn move_down(&mut self) {
        self.state.select_next();
        self.sync_scroll();
    }

    fn page_up(&mut self) {
        self.state.scroll_up_by(self.page_size);
        self.sync_scroll();
    }

    fn page_down(&mut self) {
        self.state.scroll_down_by(self.page_size);
        self.sync_scroll();
    }

    fn home(&mut self) {
        self.state.select_first();
        self.sync_scroll();
    }

    fn end(&mut self) {
        self.state.select_last();
        self.sync_scroll();
    }
}
