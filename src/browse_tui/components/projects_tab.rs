use humansize::{DECIMAL, format_size};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
};
use ratatui::{Frame, crossterm::event::KeyCode};

use crate::browse_tui::component::{Component, Navigable};
use crate::browse_tui::helpers::{last_update_cell, now, parent_size_cell, size_cell};
use crate::browse_tui::message::{AppMessage, SortBy, SortDirection};
use crate::discovery::ProjectResult;

#[derive(Debug)]
pub struct ProjectsTab {
    results: Vec<ProjectResult>,
    sum: u64,
    state: TableState,
    scroll_state: ScrollbarState,
    page_size: u16,
    sort_by: Option<SortBy>,
    sort_direction: SortDirection,
}

impl ProjectsTab {
    const SORT_OPTIONS: [SortBy; 3] = [SortBy::Project, SortBy::Size, SortBy::LastUpdate];

    pub fn new(results: Vec<ProjectResult>) -> Self {
        Self {
            state: {
                let mut projects_state = TableState::default();
                projects_state.select(Some(0));
                projects_state
            },
            scroll_state: ScrollbarState::new(results.len()),
            sum: results.iter().map(|r| r.size).sum(),
            results,
            page_size: 0,
            sort_by: None,
            sort_direction: SortDirection::default(),
        }
    }

    fn enter(&mut self) -> Option<AppMessage> {
        self.state
            .selected()
            .and_then(|idx| self.results.get(idx))
            .map(|res| res.path.clone())
            .map(AppMessage::EnterBrowser)
    }

    fn enter_parent(&mut self) -> Option<AppMessage> {
        self.state
            .selected()
            .and_then(|idx| self.results.get(idx))
            .and_then(|res| res.parent.as_ref())
            .map(|parent| parent.path.clone())
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

        if let Some(sort_by) = self.sort_by.as_ref() {
            match sort_by {
                SortBy::Project => self.results.sort_by(|a, b| a.path.cmp(&b.path)),
                SortBy::Size => self.results.sort_by_key(|r| r.size),
                SortBy::LastUpdate => self.results.sort_by_key(|r| r.last_update),
            }

            if self.sort_direction == SortDirection::Descending {
                self.results.reverse();
            }
        }

        None
    }
}

#[derive(Debug)]
pub enum ProjectsTabMessage {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    EnterParent,
    RequestSort,
    ApplySort(SortBy),
}

impl Component for ProjectsTab {
    type Message = ProjectsTabMessage;

    fn update(&mut self, message: Self::Message) -> Option<AppMessage> {
        match message {
            ProjectsTabMessage::MoveUp => self.move_up(),
            ProjectsTabMessage::MoveDown => self.move_down(),
            ProjectsTabMessage::PageUp => self.page_up(),
            ProjectsTabMessage::PageDown => self.page_down(),
            ProjectsTabMessage::Home => self.home(),
            ProjectsTabMessage::End => self.end(),
            ProjectsTabMessage::Enter => {
                return self.enter();
            }
            ProjectsTabMessage::EnterParent => {
                return self.enter_parent();
            }
            ProjectsTabMessage::RequestSort => {
                return self.request_sort();
            }
            ProjectsTabMessage::ApplySort(sort_by) => {
                return self.apply_sort(sort_by);
            }
        }
        None
    }

    fn handle_key(&mut self, key: KeyCode) -> Option<Self::Message> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => Some(ProjectsTabMessage::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(ProjectsTabMessage::MoveDown),
            KeyCode::Right | KeyCode::Char('l') => Some(ProjectsTabMessage::Enter),
            KeyCode::Char('p') => Some(ProjectsTabMessage::EnterParent),
            KeyCode::PageDown => Some(ProjectsTabMessage::PageDown),
            KeyCode::PageUp => Some(ProjectsTabMessage::PageUp),
            KeyCode::Home => Some(ProjectsTabMessage::Home),
            KeyCode::End => Some(ProjectsTabMessage::End),
            KeyCode::Char('s') => Some(ProjectsTabMessage::RequestSort),
            _ => None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.page_size = area.height.saturating_sub(3);

        let rows: Vec<_> = self.results.iter().map(create_row).collect();
        let human_size = format_size(self.sum, DECIMAL);

        let table = Table::new(
            rows,
            &[
                Constraint::Length(3),
                Constraint::Percentage(60),
                Constraint::Length(10),
                Constraint::Length(20),
                Constraint::Length(11),
            ],
        )
        .header(
            Row::new(vec![
                "",
                "Project",
                "Size",
                "Last project update",
                "Parent size",
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
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Projects ")
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

        let needs_scroll = self.results.len() > area.height.saturating_sub(3) as usize;
        if needs_scroll {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"));

            frame.render_stateful_widget(scrollbar, area, &mut self.scroll_state);
        }
    }
}

fn create_row<'a>(result: &'a ProjectResult) -> Row<'a> {
    Row::new(vec![
        Cell::from(format!("{} ", result.lang)),
        Cell::from(Line::from(result.path.display().to_string())),
        size_cell(result.size),
        last_update_cell(now(), result.last_update),
        parent_size_cell(result.parent.as_ref().map(|p| p.size)),
    ])
}

impl Navigable for ProjectsTab {
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

impl ProjectsTab {
    fn sync_scroll(&mut self) {
        self.scroll_state = self
            .scroll_state
            .position(self.state.selected().unwrap_or(0));
    }
}
