use std::io;
use std::option::Option;
use std::path::PathBuf;
use std::time::Duration;

use ratatui::crossterm::event::KeyEventKind;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
};

use crate::browse_tui::component::Component;
use crate::browse_tui::components::sort_modal::SortModal;
use crate::browse_tui::components::{
    DirectoryBrowser, Footer, Header, HelpModal, InfoModal, ProjectsTab, ToolingTab,
};
use crate::browse_tui::message::{AppMessage, SortBy, Tab};
use crate::discovery::{ProjectResult, ToolingResult};
use crate::files_db::FilesDB;

#[derive(Debug, Default, Eq, PartialEq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug)]
enum Modal {
    Help(HelpModal),
    Info(InfoModal),
    Sort(SortModal),
}

#[derive(Debug, Default)]
enum UiMode {
    #[default]
    Normal,
    Modal(Modal),
}

#[derive(Debug)]
enum Message {
    AppMessage(AppMessage),
    ProjectsTab(<ProjectsTab as Component>::Message),
    ToolingTab(<ToolingTab as Component>::Message),
    DirectoryBrowser(<DirectoryBrowser as Component>::Message),
    InfoModal(<InfoModal as Component>::Message),
    SortModal(<SortModal as Component>::Message),
}

#[derive(Debug)]
pub struct App {
    // Basic application state
    running_state: RunningState,
    mode: UiMode,
    tab: Tab,
    // Components
    header: Header,
    footer: Footer,
    projects_tab: ProjectsTab,
    tooling_tab: ToolingTab,
    browser: DirectoryBrowser,
    // Helper data
    error_message: Option<String>,
}

impl App {
    pub fn new(
        projects_data: Vec<ProjectResult>,
        tooling_data: Vec<ToolingResult>,
        db: FilesDB,
    ) -> Self {
        Self {
            running_state: RunningState::default(),
            mode: UiMode::default(),
            tab: Tab::default(),
            header: Header::new(Tab::default(), projects_data.len(), tooling_data.len()),
            footer: Footer::new(),
            projects_tab: ProjectsTab::new(projects_data),
            tooling_tab: ToolingTab::new(tooling_data),
            browser: DirectoryBrowser::new(db),
            error_message: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while self.running_state != RunningState::Done {
            terminal.draw(|frame| self.draw(frame))?;

            let mut current_msg = self.handle_events()?;

            while current_msg.is_some() {
                current_msg = self.update(current_msg.unwrap());
            }
        }

        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<Option<Message>> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    return Ok(self.handle_key(key.code));
                }
                Event::Resize(_, _) => {
                    return Ok(Some(Message::AppMessage(AppMessage::Refresh)));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn handle_key(&mut self, key: KeyCode) -> Option<Message> {
        let mut message = match key {
            KeyCode::Char('q') => Some(Message::AppMessage(AppMessage::Quit)),
            KeyCode::Char('r') => Some(Message::AppMessage(AppMessage::Refresh)),
            KeyCode::Char('d') | KeyCode::Char('1') => {
                Some(Message::AppMessage(AppMessage::SelectTab(Tab::Projects)))
            }
            KeyCode::Char('t') | KeyCode::Char('2') => {
                Some(Message::AppMessage(AppMessage::SelectTab(Tab::Tooling)))
            }
            KeyCode::Char('?') => Some(Message::AppMessage(AppMessage::OpenHelp)),
            KeyCode::Esc => Some(Message::AppMessage(AppMessage::CloseModal)),
            _ => None,
        };
        if message.is_none() {
            message = match &mut self.mode {
                UiMode::Normal => match (self.browser.is_clear(), self.tab) {
                    (true, Tab::Projects) => {
                        self.projects_tab.handle_key(key).map(Message::ProjectsTab)
                    }
                    (true, Tab::Tooling) => {
                        self.tooling_tab.handle_key(key).map(Message::ToolingTab)
                    }
                    (false, _) => self.browser.handle_key(key).map(Message::DirectoryBrowser),
                },
                UiMode::Modal(Modal::Help(_)) => None,
                UiMode::Modal(Modal::Info(info_modal)) => {
                    info_modal.handle_key(key).map(Message::InfoModal)
                }
                UiMode::Modal(Modal::Sort(sort_modal)) => {
                    sort_modal.handle_key(key).map(Message::SortModal)
                }
            }
        }
        message
    }

    fn update(&mut self, message: Message) -> Option<Message> {
        // Clear error message from previous update
        self.error_message = None;

        match message {
            Message::AppMessage(msg) => self.handle_app_message(msg),
            Message::ProjectsTab(msg) => self.projects_tab.update(msg).map(Message::AppMessage),
            Message::ToolingTab(msg) => self.tooling_tab.update(msg).map(Message::AppMessage),
            Message::DirectoryBrowser(msg) => self.browser.update(msg).map(Message::AppMessage),
            Message::InfoModal(msg) => {
                if let UiMode::Modal(Modal::Info(info_modal)) = &mut self.mode {
                    info_modal.update(msg).map(Message::AppMessage)
                } else {
                    None
                }
            }
            Message::SortModal(msg) => {
                if let UiMode::Modal(Modal::Sort(sort_modal)) = &mut self.mode {
                    sort_modal.update(msg).map(Message::AppMessage)
                } else {
                    None
                }
            }
        }
    }

    fn handle_app_message(&mut self, message: AppMessage) -> Option<Message> {
        match message {
            AppMessage::Quit => self.quit(),
            AppMessage::Refresh => {}
            AppMessage::SetError(err) => self.error_message = Some(err),
            AppMessage::OpenHelp => self.open_help(),
            AppMessage::CloseModal => self.close_modal(),
            AppMessage::EnterBrowser(path) => self.enter_browser(path),
            AppMessage::SelectTab(i) => self.select_tab(i),
            AppMessage::OpenInfo(text) => self.open_info(text),
            AppMessage::OpenSort(options) => self.open_sort(options),
            AppMessage::RequestSort(sort_by) => {
                return self.request_sort(sort_by);
            }
        }
        None
    }

    fn quit(&mut self) {
        self.running_state = RunningState::Done;
    }

    fn open_help(&mut self) {
        self.mode = UiMode::Modal(Modal::Help(HelpModal::new()));
    }

    fn close_modal(&mut self) {
        self.mode = UiMode::Normal;
    }

    fn enter_browser(&mut self, path: PathBuf) {
        if let Err(msg) = self.browser.open_path(path) {
            self.error_message = Some(msg);
        }
    }

    fn select_tab(&mut self, tab: Tab) {
        self.mode = UiMode::Normal;
        self.browser.clear();
        self.tab = tab;
    }

    fn open_info(&mut self, text: &'static str) {
        self.mode = UiMode::Modal(Modal::Info(InfoModal::new(text)))
    }

    fn open_sort(&mut self, options: &'static [SortBy]) {
        self.mode = UiMode::Modal(Modal::Sort(SortModal::new(options)))
    }

    fn request_sort(&mut self, sort_by: SortBy) -> Option<Message> {
        self.mode = UiMode::Normal;
        match (self.browser.is_clear(), self.tab) {
            (true, Tab::Projects) => Some(Message::ProjectsTab(
                <ProjectsTab as Component>::Message::ApplySort(sort_by),
            )),
            (true, Tab::Tooling) => Some(Message::ToolingTab(
                <ToolingTab as Component>::Message::ApplySort(sort_by),
            )),
            (false, _) => None,
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        // Handle data exchange among components
        self.header.set_selected_tab(self.tab);
        self.header
            .set_browser_path(self.browser.get_current_path());
        self.footer.set_error(self.error_message.clone());

        // Render the whole app
        let chunks = self.create_layout(frame.area());

        self.header.render(frame, chunks[0]);
        match (self.browser.is_clear(), self.tab) {
            (true, Tab::Projects) => self.projects_tab.render(frame, chunks[1]),
            (true, Tab::Tooling) => self.tooling_tab.render(frame, chunks[1]),
            (false, _) => self.browser.render(frame, chunks[1]),
        }
        self.footer.render(frame, chunks[2]);
        self.render_modal(frame, chunks[1]);
    }

    fn create_layout(&self, area: Rect) -> Vec<Rect> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area)
            .to_vec()
    }

    fn render_modal(&mut self, frame: &mut Frame, area: Rect) {
        if let UiMode::Modal(modal) = &mut self.mode {
            match modal {
                Modal::Help(component) => component.render(frame, area),
                Modal::Info(info_modal) => info_modal.render(frame, area),
                Modal::Sort(sort_modal) => sort_modal.render(frame, area),
            }
        }
    }
}
