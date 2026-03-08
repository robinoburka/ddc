use std::io;
use std::option::Option;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use ratatui::crossterm::event::KeyEventKind;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
};

use crate::browse_tui::component::Component;
use crate::browse_tui::components::filter_bar::FilterBar;
use crate::browse_tui::components::sort_modal::SortModal;
use crate::browse_tui::components::{
    DirectoryBrowser, Footer, Header, HelpModal, InfoModal, ProjectsTab, ToolingTab, VcsTab,
};
use crate::browse_tui::message::{AppMessage, SortBy, Tab};
use crate::discovery::{ProjectResult, ToolingResult};
use crate::files_db::FilesDB;
use crate::vcs_postprocess::EnrichedVcsResult;

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

#[derive(Debug)]
enum Message {
    AppMessage(AppMessage),
    ProjectsTab(<ProjectsTab as Component>::Message),
    ToolingTab(<ToolingTab as Component>::Message),
    VcsTab(<VcsTab as Component>::Message),
    DirectoryBrowser(<DirectoryBrowser as Component>::Message),
    InfoModal(<InfoModal as Component>::Message),
    SortModal(<SortModal as Component>::Message),
    Filter(<FilterBar as Component>::Message),
}

#[derive(Debug)]
enum UiLayer {
    Tab,
    Filter,
    Browser,
    Modal(Modal),
}

#[derive(Debug)]
pub struct App {
    // Basic application state
    running_state: RunningState,
    layers: Vec<UiLayer>,
    selected_tab: Tab,
    // Components
    header: Header,
    footer: Footer,
    projects_tab: ProjectsTab,
    tooling_tab: ToolingTab,
    vcs_tab: VcsTab,
    browser: Option<DirectoryBrowser>,
    filter: FilterBar,
    // Helper data
    error_message: Option<String>,
    // Persisting inputs
    db: Rc<FilesDB>,
}

impl App {
    pub fn new(
        projects_data: Vec<ProjectResult>,
        tooling_data: Vec<ToolingResult>,
        vcs_result: Vec<EnrichedVcsResult>,
        db: FilesDB,
    ) -> Self {
        Self {
            running_state: RunningState::default(),
            layers: vec![UiLayer::Tab],
            selected_tab: Tab::default(),
            header: Header::new(
                Tab::default(),
                projects_data.len(),
                tooling_data.len(),
                vcs_result.len(),
            ),
            footer: Footer::new(),
            projects_tab: ProjectsTab::new(projects_data),
            tooling_tab: ToolingTab::new(tooling_data),
            vcs_tab: VcsTab::new(vcs_result),
            browser: None,
            filter: FilterBar::new(),
            error_message: None,
            db: Rc::new(db),
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
        let mut message = match self.layers.last_mut()? {
            UiLayer::Tab => match self.selected_tab {
                Tab::Projects => self.projects_tab.handle_key(key).map(Message::ProjectsTab),
                Tab::Tooling => self.tooling_tab.handle_key(key).map(Message::ToolingTab),
                Tab::Vcs => self.vcs_tab.handle_key(key).map(Message::VcsTab),
            },
            UiLayer::Filter => self.filter.handle_key(key).map(Message::Filter),
            UiLayer::Browser => {
                if let Some(browser) = self.browser.as_mut() {
                    browser.handle_key(key).map(Message::DirectoryBrowser)
                } else {
                    None
                }
            }
            UiLayer::Modal(Modal::Help(_)) => None,
            UiLayer::Modal(Modal::Info(info_modal)) => {
                info_modal.handle_key(key).map(Message::InfoModal)
            }
            UiLayer::Modal(Modal::Sort(sort_modal)) => {
                sort_modal.handle_key(key).map(Message::SortModal)
            }
        };
        if message.is_none() {
            message = match key {
                KeyCode::Char('q') => Some(Message::AppMessage(AppMessage::Quit)),
                KeyCode::Char('r') => Some(Message::AppMessage(AppMessage::Refresh)),
                KeyCode::Char('d') | KeyCode::Char('1') => {
                    Some(Message::AppMessage(AppMessage::SelectTab(Tab::Projects)))
                }
                KeyCode::Char('t') | KeyCode::Char('2') => {
                    Some(Message::AppMessage(AppMessage::SelectTab(Tab::Tooling)))
                }
                KeyCode::Char('v') | KeyCode::Char('3') => {
                    Some(Message::AppMessage(AppMessage::SelectTab(Tab::Vcs)))
                }
                KeyCode::Char('?') => Some(Message::AppMessage(AppMessage::OpenHelp)),
                KeyCode::Esc => Some(Message::AppMessage(AppMessage::CloseModal)),
                _ => None,
            };
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
            Message::VcsTab(msg) => self.vcs_tab.update(msg).map(Message::AppMessage),
            Message::Filter(msg) => self.filter.update(msg).map(Message::AppMessage),
            Message::DirectoryBrowser(msg) => {
                if let Some(browser) = self.browser.as_mut() {
                    browser.update(msg).map(Message::AppMessage)
                } else {
                    None
                }
            }
            Message::InfoModal(msg) => {
                if let Some(UiLayer::Modal(Modal::Info(info_modal))) = self.layers.last_mut() {
                    info_modal.update(msg).map(Message::AppMessage)
                } else {
                    None
                }
            }
            Message::SortModal(msg) => {
                if let Some(UiLayer::Modal(Modal::Sort(sort_modal))) = self.layers.last_mut() {
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
            AppMessage::CloseModal => self.close_modal(),
            AppMessage::OpenHelp => self.open_help(),
            AppMessage::CloseBrowser => self.close_browser(),
            AppMessage::EnterBrowser(path) => self.enter_browser(path),
            AppMessage::SelectTab(i) => self.select_tab(i),
            AppMessage::SelectTabWithFilter(tab, filter) => {
                self.select_tab_with_filter(tab, filter)
            }
            AppMessage::OpenInfo(text) => self.open_info(text),
            AppMessage::OpenSort(options) => self.open_sort(options),
            AppMessage::RequestSort(sort_by) => {
                return self.request_sort(sort_by);
            }
            AppMessage::StartFilter => {
                return self.start_filter();
            }
            AppMessage::AcceptFilter => self.accept_filter(),
            AppMessage::DismissFilter => self.dismiss_filter(),
        }
        None
    }

    fn quit(&mut self) {
        self.running_state = RunningState::Done;
    }

    fn open_help(&mut self) {
        if !matches!(self.layers.last_mut(), Some(UiLayer::Modal(_))) {
            self.layers
                .push(UiLayer::Modal(Modal::Help(HelpModal::new())));
        }
    }

    fn close_modal(&mut self) {
        if matches!(self.layers.last_mut(), Some(UiLayer::Modal(_))) {
            self.layers.pop();
        }
    }

    fn enter_browser(&mut self, path: PathBuf) {
        match DirectoryBrowser::new(self.db.clone(), path) {
            Ok(browser) => {
                self.browser = Some(browser);
                self.layers.push(UiLayer::Browser);
            }
            Err(msg) => self.error_message = Some(msg),
        }
    }

    fn close_browser(&mut self) {
        if matches!(self.layers.last_mut(), Some(UiLayer::Browser)) {
            self.layers.pop();
            self.browser = None;
        }
    }

    fn select_tab(&mut self, tab: Tab) {
        if self.selected_tab != tab {
            self.filter.clear();
        }
        self.layers.clear();
        self.layers.push(UiLayer::Tab);
        self.selected_tab = tab;
        self.browser = None;
    }

    fn select_tab_with_filter(&mut self, tab: Tab, filter: String) {
        self.select_tab(tab);
        self.filter.set_filter(filter);
    }

    fn open_info(&mut self, text: &'static str) {
        self.layers
            .push(UiLayer::Modal(Modal::Info(InfoModal::new(text))));
    }

    fn open_sort(&mut self, options: &'static [SortBy]) {
        self.layers
            .push(UiLayer::Modal(Modal::Sort(SortModal::new(options))));
    }

    fn request_sort(&mut self, sort_by: SortBy) -> Option<Message> {
        if matches!(self.layers.last_mut(), Some(UiLayer::Modal(Modal::Sort(_)))) {
            self.layers.pop();
        }
        match self.layers.last_mut() {
            Some(UiLayer::Tab) => match self.selected_tab {
                Tab::Projects => Some(Message::ProjectsTab(
                    <ProjectsTab as Component>::Message::ApplySort(sort_by),
                )),
                Tab::Tooling => Some(Message::ToolingTab(
                    <ToolingTab as Component>::Message::ApplySort(sort_by),
                )),
                Tab::Vcs => Some(Message::VcsTab(<VcsTab as Component>::Message::ApplySort(
                    sort_by,
                ))),
            },
            _ => None,
        }
    }

    fn start_filter(&mut self) -> Option<Message> {
        self.layers.push(UiLayer::Filter);
        Some(Message::Filter(<FilterBar as Component>::Message::Activate))
    }

    fn accept_filter(&mut self) {
        if matches!(self.layers.last_mut(), Some(UiLayer::Filter)) {
            self.layers.pop();
        }
    }

    fn dismiss_filter(&mut self) {
        if matches!(self.layers.last_mut(), Some(UiLayer::Filter)) {
            self.layers.pop();
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        // Handle data exchange among components
        self.footer.set_error(self.error_message.clone());
        self.header.set_selected_tab(self.selected_tab);
        self.header
            .set_browser_path(self.browser.as_mut().and_then(|b| b.get_current_path()));

        // Render the whole app
        let chunks = self.create_layout(frame.area(), self.filter.is_active());

        self.header.render(frame, chunks[0]);
        if let Some(browser) = self.browser.as_mut() {
            browser.render(frame, chunks[1]);
        } else {
            match self.selected_tab {
                Tab::Projects => {
                    self.projects_tab.apply_filter(self.filter.get_filter());
                    self.projects_tab.render(frame, chunks[1]);
                }
                Tab::Tooling => self.tooling_tab.render(frame, chunks[1]),
                Tab::Vcs => {
                    self.vcs_tab.apply_filter(self.filter.get_filter());
                    self.vcs_tab.render(frame, chunks[1]);
                }
            }
        }
        match self.layers.last_mut() {
            Some(UiLayer::Modal(Modal::Help(help_modal))) => help_modal.render(frame, chunks[1]),
            Some(UiLayer::Modal(Modal::Info(info_modal))) => info_modal.render(frame, chunks[1]),
            Some(UiLayer::Modal(Modal::Sort(sort_modal))) => sort_modal.render(frame, chunks[1]),
            _ => {}
        }
        if self.filter.is_active() {
            self.filter.render(frame, chunks[2]);
            self.footer.render(frame, chunks[3]);
        } else {
            self.footer.render(frame, chunks[2]);
        }
    }

    fn create_layout(&self, area: Rect, with_filter: bool) -> Vec<Rect> {
        if with_filter {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .split(area)
                .to_vec()
        } else {
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
    }
}
