use std::io;
use std::option::Option;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use ratatui::crossterm::event::KeyEventKind;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Clear, List, ListItem, ListState, Padding, Row, Table, TableState, Tabs,
};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    widgets::Paragraph,
};

use crate::browse_tui::model::{Browser, DirItem, DirectoryBrowserFrame, ResultsTab, Tab};
use crate::discovery::{ProjectResult, ToolingResult};
use crate::display_tools::{ColorCode, get_size_color_code, get_time_color_code};
use crate::files_db::FilesDB;

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(Debug, Default, PartialEq, Eq)]
enum UiMode {
    #[default]
    Normal,
    Help,
}

#[derive(PartialEq)]
enum Message {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    GoBack,
    Refresh,
    Quit,
    SelectTab(usize),
    EnterParent,
    Help,
    Close,
}

enum Navigation {
    Up,
    Down,
    PageUp(usize),
    PageDown(usize),
    Home,
    End,
}

const PROJECTS_TAB: usize = 0;
const TOOLING_TAB: usize = 1;

#[derive(Debug)]
pub struct App {
    db: FilesDB,
    running_state: RunningState,
    mode: UiMode,
    tabs: Vec<Tab>,
    selected_tab: usize,
    browser: Browser,
    error_message: Option<String>,
    now: SystemTime,
    page_size: usize,
}

impl App {
    pub fn new(
        projects_data: Vec<ProjectResult>,
        tooling_data: Vec<ToolingResult>,
        db: FilesDB,
    ) -> Self {
        Self {
            db,
            running_state: RunningState::Running,
            mode: UiMode::Normal,
            tabs: vec![
                Tab::Projects(ResultsTab {
                    current_item: 0,
                    results: projects_data,
                }),
                Tab::Tooling(ResultsTab {
                    current_item: 0,
                    results: tooling_data,
                }),
            ],
            selected_tab: PROJECTS_TAB,
            browser: Browser { frames: vec![] },
            error_message: None,
            now: SystemTime::now(),
            page_size: 0,
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

    fn handle_events(&self) -> io::Result<Option<Message>> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        return Ok(match key.code {
                            KeyCode::Char('r') => Some(Message::Refresh),
                            KeyCode::Char('q') => Some(Message::Quit),
                            KeyCode::Up | KeyCode::Char('k') => Some(Message::MoveUp),
                            KeyCode::Down | KeyCode::Char('j') => Some(Message::MoveDown),
                            KeyCode::Right | KeyCode::Char('l') => Some(Message::Enter),
                            KeyCode::Left | KeyCode::Char('h') => Some(Message::GoBack),
                            KeyCode::Char('p') => Some(Message::EnterParent),
                            KeyCode::PageDown => Some(Message::PageDown),
                            KeyCode::PageUp => Some(Message::PageUp),
                            KeyCode::Home => Some(Message::Home),
                            KeyCode::End => Some(Message::End),
                            KeyCode::Char('d') | KeyCode::Char('1') => {
                                Some(Message::SelectTab(PROJECTS_TAB))
                            }
                            KeyCode::Char('t') | KeyCode::Char('2') => {
                                Some(Message::SelectTab(TOOLING_TAB))
                            }
                            KeyCode::Char('?') => Some(Message::Help),
                            KeyCode::Esc => Some(Message::Close),
                            _ => None,
                        });
                    }
                }
                Event::Resize(_, _) => {
                    return Ok(Some(Message::Refresh));
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        // Removes error message from previous update
        self.error_message = None;

        match self.mode {
            UiMode::Normal => match msg {
                Message::Refresh => {}
                Message::Quit => self.quit(),
                Message::MoveUp => self.move_up(),
                Message::MoveDown => self.move_down(),
                Message::PageUp => self.page_up(),
                Message::PageDown => self.page_down(),
                Message::Home => self.home(),
                Message::End => self.end(),
                Message::Enter => self.enter(),
                Message::GoBack => self.go_back(),
                Message::EnterParent => self.enter_parent(),
                Message::SelectTab(tab) => self.select_tab(tab),
                Message::Help => self.help(),
                _ => {}
            },
            UiMode::Help => match msg {
                Message::Refresh => {}
                Message::Quit => self.quit(),
                Message::Help => self.help(),
                Message::Close => self.close(),
                _ => {}
            },
        };

        None
    }

    fn quit(&mut self) {
        self.running_state = RunningState::Done;
    }

    fn move_up(&mut self) {
        self.navigate(Navigation::Up);
    }

    fn move_down(&mut self) {
        self.navigate(Navigation::Down);
    }

    fn page_up(&mut self) {
        self.navigate(Navigation::PageUp(self.page_size));
    }

    fn page_down(&mut self) {
        self.navigate(Navigation::PageDown(self.page_size));
    }

    fn home(&mut self) {
        self.navigate(Navigation::Home);
    }

    fn end(&mut self) {
        self.navigate(Navigation::End);
    }

    fn navigate(&mut self, nav: Navigation) {
        match self.browser.frames.last_mut() {
            Some(frame) => Self::navigate_browser(frame, nav),
            None => match self.tabs.get_mut(self.selected_tab) {
                Some(Tab::Projects(tab)) => Self::navigate_tab(tab, nav),
                Some(Tab::Tooling(tab)) => Self::navigate_tab(tab, nav),
                None => panic!("Tried to select non-existent tab. This shouldn't happen."),
            },
        }
    }

    fn navigate_browser(frame: &mut DirectoryBrowserFrame, nav: Navigation) {
        let len = frame.directory_list.len();
        frame.current_item = Self::compute_index(frame.current_item, len, nav);
    }

    fn navigate_tab<T>(tab: &mut ResultsTab<T>, nav: Navigation) {
        let len = tab.results.len();
        tab.current_item = Self::compute_index(tab.current_item, len, nav);
    }

    fn compute_index(current: usize, len: usize, nav: Navigation) -> usize {
        if len == 0 {
            return 0;
        }

        match nav {
            Navigation::Up => current.saturating_sub(1),
            Navigation::Down => (current + 1).min(len - 1),
            Navigation::PageUp(n) => current.saturating_sub(n),
            Navigation::PageDown(n) => (current + n).min(len - 1),
            Navigation::Home => 0,
            Navigation::End => len - 1,
        }
    }

    fn enter(&mut self) {
        let path = match self.browser.frames.last_mut() {
            Some(frame) => {
                let Some(item) = frame.directory_list.get(frame.current_item) else {
                    self.error_message = Some(String::from("No item selected."));
                    return;
                };

                if !item.is_directory {
                    self.error_message = Some(String::from("Item is not a directory."));
                    return;
                }

                Some(item.path.clone())
            }
            None => match self.tabs.get(self.selected_tab) {
                Some(Tab::Projects(tab)) => {
                    tab.results.get(tab.current_item).map(|r| r.path.clone())
                }
                Some(Tab::Tooling(tab)) => {
                    tab.results.get(tab.current_item).map(|r| r.path.clone())
                }
                None => None,
            },
        };

        let Some(path) = path else {
            self.error_message = Some(String::from("No item selected."));
            return;
        };

        self.open_directory(path);
    }

    fn enter_parent(&mut self) {
        match self.browser.frames.last_mut() {
            Some(_) => {
                self.error_message = Some(String::from(
                    "Parent traversal is implemented only for projects and tools views.",
                ));
            }
            None => {
                let parent_path = match self.tabs.get(self.selected_tab) {
                    Some(Tab::Projects(tab)) => tab
                        .results
                        .get(tab.current_item)
                        .and_then(|r| r.parent.as_ref())
                        .map(|p| p.path.clone()),
                    Some(Tab::Tooling(tab)) => tab
                        .results
                        .get(tab.current_item)
                        .and_then(|r| r.path.parent())
                        .map(PathBuf::from),
                    None => None,
                };

                let Some(path) = parent_path else {
                    self.error_message = Some(String::from("Unable to detect parent directory."));
                    return;
                };

                self.open_directory(path);
            }
        }
    }

    fn open_directory(&mut self, path: PathBuf) {
        let directory_list: Vec<_> = self
            .db
            .iter_level(&path)
            .map(|fi| DirItem::from_file_info(&fi, &self.db))
            .collect();

        if directory_list.is_empty() {
            self.error_message = Some(String::from("Directory is empty."));
            return;
        }

        self.browser.frames.push(DirectoryBrowserFrame {
            current_item: 0,
            cwd: path.clone(),
            directory_list,
        });
    }

    fn go_back(&mut self) {
        self.browser.frames.pop();
    }

    fn select_tab(&mut self, tab: usize) {
        self.selected_tab = tab;
        self.browser.frames.truncate(0);
    }

    fn help(&mut self) {
        self.mode = match self.mode {
            UiMode::Normal => UiMode::Help,
            UiMode::Help => UiMode::Normal,
        };
    }

    fn close(&mut self) {
        self.mode = UiMode::Normal;
    }

    fn draw(&mut self, frame: &mut Frame) {
        let chunks = self.create_layout(frame.area());

        self.render_header(frame, chunks[0]);
        match self.browser.frames.last() {
            Some(directory_frame) => {
                self.page_size = chunks[1].height.saturating_sub(2) as usize;
                self.render_directory(frame, chunks[1], directory_frame)
            }
            None => match self.tabs.get(self.selected_tab) {
                Some(Tab::Projects(project_tab)) => {
                    self.page_size = chunks[1].height.saturating_sub(3) as usize;
                    self.render_projects(frame, chunks[1], project_tab)
                }
                Some(Tab::Tooling(tooling_tab)) => {
                    self.page_size = chunks[1].height.saturating_sub(3) as usize;
                    self.render_tooling(frame, chunks[1], tooling_tab)
                }
                None => panic!("Tried to select non-existent tab. This shouldn't happen."),
            },
        }
        self.render_footer(frame, chunks[2]);
        if self.mode == UiMode::Help {
            self.render_help_popup(frame, chunks[1]);
        }
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

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        fn get_tab_len(tab: &Tab) -> usize {
            match tab {
                Tab::Projects(projects) => projects.results.len(),
                Tab::Tooling(tooling) => tooling.results.len(),
            }
        }

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" DDC Project Browser ")
            .title_style(Style::default().fg(Color::Cyan))
            .border_style(Style::default().fg(Color::Cyan));

        match self.browser.frames.last() {
            Some(directory_frame) => {
                let text = format!(" Browsing: {} ", directory_frame.cwd.display());
                let header = Paragraph::new(text)
                    .block(block)
                    .style(Style::default().fg(Color::White));

                frame.render_widget(header, area);
            }
            None => {
                let titles = [
                    Line::from(vec![
                        Span::styled("D", Style::default().add_modifier(Modifier::UNDERLINED)),
                        Span::raw(format!(
                            "iscovered Projects ({})",
                            get_tab_len(&self.tabs[PROJECTS_TAB])
                        )),
                    ]),
                    Line::from(vec![
                        Span::styled("T", Style::default().add_modifier(Modifier::UNDERLINED)),
                        Span::raw(format!(
                            "ooling Overview ({})",
                            get_tab_len(&self.tabs[TOOLING_TAB])
                        )),
                    ]),
                ];
                let tabs = Tabs::new(titles)
                    .highlight_style(
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .select(self.selected_tab)
                    .padding(" ", " ")
                    .style(Style::default().fg(Color::DarkGray))
                    .divider("|")
                    .block(block);
                frame.render_widget(tabs, area);
            }
        }
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = if let Some(message) = &self.error_message {
            let msg = format!(" {}", message);
            Paragraph::new(msg)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" ERROR ")
                        .style(
                            Style::default()
                                .fg(Color::LightRed)
                                .add_modifier(Modifier::BOLD),
                        ),
                )
                .style(Style::default().fg(Color::White))
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
                    "d",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Projects View  "),
                Span::styled(
                    "t",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Tools View  "),
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

            Paragraph::new(line)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Line::from(" Controls ").alignment(Alignment::Left))
                        .title_bottom(
                            Line::from(" More controls in the help ")
                                .alignment(Alignment::Right)
                                .style(
                                    Style::default()
                                        .fg(Color::Gray)
                                        .add_modifier(Modifier::ITALIC),
                                ),
                        )
                        .style(Style::default().fg(Color::Green)),
                )
                .style(Style::default().fg(Color::White))
        };

        frame.render_widget(footer, area);
    }

    fn render_help_popup(&self, frame: &mut Frame, area: Rect) {
        let area = popup_area(area, 80, 60);

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
                Span::raw("This is useful if you want to inspect a footprint"),
            ])
            .style(Style::default().fg(Color::Gray)),
            Line::from(vec![
                Span::raw("          "),
                Span::raw("of the whole project, and not just the dev dir."),
            ])
            .style(Style::default().fg(Color::Gray)),
            Line::from(""),
            Line::from(vec![
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw("       "),
                Span::raw("Close any pop-up window, e.g. Help"),
            ]),
            Line::from(vec![
                Span::styled("?", Style::default().fg(Color::Yellow)),
                Span::raw("         "),
                Span::raw("Show/close the help pop-up window"),
            ]),
            Line::from(vec![
                Span::styled("q", Style::default().fg(Color::Yellow)),
                Span::raw("         "),
                Span::raw("Quit the application"),
            ]),
        ])
        .style(Style::default().fg(Color::White))
        .block(
            Block::bordered()
                .style(Style::default().fg(Color::Green))
                .padding(Padding::symmetric(2, 1))
                .title(Line::from(" Help ").alignment(Alignment::Left))
                .title(
                    Line::from(" Esc ").alignment(Alignment::Right).style(
                        Style::default()
                            .fg(Color::Red)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ),
        );

        frame.render_widget(Clear, area);
        frame.render_widget(help, area);
    }

    fn render_projects(
        &self,
        frame: &mut Frame,
        area: Rect,
        projects_tab: &ResultsTab<ProjectResult>,
    ) {
        let rows: Vec<_> = projects_tab
            .results
            .iter()
            .map(|r| self.create_project_row(r))
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Percentage(60),
                Constraint::Length(10),
                Constraint::Length(20),
                Constraint::Length(11),
            ],
        )
        .header(
            Row::new(["", "Project", "Size", "Last update", "Parent size"]).style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Projects ")
                .style(Style::default().fg(Color::LightYellow)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

        let mut table_state = TableState::default();
        table_state.select(Some(projects_tab.current_item));

        frame.render_stateful_widget(table, area, &mut table_state);
    }

    fn create_project_row<'a>(&self, result: &'a ProjectResult) -> Row<'a> {
        Row::new(vec![
            Cell::from(format!("{} ", result.lang)),
            Cell::from(Line::from(result.path.display().to_string())),
            size_cell(result.size),
            last_update_cell(self.now, result.last_update),
            parent_size_cell(result.parent.as_ref().map(|p| p.size)),
        ])
        .style(Style::default().fg(Color::White))
    }

    fn render_tooling(
        &self,
        frame: &mut Frame,
        area: Rect,
        tooling_tab: &ResultsTab<ToolingResult>,
    ) {
        let rows: Vec<_> = tooling_tab
            .results
            .iter()
            .map(|r| self.create_tooling_row(r))
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Length(3),
                Constraint::Percentage(60),
                Constraint::Length(10),
                Constraint::Length(20),
            ],
        )
        .header(
            Row::new(["", "Tool", "Size", "Last update"]).style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Tools ")
                .style(Style::default().fg(Color::LightYellow)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

        let mut table_state = TableState::default();
        table_state.select(Some(tooling_tab.current_item));

        frame.render_stateful_widget(table, area, &mut table_state);
    }

    fn create_tooling_row<'a>(&self, result: &'a ToolingResult) -> Row<'a> {
        Row::new(vec![
            Cell::from(format!("{} ", result.lang)),
            Cell::from(Line::from(vec![
                Span::raw(&result.description),
                Span::styled(
                    format!(" ({})", result.path.display()),
                    Style::default().fg(Color::DarkGray),
                ),
            ])),
            size_cell(result.size),
            last_update_cell(self.now, result.last_update),
        ])
        .style(Style::default().fg(Color::White))
    }

    fn render_directory(
        &self,
        frame: &mut Frame,
        area: Rect,
        directory_frame: &DirectoryBrowserFrame,
    ) {
        let list_items: Vec<ListItem> = directory_frame
            .directory_list
            .iter()
            .map(|path| self.create_directory_list_item(path))
            .collect();

        // Create the list widget
        let list = List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Directory List ")
                    .style(Style::default().fg(Color::LightYellow)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("► ");

        let mut list_state = ListState::default();
        list_state.select(Some(directory_frame.current_item));

        frame.render_stateful_widget(list, area, &mut list_state);
    }

    fn create_directory_list_item<'a>(&self, item: &'a DirItem) -> ListItem<'a> {
        let (icon, name_style) = if item.is_directory {
            ("📁", Style::default().fg(Color::Cyan))
        } else {
            ("📄", Style::default().fg(Color::White))
        };

        let size_text = item
            .size
            .map(|size| format_size(size, DECIMAL))
            .unwrap_or_else(|| "?".to_string());

        let content = Line::from(vec![
            Span::raw(format!("{} ", icon)),
            Span::styled(&item.name, name_style),
            Span::styled(
                format!(" ({})", size_text),
                Style::default().fg(Color::Gray),
            ),
        ]);

        ListItem::new(content)
    }
}

fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

fn size_cell(size: u64) -> Cell<'static> {
    let text = format_size(size, DECIMAL);
    let color = match get_size_color_code(size) {
        ColorCode::None => Color::White,
        ColorCode::Low => Color::Green,
        ColorCode::Medium => Color::Yellow,
        ColorCode::High => Color::Red,
    };

    Cell::from(text).style(Style::default().fg(color))
}

fn last_update_cell(now: SystemTime, last: Option<SystemTime>) -> Cell<'static> {
    let text = last
        .map(|t| {
            DateTime::<Local>::from(t)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_default();

    let color = match get_time_color_code(&now, &last) {
        ColorCode::None => Color::White,
        ColorCode::Low => Color::Green,
        ColorCode::Medium => Color::Yellow,
        ColorCode::High => Color::Red,
    };

    Cell::from(text).style(Style::default().fg(color))
}

fn parent_size_cell(parent_size: Option<u64>) -> Cell<'static> {
    let text = parent_size
        .map(|s| format_size(s, DECIMAL))
        .unwrap_or_default();

    Cell::from(text).style(Style::default().fg(Color::DarkGray))
}
