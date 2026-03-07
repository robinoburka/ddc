use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};
use ratatui::{Frame, crossterm::event::KeyCode};

use crate::browse_tui::component::Component;
use crate::browse_tui::message::AppMessage;

#[derive(Debug)]
pub struct FilterBar {
    input: String,
    cursor_position: usize,
    is_active: bool,
    has_focus: bool,
}

impl FilterBar {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            is_active: false,
            has_focus: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn clear(&mut self) {
        self.is_active = false;
        self.has_focus = false;
        self.input.clear();
        self.cursor_position = 0;
    }

    pub fn get_filter(&self) -> Option<String> {
        if self.is_active {
            Some(self.input.clone())
        } else {
            None
        }
    }

    fn activate(&mut self) {
        self.is_active = true;
        self.has_focus = true;
    }

    fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    fn delete_char(&mut self) {
        if self.cursor_position < self.input.len() {
            self.input.remove(self.cursor_position);
        }
    }

    fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    fn move_cursor_home(&mut self) {
        self.cursor_position = 0;
    }

    fn move_cursor_end(&mut self) {
        self.cursor_position = self.input.len();
    }

    fn deactivate(&mut self) -> Option<AppMessage> {
        self.clear();
        self.has_focus = false;
        Some(AppMessage::DismissFilter)
    }

    fn accept(&mut self) -> Option<AppMessage> {
        self.has_focus = false;
        Some(AppMessage::AcceptFilter)
    }
}

#[derive(Debug)]
pub enum FilterBarMessage {
    Activate,
    InsertChar(char),
    Delete,
    Backspace,
    MoveCursorLeft,
    MoveCursorRight,
    MoveCursorHome,
    MoveCursorEnd,
    Deactivate,
    Accept,
}

impl Component for FilterBar {
    type Message = FilterBarMessage;

    fn update(&mut self, message: Self::Message) -> Option<AppMessage> {
        match message {
            FilterBarMessage::Activate => self.activate(),
            FilterBarMessage::InsertChar(c) => self.insert_char(c),
            FilterBarMessage::Delete => self.delete_char(),
            FilterBarMessage::Backspace => self.backspace(),
            FilterBarMessage::MoveCursorLeft => self.move_cursor_left(),
            FilterBarMessage::MoveCursorRight => self.move_cursor_right(),
            FilterBarMessage::MoveCursorHome => self.move_cursor_home(),
            FilterBarMessage::MoveCursorEnd => self.move_cursor_end(),
            FilterBarMessage::Deactivate => {
                return self.deactivate();
            }
            FilterBarMessage::Accept => {
                return self.accept();
            }
        }
        None
    }

    fn handle_key(&mut self, key: KeyCode) -> Option<Self::Message> {
        match key {
            KeyCode::Char(c) => Some(FilterBarMessage::InsertChar(c)),
            KeyCode::Backspace => Some(FilterBarMessage::Backspace),
            KeyCode::Delete => Some(FilterBarMessage::Delete),
            KeyCode::Left => Some(FilterBarMessage::MoveCursorLeft),
            KeyCode::Right => Some(FilterBarMessage::MoveCursorRight),
            KeyCode::Home => Some(FilterBarMessage::MoveCursorHome),
            KeyCode::End => Some(FilterBarMessage::MoveCursorEnd),
            KeyCode::Esc => Some(FilterBarMessage::Deactivate),
            KeyCode::Enter => Some(FilterBarMessage::Accept),
            _ => None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let line = if self.has_focus {
            let before_cursor = &self.input[..self.cursor_position];
            let cursor_char = if self.cursor_position < self.input.len() {
                self.input.chars().nth(self.cursor_position).unwrap()
            } else {
                ' '
            };
            let after_cursor = if self.cursor_position < self.input.len() {
                &self.input[self.cursor_position + 1..]
            } else {
                ""
            };

            Line::from(vec![
                Span::raw(before_cursor),
                Span::styled(
                    cursor_char.to_string(),
                    Style::default()
                        .bg(Color::White)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(after_cursor),
            ])
        } else {
            Line::from(self.input.clone())
        };
        let focus_color = if self.has_focus {
            Color::LightYellow
        } else {
            Color::Cyan
        };

        let paragraph = Paragraph::new(line).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Filter ")
                .title_style(Style::default().fg(focus_color))
                .title_bottom(
                    Line::from(" Enter to apply, Esc to clear, / to get focus again ")
                        .alignment(Alignment::Right)
                        .style(
                            Style::default()
                                .fg(Color::Gray)
                                .add_modifier(Modifier::ITALIC),
                        ),
                )
                .border_style(Style::default().fg(focus_color))
                .padding(Padding::symmetric(1, 0)),
        );

        frame.render_widget(paragraph, area);
    }
}
