use std::path::PathBuf;
use std::rc::Rc;

use iced::widget::{
    button, center, column, container, mouse_area, opaque, row, scrollable, space, stack, text,
};
use iced::{Color, Element, Length, Subscription, Task, Theme, keyboard};

use super::browser::{self, DirectoryBrowser};
use super::list::Nav;
use super::table::{self, Record, SortBy, Table};
use super::tabs;
use crate::discovery::{ProjectResult, ToolingResult};
use crate::files_db::FilesDB;
use crate::vcs_postprocess::EnrichedVcsResult;

#[derive(Debug, Clone)]
pub enum Message {
    // Each tab has its own variant, so late events (e.g. scrolling) reach the right table
    Projects(table::Message),
    Tooling(table::Message),
    Vcs(table::Message),
    Browser(browser::Message),
    SelectTab(Tab),
    // Keyboard-driven, routed to whichever layer is on top
    Nav(Nav),
    Enter,
    EnterParent,
    Back,
    ShowDetectedProjects,
    // Modals
    OpenHelp,
    OpenSort,
    OpenInfo,
    CloseModal,
    Sort(SortBy),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Projects,
    Tooling,
    Vcs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modal {
    Help,
    Sort,
    Info(&'static str),
}

pub struct App {
    tab: Tab,
    projects: Table<ProjectResult>,
    tooling: Table<ToolingResult>,
    vcs: Table<EnrichedVcsResult>,
    browser: Option<DirectoryBrowser>,
    modal: Option<Modal>,
    error: Option<String>,
    db: Rc<FilesDB>,
}

impl App {
    pub fn new(
        projects: Vec<ProjectResult>,
        tooling: Vec<ToolingResult>,
        vcs: Vec<EnrichedVcsResult>,
        db: Rc<FilesDB>,
    ) -> Self {
        Self {
            tab: Tab::default(),
            projects: tabs::projects(projects),
            tooling: tabs::tooling(tooling),
            vcs: tabs::vcs(vcs),
            browser: None,
            modal: None,
            error: None,
            db,
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        // Errors are shown until the next thing happens
        self.error = None;

        // Everything but closing is blocked while a modal is up
        if self.modal.is_some() {
            return match message {
                Message::Back | Message::CloseModal => {
                    self.modal = None;
                    Task::none()
                }
                Message::Sort(sort_by) => {
                    self.modal = None;
                    self.update_table(self.tab, table::Message::SortBy(sort_by))
                }
                _ => Task::none(),
            };
        }

        match message {
            Message::Projects(msg) => self.table_message(Tab::Projects, msg),
            Message::Tooling(msg) => self.table_message(Tab::Tooling, msg),
            Message::Vcs(msg) => self.table_message(Tab::Vcs, msg),
            Message::Browser(msg) => {
                let open = matches!(msg, browser::Message::Open(_));
                let Some(browser) = self.browser.as_mut() else {
                    return Task::none();
                };
                let (task, event) = browser.update(msg);
                self.handle_browser_event(event);
                if open {
                    let event = self.browser.as_mut().and_then(|b| b.enter());
                    self.handle_browser_event(event);
                }
                task.map(Message::Browser)
            }
            Message::SelectTab(tab) => {
                self.select_tab(tab);
                Task::none()
            }
            Message::Nav(nav) => match self.browser.as_mut() {
                Some(browser) => browser
                    .update(browser::Message::Nav(nav))
                    .0
                    .map(Message::Browser),
                None => self.update_table(self.tab, table::Message::Nav(nav)),
            },
            Message::Enter => {
                self.enter();
                Task::none()
            }
            Message::EnterParent => {
                if self.browser.is_none() {
                    self.enter_parent();
                }
                Task::none()
            }
            Message::Back => {
                if let Some(browser) = self.browser.as_mut() {
                    let event = browser.back();
                    self.handle_browser_event(event);
                }
                Task::none()
            }
            Message::ShowDetectedProjects => {
                self.show_detected_projects();
                Task::none()
            }
            Message::OpenHelp => {
                self.modal = Some(Modal::Help);
                Task::none()
            }
            Message::OpenSort => {
                if self.browser.is_none() {
                    self.modal = Some(Modal::Sort);
                }
                Task::none()
            }
            Message::OpenInfo => {
                if self.browser.is_none() && self.tab == Tab::Tooling {
                    match self.tooling.selected_record().and_then(|r| r.info) {
                        Some(info) => self.modal = Some(Modal::Info(info)),
                        None => self.error = Some(String::from("There is no info for this item.")),
                    }
                }
                Task::none()
            }
            Message::CloseModal => Task::none(),
            Message::Sort(sort_by) => self.update_table(self.tab, table::Message::SortBy(sort_by)),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Letters never trigger shortcuts on their own, so typing into the filter box is safe.
        keyboard::listen().filter_map(|event| {
            let keyboard::Event::KeyPressed { key, modifiers, .. } = event else {
                return None;
            };
            use keyboard::key::Named;
            match key.as_ref() {
                keyboard::Key::Named(named) => match named {
                    Named::ArrowUp => Some(Message::Nav(Nav::Up)),
                    Named::ArrowDown => Some(Message::Nav(Nav::Down)),
                    Named::PageUp => Some(Message::Nav(Nav::PageUp)),
                    Named::PageDown => Some(Message::Nav(Nav::PageDown)),
                    Named::Home => Some(Message::Nav(Nav::Home)),
                    Named::End => Some(Message::Nav(Nav::End)),
                    Named::Enter if modifiers.shift() => Some(Message::EnterParent),
                    Named::Enter => Some(Message::Enter),
                    Named::Escape => Some(Message::Back),
                    Named::F1 => Some(Message::OpenHelp),
                    Named::F2 => Some(Message::OpenInfo),
                    Named::F3 => Some(Message::ShowDetectedProjects),
                    _ => None,
                },
                keyboard::Key::Character("1") if modifiers.command() => {
                    Some(Message::SelectTab(Tab::Projects))
                }
                keyboard::Key::Character("2") if modifiers.command() => {
                    Some(Message::SelectTab(Tab::Tooling))
                }
                keyboard::Key::Character("3") if modifiers.command() => {
                    Some(Message::SelectTab(Tab::Vcs))
                }
                _ => None,
            }
        })
    }

    pub fn view(&self) -> Element<'_, Message> {
        let in_browser = self.browser.is_some();

        let tab_bar = row![
            self.tab_button("Projects", self.projects.len(), Tab::Projects),
            self.tab_button("Tooling", self.tooling.len(), Tab::Tooling),
            self.tab_button("Version controlled", self.vcs.len(), Tab::Vcs),
        ]
        .spacing(8);

        let title = match self.browser.as_ref().and_then(|b| b.current_path()) {
            Some(path) => text(path.display().to_string()),
            None => text(""),
        };
        let toolbar = row![
            tab_bar,
            title,
            space::horizontal(),
            button("Sort…").on_press_maybe((!in_browser).then_some(Message::OpenSort)),
            button("Info (F2)").on_press_maybe(
                (!in_browser && self.tab == Tab::Tooling).then_some(Message::OpenInfo)
            ),
            button("Show projects (F3)").on_press_maybe(
                (!in_browser && self.tab == Tab::Vcs).then_some(Message::ShowDetectedProjects)
            ),
            button("Help (F1)").on_press(Message::OpenHelp),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        let body: Element<'_, Message> = match (self.browser.as_ref(), self.tab) {
            (Some(browser), _) => browser.view().map(Message::Browser),
            (None, Tab::Projects) => self.projects.view().map(Message::Projects),
            (None, Tab::Tooling) => self.tooling.view().map(Message::Tooling),
            (None, Tab::Vcs) => self.vcs.view().map(Message::Vcs),
        };

        let status = match self.error.as_ref() {
            Some(err) => text(err.as_str()).color(Color::from_rgb(0.9, 0.4, 0.4)),
            None => text("↑↓ navigate · Enter open · Shift+Enter open parent · Esc back"),
        };

        let base = column![toolbar, body, status].spacing(8).padding(12);

        match self.modal {
            Some(Modal::Help) => modal(base, help_view()),
            Some(Modal::Sort) => modal(base, sort_view(self.active_sort_options())),
            Some(Modal::Info(info)) => modal(base, info_view(info)),
            None => base.into(),
        }
    }

    fn tab_button(&self, label: &str, count: usize, tab: Tab) -> Element<'_, Message> {
        button(text(format!("{label} ({count})")))
            .on_press(Message::SelectTab(tab))
            .style(if self.tab == tab {
                button::primary
            } else {
                button::secondary
            })
            .into()
    }

    fn table_message(&mut self, tab: Tab, msg: table::Message) -> Task<Message> {
        let open = matches!(msg, table::Message::Open(_));
        let task = self.update_table(tab, msg);
        if open {
            self.enter();
        }
        task
    }

    fn update_table(&mut self, tab: Tab, msg: table::Message) -> Task<Message> {
        match tab {
            Tab::Projects => self.projects.update(msg).map(Message::Projects),
            Tab::Tooling => self.tooling.update(msg).map(Message::Tooling),
            Tab::Vcs => self.vcs.update(msg).map(Message::Vcs),
        }
    }

    fn active_sort_options(&self) -> &'static [SortBy] {
        match self.tab {
            Tab::Projects => self.projects.sort_options(),
            Tab::Tooling => self.tooling.sort_options(),
            Tab::Vcs => self.vcs.sort_options(),
        }
    }

    fn selected_path(&self) -> Option<PathBuf> {
        match self.tab {
            Tab::Projects => self
                .projects
                .selected_record()
                .map(|r| r.path().to_path_buf()),
            Tab::Tooling => self
                .tooling
                .selected_record()
                .map(|r| r.path().to_path_buf()),
            Tab::Vcs => self.vcs.selected_record().map(|r| r.path().to_path_buf()),
        }
    }

    fn select_tab(&mut self, tab: Tab) {
        self.tab = tab;
        self.browser = None;
    }

    /// Enters the selected directory in the browser, or goes deeper if it's already open.
    fn enter(&mut self) {
        match self.browser.as_mut() {
            Some(browser) => {
                let event = browser.enter();
                self.handle_browser_event(event);
            }
            None => match self.selected_path() {
                Some(path) => self.open_browser(path),
                None => self.error = Some(String::from("Nothing is selected.")),
            },
        }
    }

    fn enter_parent(&mut self) {
        let parent = match self.tab {
            Tab::Projects => self
                .projects
                .selected_record()
                .and_then(|r| r.parent.as_ref())
                .map(|p| p.path.clone()),
            Tab::Tooling => self
                .tooling
                .selected_record()
                .and_then(|r| r.path.parent())
                .map(PathBuf::from),
            Tab::Vcs => {
                self.error = Some(String::from(
                    "Not available here. Open the repository directly instead.",
                ));
                return;
            }
        };
        match parent {
            Some(path) => self.open_browser(path),
            None => self.error = Some(String::from("This item has no parent.")),
        }
    }

    fn show_detected_projects(&mut self) {
        if self.browser.is_some() || self.tab != Tab::Vcs {
            return;
        }
        let Some(record) = self.vcs.selected_record() else {
            return;
        };
        if record.matched_projects.is_empty() {
            self.error = Some(String::from("This record has no detected projects."));
            return;
        }

        let filter = record.path.to_string_lossy().to_string();
        self.select_tab(Tab::Projects);
        self.projects.set_filter(filter);
    }

    fn open_browser(&mut self, path: PathBuf) {
        match DirectoryBrowser::new(self.db.clone(), path) {
            Ok(browser) => self.browser = Some(browser),
            Err(msg) => self.error = Some(msg),
        }
    }

    fn handle_browser_event(&mut self, event: Option<browser::Event>) {
        match event {
            Some(browser::Event::Close) => self.browser = None,
            Some(browser::Event::Error(msg)) => self.error = Some(msg),
            None => {}
        }
    }
}

/// Dims `base` and shows `content` centered above it. Clicking outside closes it.
fn modal<'a>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    stack![
        base.into(),
        opaque(
            mouse_area(center(opaque(content)).style(|_theme: &Theme| {
                container::Style {
                    background: Some(
                        Color {
                            a: 0.7,
                            ..Color::BLACK
                        }
                        .into(),
                    ),
                    ..container::Style::default()
                }
            }))
            .on_press(Message::CloseModal)
        )
    ]
    .into()
}

fn help_view<'a>() -> Element<'a, Message> {
    const KEYS: [(&str, &str); 10] = [
        ("Ctrl/Cmd + 1 2 3", "Switch tab"),
        ("↑ ↓", "Move the selection"),
        ("PageUp PageDown", "Move by one page"),
        ("Home End", "Jump to the first or last item"),
        ("Enter", "Open the selected item (or double-click)"),
        ("Shift+Enter", "Open the parent of the selected item"),
        ("Esc", "Go back / close this window"),
        ("F1", "Show this help"),
        ("F2", "Info about the selected tool (Tooling tab)"),
        (
            "F3",
            "Show projects of a repository (Version controlled tab)",
        ),
    ];

    let mut rows = column![text("Keyboard shortcuts").size(20)].spacing(8);
    for (keys, description) in KEYS {
        rows = rows.push(
            row![
                text(keys)
                    .color(Color::from_rgb(0.9, 0.8, 0.3))
                    .width(Length::Fixed(150.0)),
                text(description),
            ]
            .spacing(8),
        );
    }
    rows =
        rows.push(text("Letter keys are left alone so you can type in the filter box.").size(13));

    dialog(rows, 560.0)
}

fn sort_view<'a>(options: &[SortBy]) -> Element<'a, Message> {
    let mut list = column![text("Sort by").size(20)].spacing(8);
    for &sort_by in options {
        list = list.push(
            button(text(sort_by.label()))
                .on_press(Message::Sort(sort_by))
                .width(Length::Fill),
        );
    }
    list = list.push(text("Choosing the same option again reverses the order.").size(13));

    dialog(list, 320.0)
}

fn info_view<'a>(info: &'static str) -> Element<'a, Message> {
    let content = column![
        text("Info").size(20),
        scrollable(text(info)).height(Length::Shrink),
    ]
    .spacing(8)
    .max_width(640.0);

    dialog(content, 680.0)
}

fn dialog<'a>(content: impl Into<Element<'a, Message>>, width: f32) -> Element<'a, Message> {
    container(content)
        .padding(20)
        .width(Length::Fixed(width))
        .max_height(500.0)
        .style(container::rounded_box)
        .into()
}
