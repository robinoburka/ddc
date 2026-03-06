use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders};
use ratatui::{Frame, crossterm::event::KeyCode, widgets::Paragraph};

use crate::browse_tui::component::Component;
use crate::browse_tui::message::AppMessage;

#[derive(Debug)]
pub struct Footer {
    error_message: Option<String>,
}

impl Footer {
    pub fn new() -> Self {
        Self {
            error_message: None,
        }
    }

    pub fn set_error(&mut self, error_message: Option<String>) {
        self.error_message = error_message;
    }
}

#[derive(Debug)]
pub enum FooterMessage {}

impl Component for Footer {
    type Message = FooterMessage;
    fn update(&mut self, _message: Self::Message) -> Option<AppMessage> {
        None
    }
    fn handle_key(&mut self, _key: KeyCode) -> Option<Self::Message> {
        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let footer = if let Some(message) = &self.error_message {
            let msg = format!(" {}", message);
            Paragraph::new(msg).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" ERROR ")
                    .style(
                        Style::default()
                            .fg(Color::LightRed)
                            .add_modifier(Modifier::BOLD),
                    ),
            )
        } else {
            let line = vec![Line::from(vec![
                Span::styled(
                    "↑/↓",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Navigate  "),
                Span::styled(
                    "←",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Go Back  "),
                Span::styled(
                    "→",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Enter Item  "),
                Span::styled(
                    "p",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Enter Parent  "),
                Span::styled(
                    "i",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Info  "),
                Span::styled(
                    "s",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Sort  "),
                Span::styled(
                    "d/t",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Select Tab  "),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Close  "),
                Span::styled(
                    "?",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Help  "),
                Span::styled(
                    "q",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Quit"),
            ])];

            Paragraph::new(line).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(Line::from(" Controls ").alignment(Alignment::Left))
                    .title_style(Style::default().fg(Color::Green))
                    .title_bottom(
                        Line::from(" More controls in the help ")
                            .alignment(Alignment::Right)
                            .style(
                                Style::default()
                                    .fg(Color::Gray)
                                    .add_modifier(Modifier::ITALIC),
                            ),
                    )
                    .border_style(Style::default().fg(Color::Green)),
            )
        };

        frame.render_widget(footer, area);
    }
}
