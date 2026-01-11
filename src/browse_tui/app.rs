use std::io;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use ratatui::crossterm::event::KeyEventKind;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, List, ListItem, ListState, Row, Table, TableState, Tabs,
};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
    widgets::Paragraph,
};

use crate::browse_tui::model::{
    BrowserFrame, DirItem, DirectoryFrame, ProjectsFrame, ProjectsFrameView, Tab,
};
use crate::discovery::{DiscoveryResult, ResultType};
use crate::display_tools::{ColorCode, get_size_color_code, get_time_color_code};
use crate::files_db::FilesDB;

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(PartialEq)]
enum Message {
    MoveUp,
    MoveDown,
    Enter,
    GoBack,
    Refresh,
    Quit,
    SelectTab(Tab),
    EnterParent,
}

#[derive(Debug)]
pub struct App {
    db: FilesDB,
    running_state: RunningState,
    frames: Vec<BrowserFrame>,
    error_message: Option<String>,
    now: SystemTime,
}

impl App {
    pub fn new(db: FilesDB, results: Vec<DiscoveryResult>) -> Self {
        let mut discovery_data = vec![];
        let mut static_data = vec![];

        for result in results {
            if result.size == 0 {
                continue;
            }
            match result.result_type {
                ResultType::Discovery => discovery_data.push(result),
                ResultType::Static(_) => static_data.push(result),
            }
        }

        Self {
            db,
            running_state: RunningState::Running,
            frames: vec![BrowserFrame::Projects(ProjectsFrame {
                current_view: Tab::Projects,
                projects: ProjectsFrameView {
                    current_item: 0,
                    results: discovery_data,
                },
                tools: ProjectsFrameView {
                    current_item: 0,
                    results: static_data,
                },
            })],
            error_message: None,
            now: SystemTime::now(),
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
                            KeyCode::Char('q') | KeyCode::Esc => Some(Message::Quit),
                            KeyCode::Up => Some(Message::MoveUp),
                            KeyCode::Down => Some(Message::MoveDown),
                            KeyCode::Right | KeyCode::Enter => Some(Message::Enter),
                            KeyCode::Left => Some(Message::GoBack),
                            KeyCode::Char('r') => Some(Message::Refresh),
                            KeyCode::Char('1') => Some(Message::SelectTab(Tab::Projects)),
                            KeyCode::Char('2') => Some(Message::SelectTab(Tab::Tools)),
                            KeyCode::Char('p') => Some(Message::EnterParent),
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

        match msg {
            Message::Quit => self.quit(),
            Message::MoveUp => self.move_up(),
            Message::MoveDown => self.move_down(),
            Message::Enter => self.enter(),
            Message::GoBack => self.go_back(),
            Message::SelectTab(tab) => self.select_tab(tab),
            Message::EnterParent => self.enter_parent(),
            _ => {
                self.error_message = Some(String::from("Operation not implemented yet"));
            }
        };

        None
    }

    fn quit(&mut self) {
        self.running_state = RunningState::Done;
    }

    fn move_up(&mut self) {
        match self.frames.last_mut() {
            Some(BrowserFrame::Projects(projects)) => {
                let view = projects.get_mut_view();
                view.current_item = if view.current_item > 0 {
                    view.current_item - 1
                } else {
                    view.current_item
                }
            }
            Some(BrowserFrame::Directory(directory)) => {
                directory.current_item = if directory.current_item > 0 {
                    directory.current_item - 1
                } else {
                    directory.current_item
                }
            }
            None => panic!("Missing frame. This shouldn't happen."),
        }
    }

    fn move_down(&mut self) {
        match self.frames.last_mut() {
            Some(BrowserFrame::Projects(projects)) => {
                let view = projects.get_mut_view();
                view.current_item = if view.current_item < view.results.len() {
                    view.current_item + 1
                } else {
                    view.results.len()
                }
            }
            Some(BrowserFrame::Directory(directory)) => {
                directory.current_item = if directory.current_item < directory.directory_list.len()
                {
                    directory.current_item + 1
                } else {
                    directory.directory_list.len()
                }
            }
            None => panic!("Missing frame. This shouldn't happen."),
        }
    }

    fn enter(&mut self) {
        match self.frames.last_mut() {
            Some(BrowserFrame::Projects(projects)) => {
                let view = projects.get_mut_view();
                let requested_path = view.results[view.current_item].path.clone();
                let new_frame = BrowserFrame::Directory(DirectoryFrame {
                    current_item: 0,
                    directory_list: self
                        .db
                        .iter_level(&requested_path)
                        .map(|fi| DirItem::from_file_info(&fi, &self.db))
                        .collect(),
                    cwd: requested_path,
                });
                self.frames.push(new_frame);
            }
            Some(BrowserFrame::Directory(directory)) => {
                let Some(requested_item) = directory.directory_list.get(directory.current_item)
                else {
                    self.error_message = Some(String::from("No item selected"));
                    return;
                };
                if requested_item.is_directory {
                    let directory_list: Vec<_> = self
                        .db
                        .iter_level(&requested_item.path)
                        .map(|fi| DirItem::from_file_info(&fi, &self.db))
                        .collect();
                    if directory_list.is_empty() {
                        self.error_message = Some(String::from("Directory is empty"));
                    } else {
                        let new_frame = BrowserFrame::Directory(DirectoryFrame {
                            current_item: 0,
                            directory_list,
                            cwd: requested_item.path.clone(),
                        });
                        self.frames.push(new_frame);
                    }
                } else {
                    self.error_message = Some(String::from("Item is not a directory"));
                }
            }
            None => panic!("Missing frame. This shouldn't happen."),
        }
    }

    fn enter_parent(&mut self) {
        match self.frames.last_mut() {
            Some(BrowserFrame::Projects(projects)) => {
                let view = projects.get_mut_view();
                let initial_path = &view.results[view.current_item].path;
                if let Some(parent_path) = initial_path.parent() {
                    let requested_path = parent_path.to_path_buf();
                    let new_frame = BrowserFrame::Directory(DirectoryFrame {
                        current_item: 0,
                        directory_list: self
                            .db
                            .iter_level(&requested_path)
                            .map(|fi| DirItem::from_file_info(&fi, &self.db))
                            .collect(),
                        cwd: requested_path,
                    });
                    self.frames.push(new_frame);
                } else {
                    self.error_message = Some(String::from("Unable to detect parent directory."));
                }
            }
            Some(BrowserFrame::Directory(_)) => {
                self.error_message = Some(String::from(
                    "Parent traversal is implemented only for projects and tools views.",
                ));
            }
            None => panic!("Missing frame. This shouldn't happen."),
        }
    }

    fn go_back(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    fn select_tab(&mut self, tab: Tab) {
        self.frames.truncate(1);
        match self.frames.last_mut() {
            Some(BrowserFrame::Projects(projects)) => projects.current_view = tab,
            Some(BrowserFrame::Directory(_)) => {
                panic!("Wrong frame present on the first position. This shouldn't happen.")
            }
            None => panic!("Missing frame. This shouldn't happen."),
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let chunks = self.create_layout(frame.area());
        self.render_header(frame, chunks[0]);
        match self.frames.last() {
            Some(BrowserFrame::Projects(projects_frame)) => {
                self.render_projects(frame, chunks[1], projects_frame)
            }
            Some(BrowserFrame::Directory(directory_frame)) => {
                self.render_directory(frame, chunks[1], directory_frame)
            }
            None => panic!("Missing frame. This shouldn't happen."),
        }
        self.render_footer(frame, chunks[2]);
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
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" DDC Project Browser ")
            .title_style(Style::default().fg(Color::Cyan))
            .border_style(Style::default().fg(Color::Cyan));

        match self.frames.last() {
            Some(BrowserFrame::Projects(projects)) => {
                let titles = vec![
                    format!("Discovered Projects ({})", projects.projects.results.len()),
                    format!("Tooling results ({})", projects.tools.results.len()),
                ];
                let selected_tab_index = match projects.current_view {
                    Tab::Projects => 0,
                    Tab::Tools => 1,
                };
                let tabs = Tabs::new(titles)
                    .highlight_style(
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    )
                    .select(selected_tab_index)
                    .padding(" ", " ")
                    .style(Style::default().fg(Color::DarkGray))
                    .divider("|")
                    .block(block);
                frame.render_widget(tabs, area);
            }
            Some(BrowserFrame::Directory(directory)) => {
                let text = format!(" Browsing: {} ", directory.cwd.display());
                let header = Paragraph::new(text)
                    .block(block)
                    .style(Style::default().fg(Color::White));

                frame.render_widget(header, area);
            }
            None => panic!("Missing frame. This shouldn't happen."),
        };
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
                    "→",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Enter Project  "),
                Span::styled(
                    "r",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Refresh  "),
                Span::styled(
                    "q/Esc",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Quit"),
            ])];

            Paragraph::new(line)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Controls ")
                        .style(Style::default().fg(Color::Green)),
                )
                .style(Style::default().fg(Color::White))
        };

        frame.render_widget(footer, area);
    }

    fn render_projects(&self, frame: &mut Frame, area: Rect, discovery_frame: &ProjectsFrame) {
        let view = discovery_frame.get_view();
        let rows: Vec<Row> = view
            .results
            .iter()
            .map(|result| self.create_project_row(result))
            .collect();

        let (title, column_name) = match discovery_frame.current_view {
            Tab::Projects => (" Projects ", "Project"),
            Tab::Tools => (" Tools ", "Tool"),
        };

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
            Row::new(["", column_name, "Size", "Last update"]).style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
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
        table_state.select(Some(view.current_item));

        frame.render_stateful_widget(table, area, &mut table_state);
    }

    fn create_project_row<'a>(&self, result: &'a DiscoveryResult) -> Row<'a> {
        let icon = format!(
            "{} ",
            if let Some(lang) = result.lang {
                format!("{} ", lang)
            } else {
                String::from(" ")
            }
        );

        let size = format_size(result.size, DECIMAL);
        let size_color_code = match get_size_color_code(result.size) {
            ColorCode::None => Color::White,
            ColorCode::Low => Color::Green,
            ColorCode::Medium => Color::Yellow,
            ColorCode::High => Color::Red,
        };

        let last_update = result
            .last_update
            .map(|t| {
                DateTime::<Local>::from(t)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            })
            .unwrap_or_default();
        let last_update_color_code = match get_time_color_code(&self.now, &result.last_update) {
            ColorCode::None => Color::White,
            ColorCode::Low => Color::Green,
            ColorCode::Medium => Color::Yellow,
            ColorCode::High => Color::Red,
        };

        let path_line = match result.result_type {
            ResultType::Discovery => Line::from(vec![Span::raw(result.path.display().to_string())]),
            ResultType::Static(ref description) => Line::from(vec![
                Span::raw(description),
                Span::styled(
                    format!(" ({})", result.path.display()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]),
        };

        Row::new(vec![
            Cell::from(icon),
            Cell::from(path_line),
            Cell::from(size).style(Style::default().fg(size_color_code)),
            Cell::from(last_update).style(Style::default().fg(last_update_color_code)),
        ])
        .style(Style::default().fg(Color::White))
    }

    fn render_directory(&self, frame: &mut Frame, area: Rect, directory_frame: &DirectoryFrame) {
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
