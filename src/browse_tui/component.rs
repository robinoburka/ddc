use ratatui::crossterm::event::KeyCode;
use ratatui::{Frame, layout::Rect};

use crate::browse_tui::message::AppMessage;

pub trait Component {
    type Message;
    fn update(&mut self, message: Self::Message) -> Option<AppMessage>;
    fn handle_key(&mut self, key: KeyCode) -> Option<Self::Message>;
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

pub trait Navigable {
    fn move_up(&mut self);
    fn move_down(&mut self);
    fn page_up(&mut self);
    fn page_down(&mut self);
    fn home(&mut self);
    fn end(&mut self);
}
