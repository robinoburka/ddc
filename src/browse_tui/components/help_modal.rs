use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Padding, Wrap};
use ratatui::{Frame, crossterm::event::KeyCode, widgets::Paragraph};

use crate::browse_tui::component::Component;
use crate::browse_tui::helpers;
use crate::browse_tui::message::AppMessage;

#[derive(Debug)]
pub struct HelpModal;

impl HelpModal {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub enum HelpModalMessage {}

impl Component for HelpModal {
    type Message = HelpModalMessage;
    fn update(&mut self, _message: Self::Message) -> Option<AppMessage> {
        None
    }
    fn handle_key(&mut self, _key: KeyCode) -> Option<Self::Message> {
        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let area = helpers::popup_area_clamped(area, 70, 150, 80, 22, 40, 60);
        let help = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw(", "),
                Span::styled("1", Style::default().fg(Color::Yellow)),
                Span::raw("      "),
                Span::raw("Switch to the "),
                Span::styled(
                    "Detected Projects",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" tab"),
            ]),
            Line::from(vec![
                Span::styled("t", Style::default().fg(Color::Yellow)),
                Span::raw(", "),
                Span::styled("2", Style::default().fg(Color::Yellow)),
                Span::raw("      "),
                Span::raw("Switch to the "),
                Span::styled(
                    "Tooling Overview",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" tab"),
            ]),
            Line::from(vec![
                Span::styled("v", Style::default().fg(Color::Yellow)),
                Span::raw(", "),
                Span::styled("3", Style::default().fg(Color::Yellow)),
                Span::raw("      "),
                Span::raw("Switch to the "),
                Span::styled(
                    "Version Controlled",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" tab"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("↑", Style::default().fg(Color::Yellow)),
                Span::raw(", "),
                Span::styled("k", Style::default().fg(Color::Yellow)),
                Span::raw("      "),
                Span::raw("Go up"),
            ]),
            Line::from(vec![
                Span::styled("↓", Style::default().fg(Color::Yellow)),
                Span::raw(", "),
                Span::styled("j", Style::default().fg(Color::Yellow)),
                Span::raw("      "),
                Span::raw("Go Down"),
            ]),
            Line::from(vec![
                Span::styled("PageUp", Style::default().fg(Color::Yellow)),
                Span::raw("    "),
                Span::raw("Go one page up"),
            ]),
            Line::from(vec![
                Span::styled("PageDown", Style::default().fg(Color::Yellow)),
                Span::raw("  "),
                Span::raw("Go one page down"),
            ]),
            Line::from(vec![
                Span::styled("Home", Style::default().fg(Color::Yellow)),
                Span::raw("      "),
                Span::raw("Go to the first item"),
            ]),
            Line::from(vec![
                Span::styled("End", Style::default().fg(Color::Yellow)),
                Span::raw("       "),
                Span::raw("Go to the last item"),
            ]),
            Line::from(vec![
                Span::styled("←", Style::default().fg(Color::Yellow)),
                Span::raw(", "),
                Span::styled("h", Style::default().fg(Color::Yellow)),
                Span::raw("      "),
                Span::raw("Go Back"),
            ]),
            Line::from(vec![
                Span::styled("→", Style::default().fg(Color::Yellow)),
                Span::raw(", "),
                Span::styled("l", Style::default().fg(Color::Yellow)),
                Span::raw("      "),
                Span::raw("Enter the selected item"),
            ]),
            Line::from(vec![
                Span::styled("p", Style::default().fg(Color::Yellow)),
                Span::raw("         "),
                Span::raw("Enter the parent of the selected project or tool"),
            ]),
            Line::from(vec![
                Span::raw("          "),
                Span::raw("Not available in Version controlled tab."),
            ])
            .style(Style::default().add_modifier(Modifier::DIM)),
            Line::from(vec![
                Span::raw("          "),
                Span::raw("Use \u{2192} to open the selected VCS root directly."),
            ])
            .style(Style::default().add_modifier(Modifier::DIM)),
            Line::from(""),
            Line::from(vec![
                Span::styled("i", Style::default().fg(Color::Yellow)),
                Span::raw("         "),
                Span::raw("Show info window for the selected tool"),
            ]),
            Line::from(vec![
                Span::styled("s", Style::default().fg(Color::Yellow)),
                Span::raw("         "),
                Span::raw("Show dialog with sorting options"),
            ]),
            Line::from(vec![
                Span::raw("          "),
                Span::raw("Once the dialogue is opened, navigate to"),
            ])
            .style(Style::default().add_modifier(Modifier::DIM)),
            Line::from(vec![
                Span::raw("          "),
                Span::raw("the requested item and select with enter."),
            ])
            .style(Style::default().add_modifier(Modifier::DIM)),
            Line::from(vec![
                Span::raw("          "),
                Span::raw("Alternatively, use a letter next to the option."),
            ])
            .style(Style::default().add_modifier(Modifier::DIM)),
            Line::from(vec![
                Span::styled("/", Style::default().fg(Color::Yellow)),
                Span::raw("         "),
                Span::raw("Start filtering projects by path"),
            ]),
            Line::from(vec![
                Span::raw("          "),
                Span::raw("Filter is accepted with Enter, cleared with Esc,"),
            ])
            .style(Style::default().add_modifier(Modifier::DIM)),
            Line::from(vec![
                Span::raw("          "),
                Span::raw("and you can use / to gain focus to the filter again."),
            ])
            .style(Style::default().add_modifier(Modifier::DIM)),
            Line::from(""),
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw("       "),
                Span::raw("Close any pop-up window, e.g. Help"),
            ]),
            Line::from(vec![
                Span::styled("?", Style::default().fg(Color::Yellow)),
                Span::raw("         "),
                Span::raw("Show the help pop-up window"),
            ]),
            Line::from(vec![
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw("         "),
                Span::raw("Quit the application"),
            ]),
        ])
        .wrap(Wrap { trim: false })
        .block(
            Block::bordered()
                .padding(Padding::symmetric(2, 1))
                .title(Line::from(" Help ").alignment(Alignment::Left))
                .title_style(Style::default().fg(Color::Green))
                .title(
                    Line::from(" Esc ").alignment(Alignment::Right).style(
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::ITALIC),
                    ),
                )
                .border_style(Style::default().fg(Color::Green)),
        );

        frame.render_widget(Clear, area);
        frame.render_widget(help, area);
    }
}
