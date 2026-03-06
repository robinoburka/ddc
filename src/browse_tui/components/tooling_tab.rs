use std::path::PathBuf;

use humansize::{DECIMAL, format_size};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
};
use ratatui::{Frame, crossterm::event::KeyCode};

use crate::browse_tui::component::{Component, Navigable};
use crate::browse_tui::helpers::{last_update_cell, now, size_cell};
use crate::browse_tui::message::{AppMessage, SortBy, SortDirection};
use crate::discovery::ToolingResult;

#[derive(Debug)]
pub struct ToolingTab {
    results: Vec<ToolingResult>,
    sum: u64,
    state: TableState,
    scroll_state: ScrollbarState,
    page_size: u16,
    sort_by: Option<SortBy>,
    sort_direction: SortDirection,
}

impl ToolingTab {
    const SORT_OPTIONS: [SortBy; 2] = [SortBy::Size, SortBy::LastUpdate];

    pub fn new(results: Vec<ToolingResult>) -> Self {
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
            .and_then(|res| res.path.parent())
            .map(PathBuf::from)
            .map(AppMessage::EnterBrowser)
    }

    fn info(&mut self) -> Option<AppMessage> {
        if let Some(msg) = self
            .state
            .selected()
            .and_then(|idx| self.results.get(idx))
            .and_then(|res| res.info)
        {
            Some(AppMessage::OpenInfo(msg))
        } else {
            Some(AppMessage::SetError(String::from(
                "There is no info for this item.",
            )))
        }
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
                SortBy::Size => self.results.sort_by_key(|r| r.size),
                SortBy::LastUpdate => self.results.sort_by_key(|r| r.last_update),
                _ => {}
            }

            if self.sort_direction == SortDirection::Descending {
                self.results.reverse();
            }
        }

        None
    }
}

#[derive(Debug)]
pub enum ToolingTabMessage {
    Info,
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

impl Component for ToolingTab {
    type Message = ToolingTabMessage;

    fn update(&mut self, message: Self::Message) -> Option<AppMessage> {
        match message {
            ToolingTabMessage::MoveUp => self.move_up(),
            ToolingTabMessage::MoveDown => self.move_down(),
            ToolingTabMessage::PageUp => self.page_up(),
            ToolingTabMessage::PageDown => self.page_down(),
            ToolingTabMessage::Home => self.home(),
            ToolingTabMessage::End => self.end(),
            ToolingTabMessage::Enter => {
                return self.enter();
            }
            ToolingTabMessage::EnterParent => {
                return self.enter_parent();
            }
            ToolingTabMessage::Info => {
                return self.info();
            }
            ToolingTabMessage::RequestSort => {
                return self.request_sort();
            }
            ToolingTabMessage::ApplySort(sort_by) => {
                return self.apply_sort(sort_by);
            }
        }
        None
    }

    fn handle_key(&mut self, key: KeyCode) -> Option<Self::Message> {
        match key {
            KeyCode::Char('i') => Some(ToolingTabMessage::Info),
            KeyCode::Up | KeyCode::Char('k') => Some(ToolingTabMessage::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(ToolingTabMessage::MoveDown),
            KeyCode::Right | KeyCode::Char('l') => Some(ToolingTabMessage::Enter),
            KeyCode::Char('p') => Some(ToolingTabMessage::EnterParent),
            KeyCode::PageDown => Some(ToolingTabMessage::PageDown),
            KeyCode::PageUp => Some(ToolingTabMessage::PageUp),
            KeyCode::Home => Some(ToolingTabMessage::Home),
            KeyCode::End => Some(ToolingTabMessage::End),
            KeyCode::Char('s') => Some(ToolingTabMessage::RequestSort),
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
                Constraint::Length(4),
            ],
        )
        .header(
            Row::new(vec!["", "Tool", "Size", "Last project update", "Info"]).style(
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
                .title(" Tools ")
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

fn create_row<'a>(result: &'a ToolingResult) -> Row<'a> {
    Row::new(vec![
        Cell::from(format!("{} ", result.lang)),
        Cell::from(Line::from(vec![
            Span::raw(result.description),
            Span::styled(
                format!(" ({})", result.path.display()),
                Style::default().add_modifier(Modifier::DIM),
            ),
        ])),
        size_cell(result.size),
        last_update_cell(now(), result.last_update),
        Cell::from(Span::raw(result.info.map(|_| "📖 »").unwrap_or_default())),
    ])
}

impl Navigable for ToolingTab {
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

impl ToolingTab {
    fn sync_scroll(&mut self) {
        self.scroll_state = self
            .scroll_state
            .position(self.state.selected().unwrap_or(0));
    }
}
