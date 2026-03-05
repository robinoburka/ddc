use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Padding, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{Frame, crossterm::event::KeyCode, widgets::Paragraph};

use crate::browse_tui::component::{Component, Navigable};
use crate::browse_tui::helpers;
use crate::browse_tui::message::AppMessage;

#[derive(Debug)]
pub struct InfoModal {
    text: &'static str,
    // Cached data
    wrapped_text: Option<Vec<String>>,
    // Caching key - this is NOT the inned width
    last_width: u16,
    // Scrolling state
    scroll: u16,
    window_height: u16,
    content_height: u16,
    scroll_state: Option<ScrollbarState>,
}

impl InfoModal {
    pub fn new(text: &'static str) -> Self {
        Self {
            text,
            wrapped_text: None,
            last_width: 0,
            scroll: 0,
            window_height: 0,
            content_height: 0,
            scroll_state: None,
        }
    }

    fn width(&self, area: Rect) -> u16 {
        area.width.saturating_sub(6)
    }

    fn height(&self, area: Rect) -> u16 {
        area.height.saturating_sub(4)
    }
}

#[derive(Debug)]
pub enum InfoModalMessage {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Home,
    End,
}

impl Component for InfoModal {
    type Message = InfoModalMessage;

    fn update(&mut self, message: Self::Message) -> Option<AppMessage> {
        match message {
            InfoModalMessage::MoveUp => self.move_up(),
            InfoModalMessage::MoveDown => self.move_down(),
            InfoModalMessage::PageUp => self.page_up(),
            InfoModalMessage::PageDown => self.page_down(),
            InfoModalMessage::Home => self.home(),
            InfoModalMessage::End => self.end(),
        }
        None
    }

    fn handle_key(&mut self, key: KeyCode) -> Option<Self::Message> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => Some(InfoModalMessage::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(InfoModalMessage::MoveDown),
            KeyCode::PageDown => Some(InfoModalMessage::PageDown),
            KeyCode::PageUp => Some(InfoModalMessage::PageUp),
            KeyCode::Home => Some(InfoModalMessage::Home),
            KeyCode::End => Some(InfoModalMessage::End),
            _ => None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let area = helpers::popup_area_clamped(area, 70, 150, 80, 22, 40, 60);

        if self.wrapped_text.is_none() || self.last_width != area.width {
            let message_lines = textwrap::wrap(self.text, self.width(area) as usize)
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>();
            let items = message_lines.len();

            self.wrapped_text = Some(message_lines);
            self.last_width = area.width; // Store caching key immediately
            self.content_height = items as u16;
        }

        if self.scroll_state.is_none() || self.window_height != self.height(area) {
            self.window_height = self.height(area);

            let needs_scroll = self.content_height > self.window_height;
            self.scroll_state = if needs_scroll {
                Some(ScrollbarState::new(
                    self.content_height.saturating_sub(self.window_height) as usize,
                ))
            } else {
                None
            }
        }

        let lines = self.wrapped_text.as_ref().unwrap();
        let lines = lines
            .iter()
            .map(|l| Line::from(l.as_ref()))
            .collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines).scroll((self.scroll, 0)).block(
            Block::bordered()
                .padding(Padding::symmetric(2, 1))
                .title_style(Style::default().fg(Color::LightBlue))
                .title(Line::from(" Info ").alignment(Alignment::Left))
                .title(
                    Line::from(" Esc ").alignment(Alignment::Right).style(
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::ITALIC),
                    ),
                )
                .border_style(Style::default().fg(Color::LightBlue)),
        );
        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);

        if let Some(ref mut scroll_state) = self.scroll_state {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"));
            frame.render_stateful_widget(scrollbar, area, scroll_state);
        }
    }
}

impl Navigable for InfoModal {
    fn move_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
        if let Some(ref mut s) = self.scroll_state {
            s.prev();
        }
    }

    fn move_down(&mut self) {
        let max_scroll = self.content_height.saturating_sub(self.window_height);
        self.scroll = self.scroll.saturating_add(1).min(max_scroll);
        if let Some(ref mut s) = self.scroll_state {
            s.next();
        }
    }

    fn page_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(self.window_height);
        self.scroll_state = self
            .scroll_state
            .map(|s| s.position(s.get_position().saturating_sub(self.window_height as usize)));
    }

    fn page_down(&mut self) {
        let max_scroll = self.content_height.saturating_sub(self.window_height);
        self.scroll = self
            .scroll
            .saturating_add(self.window_height)
            .min(max_scroll);
        self.scroll_state = self
            .scroll_state
            .map(|s| s.position(s.get_position().saturating_add(self.window_height as usize)));
    }

    fn home(&mut self) {
        self.scroll = 0;
        if let Some(ref mut s) = self.scroll_state {
            s.first();
        }
    }

    fn end(&mut self) {
        self.scroll = self.content_height.saturating_sub(self.window_height);
        if let Some(ref mut s) = self.scroll_state {
            s.last();
        }
    }
}
