use std::path::PathBuf;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Tabs};
use ratatui::{Frame, crossterm::event::KeyCode, widgets::Paragraph};

use crate::browse_tui::component::Component;
use crate::browse_tui::message::{AppMessage, Tab};

#[derive(Debug)]
pub struct Header {
    projects_count: usize,
    tooling_count: usize,
    vcs_count: usize,
    browser_path: Option<PathBuf>,
    selected_tab: Tab,
}

impl Header {
    pub fn new(
        selected_tab: Tab,
        projects_count: usize,
        tooling_count: usize,
        vcs_count: usize,
    ) -> Self {
        Self {
            projects_count,
            tooling_count,
            vcs_count,
            browser_path: None,
            selected_tab,
        }
    }

    pub fn set_browser_path(&mut self, path: Option<PathBuf>) {
        self.browser_path = path;
    }

    pub fn set_selected_tab(&mut self, selected_tab: Tab) {
        self.selected_tab = selected_tab;
    }
}

#[derive(Debug)]
pub enum HeaderMessage {}

impl Component for Header {
    type Message = HeaderMessage;
    fn update(&mut self, _message: Self::Message) -> Option<AppMessage> {
        None
    }
    fn handle_key(&mut self, _key: KeyCode) -> Option<Self::Message> {
        None
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" DDC Project Browser ")
            .title_style(Style::default().fg(Color::Cyan))
            .border_style(Style::default().fg(Color::Cyan));

        match &self.browser_path {
            Some(path) => {
                let text = format!(" Browsing: {} ", path.display());
                let header = Paragraph::new(text).block(block);

                frame.render_widget(header, area);
            }
            None => {
                let titles = [
                    Line::from(vec![
                        Span::styled("D", Style::default().add_modifier(Modifier::UNDERLINED)),
                        Span::raw(format!("iscovered Projects ({})", self.projects_count,)),
                    ]),
                    Line::from(vec![
                        Span::styled("T", Style::default().add_modifier(Modifier::UNDERLINED)),
                        Span::raw(format!("ooling Overview ({})", self.tooling_count,)),
                    ]),
                    Line::from(vec![
                        Span::styled("V", Style::default().add_modifier(Modifier::UNDERLINED)),
                        Span::raw(format!("ersion Controlled ({})", self.vcs_count,)),
                    ]),
                ];
                let tabs = Tabs::new(titles)
                    .highlight_style(
                        Style::default()
                            .bg(Color::Blue)
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .select(self.selected_tab.index())
                    .padding(" ", " ")
                    .divider("|")
                    .block(block);
                frame.render_widget(tabs, area);
            }
        }
    }
}
