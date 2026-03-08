use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Clear, List, ListItem, ListState, Padding, Scrollbar, ScrollbarOrientation,
    ScrollbarState,
};
use ratatui::{Frame, crossterm::event::KeyCode};

use crate::browse_tui::component::{Component, Navigable};
use crate::browse_tui::helpers::popup_area_clamped;
use crate::browse_tui::message::{AppMessage, SortBy};

#[derive(Debug)]
pub struct SortModal {
    options: &'static [SortBy],
    state: ListState,
    scroll_state: ScrollbarState,
    page_size: u16,
}

impl SortModal {
    pub fn new(options: &'static [SortBy]) -> Self {
        Self {
            state: {
                let mut state = ListState::default();
                state.select(Some(0));
                state
            },
            scroll_state: ScrollbarState::new(options.len()),
            options,
            page_size: 0,
        }
    }

    fn set_sort_by(&mut self, sort_by: SortBy) -> Option<AppMessage> {
        if self.options.contains(&sort_by) {
            Some(AppMessage::RequestSort(sort_by))
        } else {
            Some(AppMessage::SetError(format!(
                "Sort option '{}' is not available in this tab",
                sort_by.label()
            )))
        }
    }

    fn select_sort_option(&mut self) -> Option<AppMessage> {
        self.state
            .selected()
            .and_then(|idx| self.options.get(idx).cloned())
            .map(AppMessage::RequestSort)
    }
}

#[derive(Debug)]
pub enum SortModalMessage {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Home,
    End,
    SortBy(SortBy),
    SelectOption,
}

impl Component for SortModal {
    type Message = SortModalMessage;

    fn update(&mut self, message: Self::Message) -> Option<AppMessage> {
        match message {
            SortModalMessage::MoveUp => self.move_up(),
            SortModalMessage::MoveDown => self.move_down(),
            SortModalMessage::PageUp => self.page_up(),
            SortModalMessage::PageDown => self.page_down(),
            SortModalMessage::Home => self.home(),
            SortModalMessage::End => self.end(),
            SortModalMessage::SortBy(sort_by) => {
                return self.set_sort_by(sort_by);
            }
            SortModalMessage::SelectOption => {
                return self.select_sort_option();
            }
        }
        None
    }

    fn handle_key(&mut self, key: KeyCode) -> Option<Self::Message> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => Some(SortModalMessage::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(SortModalMessage::MoveDown),
            KeyCode::PageDown => Some(SortModalMessage::PageDown),
            KeyCode::PageUp => Some(SortModalMessage::PageUp),
            KeyCode::Home => Some(SortModalMessage::Home),
            KeyCode::End => Some(SortModalMessage::End),
            KeyCode::Char('p') => Some(SortModalMessage::SortBy(SortBy::Project)),
            KeyCode::Char('s') => Some(SortModalMessage::SortBy(SortBy::Size)),
            KeyCode::Char('u') => Some(SortModalMessage::SortBy(SortBy::LastUpdate)),
            KeyCode::Char('d') => Some(SortModalMessage::SortBy(SortBy::DetectedProjects)),
            KeyCode::Enter => Some(SortModalMessage::SelectOption),
            _ => None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let area = popup_area_clamped(area, 50, 80, 60, 15, 30, 40);
        self.page_size = area.height.saturating_sub(4);

        let items: Vec<ListItem> = self
            .options
            .iter()
            .map(|opt| {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{}", opt.key()),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" - "),
                    Span::raw(opt.label()),
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .padding(Padding::symmetric(2, 1))
                    .title(Line::from(" Sort By ").alignment(Alignment::Left))
                    .title_style(Style::default().fg(Color::Green))
                    .title(
                        Line::from(" Esc ").alignment(Alignment::Right).style(
                            Style::default()
                                .fg(Color::Red)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    )
                    .border_style(Style::default().fg(Color::Green)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        frame.render_widget(Clear, area);
        frame.render_stateful_widget(list, area, &mut self.state);

        let needs_scroll = self.options.len() > area.height.saturating_sub(4) as usize;
        if needs_scroll {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"));

            frame.render_stateful_widget(scrollbar, area, &mut self.scroll_state);
        }
    }
}

impl Navigable for SortModal {
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

impl SortModal {
    fn sync_scroll(&mut self) {
        self.scroll_state = self
            .scroll_state
            .position(self.state.selected().unwrap_or(0));
    }
}
