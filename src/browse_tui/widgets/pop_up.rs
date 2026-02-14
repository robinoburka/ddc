use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::prelude::{Color, Line, Modifier, Style};
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;
use ratatui::widgets::{
    Block, Clear, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

use crate::browse_tui::helpers::popup_area_clamped;

pub struct PopUp {
    title: &'static str,
    text: &'static str,
}

impl PopUp {
    pub fn new(title: &'static str, text: &'static str) -> Self {
        Self { title, text }
    }

    fn width(&self, area: Rect) -> u16 {
        area.width.saturating_sub(6)
    }

    fn height(&self, area: Rect) -> u16 {
        area.height.saturating_sub(4)
    }
}

impl StatefulWidget for PopUp {
    type State = PopUpState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut PopUpState) {
        let area = popup_area_clamped(area, 70, 150, 80, 22, 40, 60);

        if state.wrapped_text.is_none() || state.last_width != area.width {
            let message_lines = textwrap::wrap(self.text, self.width(area) as usize)
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>();
            let items = message_lines.len();

            state.wrapped_text = Some(message_lines);
            state.last_width = area.width; // Store caching key immediately
            state.content_height = items as u16;
        }

        if state.scroll_state.is_none() || state.window_height != self.height(area) {
            state.window_height = self.height(area);

            let needs_scroll = state.content_height > state.window_height;
            state.scroll_state = if needs_scroll {
                Some(ScrollbarState::new(
                    state.content_height.saturating_sub(state.window_height) as usize,
                ))
            } else {
                None
            }
        }

        let lines = state.wrapped_text.as_ref().unwrap();
        let lines = lines
            .iter()
            .map(|l| Line::from(l.as_ref()))
            .collect::<Vec<_>>();
        let paragraph = Paragraph::new(lines).scroll((state.scroll, 0)).block(
            Block::bordered()
                .padding(Padding::symmetric(2, 1))
                .title_style(Style::default().fg(Color::LightBlue))
                .title(Line::from(self.title).alignment(Alignment::Left))
                .title(
                    Line::from(" Esc ").alignment(Alignment::Right).style(
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::ITALIC),
                    ),
                )
                .border_style(Style::default().fg(Color::LightBlue)),
        );

        Clear.render(area, buf);
        paragraph.render(area, buf);

        if let Some(ref mut scroll_state) = state.scroll_state {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"));

            scrollbar.render(area, buf, scroll_state);
        }
    }
}

#[derive(Debug, Default)]
pub struct PopUpState {
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

impl PopUpState {
    pub fn up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
        if let Some(ref mut s) = self.scroll_state {
            s.prev();
        }
    }

    pub fn down(&mut self) {
        let max_scroll = self.content_height.saturating_sub(self.window_height);
        self.scroll = self.scroll.saturating_add(1).min(max_scroll);
        if let Some(ref mut s) = self.scroll_state {
            s.next();
        }
    }

    pub fn page_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(self.window_height);
        self.scroll_state = self
            .scroll_state
            .map(|s| s.position(s.get_position().saturating_sub(self.window_height as usize)));
    }

    pub fn page_down(&mut self) {
        let max_scroll = self.content_height.saturating_sub(self.window_height);
        self.scroll = self
            .scroll
            .saturating_add(self.window_height)
            .min(max_scroll);
        self.scroll_state = self
            .scroll_state
            .map(|s| s.position(s.get_position().saturating_add(self.window_height as usize)));
    }

    pub fn home(&mut self) {
        self.scroll = 0;
        if let Some(ref mut s) = self.scroll_state {
            s.first();
        }
    }

    pub fn end(&mut self) {
        self.scroll = self.content_height.saturating_sub(self.window_height);
        if let Some(ref mut s) = self.scroll_state {
            s.last();
        }
    }
}
