use std::path::PathBuf;
use std::time::SystemTime;

use humansize::{DECIMAL, format_size};
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
};
use ratatui::{Frame, crossterm::event::KeyCode};

use crate::browse_tui::component::{Component, Navigable};
use crate::browse_tui::helpers;
use crate::browse_tui::helpers::{last_update_cell, now, size_cell};
use crate::browse_tui::message::AppMessage;
use crate::file_info::FileInfo;
use crate::files_db::FilesDB;

#[derive(Debug)]
pub struct DirectoryBrowser {
    db: FilesDB,
    frames: Vec<DirectoryBrowserFrame>,
    page_size: u16,
}

impl DirectoryBrowser {
    pub fn new(db: FilesDB) -> Self {
        Self {
            db,
            frames: vec![],
            page_size: 0,
        }
    }

    pub fn is_clear(&mut self) -> bool {
        self.frames.is_empty()
    }

    pub fn clear(&mut self) {
        self.frames.clear();
    }

    pub fn get_current_path(&mut self) -> Option<PathBuf> {
        self.frames.last().map(|frame| frame.cwd.clone())
    }

    pub fn open_path(&mut self, path: PathBuf) -> Result<(), String> {
        let directory_list: Vec<_> = self
            .db
            .iter_level(&path)
            .map(|fi| DirItem::from_file_info(&fi, &self.db))
            .collect();

        if directory_list.is_empty() {
            return Err(String::from("Directory is empty."));
        }

        self.frames.push(DirectoryBrowserFrame {
            state: {
                let mut browser_sate = TableState::default();
                browser_sate.select(Some(0));
                browser_sate
            },
            scroll_state: ScrollbarState::new(directory_list.len()),
            cwd: path.clone(),
            sum: directory_list.iter().filter_map(|i| i.size).sum(),
            directory_list,
        });

        Ok(())
    }

    pub fn enter(&mut self) -> Option<AppMessage> {
        let path = if let Some(frame) = self.frames.last_mut() {
            let Some(item) = frame
                .state
                .selected()
                .and_then(|idx| frame.directory_list.get(idx))
            else {
                return Some(AppMessage::SetError(String::from("No item selected.")));
            };

            if !item.is_directory {
                return Some(AppMessage::SetError(String::from(
                    "Item is not a directory.",
                )));
            }

            Some(item.path.clone())
        } else {
            None
        };

        let Some(path) = path else {
            return Some(AppMessage::SetError(String::from("No item selected.")));
        };

        if let Err(msg) = self.open_path(path) {
            return Some(AppMessage::SetError(msg));
        }

        None
    }

    pub fn back(&mut self) {
        self.frames.pop();
    }
}

#[derive(Debug)]
pub enum DirectoryBrowserMessage {
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    Back,
}

impl Component for DirectoryBrowser {
    type Message = DirectoryBrowserMessage;

    fn update(&mut self, message: Self::Message) -> Option<AppMessage> {
        match message {
            DirectoryBrowserMessage::MoveUp => self.move_up(),
            DirectoryBrowserMessage::MoveDown => self.move_down(),
            DirectoryBrowserMessage::PageUp => self.page_up(),
            DirectoryBrowserMessage::PageDown => self.page_down(),
            DirectoryBrowserMessage::Home => self.home(),
            DirectoryBrowserMessage::End => self.end(),
            DirectoryBrowserMessage::Enter => {
                return self.enter();
            }
            DirectoryBrowserMessage::Back => self.back(),
        }
        None
    }

    fn handle_key(&mut self, key: KeyCode) -> Option<Self::Message> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => Some(DirectoryBrowserMessage::MoveUp),
            KeyCode::Down | KeyCode::Char('j') => Some(DirectoryBrowserMessage::MoveDown),
            KeyCode::Right | KeyCode::Char('l') => Some(DirectoryBrowserMessage::Enter),
            KeyCode::Left | KeyCode::Char('h') => Some(DirectoryBrowserMessage::Back),
            KeyCode::PageDown => Some(DirectoryBrowserMessage::PageDown),
            KeyCode::PageUp => Some(DirectoryBrowserMessage::PageUp),
            KeyCode::Home => Some(DirectoryBrowserMessage::Home),
            KeyCode::End => Some(DirectoryBrowserMessage::End),
            _ => None,
        }
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.page_size = area.height.saturating_sub(3);

        let Some(directory_frame) = self.frames.last_mut() else {
            return;
        };

        let directory_size: u64 = directory_frame
            .directory_list
            .iter()
            .filter_map(|di| di.size)
            .sum();
        let rows: Vec<_> = directory_frame
            .directory_list
            .iter()
            .map(|di| create_row(di, directory_size))
            .collect();
        let human_size = format_size(directory_frame.sum, DECIMAL);

        let table = Table::new(
            rows,
            &[
                Constraint::Length(3),
                Constraint::Percentage(60),
                Constraint::Length(6),
                Constraint::Length(20),
                Constraint::Length(10),
                Constraint::Length(20),
            ],
        )
        .header(
            Row::new(vec!["", "Item", "", "", "Size", "Last modified"]).style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        )
        .footer(Row::new(vec![
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
            Cell::from(human_size.as_str()).style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from(""),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Directory List ")
                .title_style(Style::default().fg(Color::LightYellow))
                .border_style(Style::default().fg(Color::LightYellow)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("► ");

        frame.render_stateful_widget(table, area, &mut directory_frame.state);

        let needs_scroll =
            directory_frame.directory_list.len() > area.height.saturating_sub(3) as usize;
        if needs_scroll {
            let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"));

            frame.render_stateful_widget(scrollbar, area, &mut directory_frame.scroll_state);
        }
    }
}

fn create_row<'a>(item: &'a DirItem, dir_size: u64) -> Row<'a> {
    let (icon, name_style) = if item.is_directory {
        ("📁", Style::default().fg(Color::Cyan))
    } else {
        ("📄", Style::default())
    };
    let size = item.size.map(size_cell).unwrap_or_else(|| Cell::from("?"));
    let percent = item
        .size
        .map(|size| {
            if dir_size == 0 {
                0.0
            } else {
                (size as f64) * 100.0 / (dir_size as f64)
            }
        })
        .unwrap_or_default();

    let bar = helpers::percent_bar(20, percent);

    Row::new(vec![
        Cell::from(icon.to_string()),
        Cell::from(Span::styled(&item.name, name_style)),
        Cell::from(Line::from(vec![
            Span::from(format!("{:>5.1}", percent)),
            Span::styled("%", Style::default().add_modifier(Modifier::DIM)),
        ])),
        Cell::from(bar),
        size,
        last_update_cell(now(), item.last_update),
    ])
}

impl Navigable for DirectoryBrowser {
    fn move_up(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.state.select_previous();
            frame.scroll_state.prev();
        }
    }

    fn move_down(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.state.select_next();
            frame.scroll_state.next();
        }
    }

    fn page_up(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.state.scroll_up_by(self.page_size);
            frame.scroll_state = frame.scroll_state.position(
                frame
                    .scroll_state
                    .get_position()
                    .saturating_sub(self.page_size as usize),
            );
        }
    }

    fn page_down(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.state.scroll_down_by(self.page_size);
            frame.scroll_state = frame.scroll_state.position(
                frame
                    .scroll_state
                    .get_position()
                    .saturating_add(self.page_size as usize),
            );
        }
    }

    fn home(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.state.select_first();
            frame.scroll_state.first();
        }
    }

    fn end(&mut self) {
        if let Some(frame) = self.frames.last_mut() {
            frame.state.select_last();
            frame.scroll_state.last();
        }
    }
}

#[derive(Debug)]
struct DirectoryBrowserFrame {
    state: TableState,
    scroll_state: ScrollbarState,
    cwd: PathBuf,
    directory_list: Vec<DirItem>,
    sum: u64,
}

#[derive(Debug, Clone)]
struct DirItem {
    name: String,
    path: PathBuf,
    is_directory: bool,
    size: Option<u64>,
    last_update: Option<SystemTime>,
}

impl DirItem {
    fn from_file_info(file_info: &FileInfo, db: &FilesDB) -> Self {
        Self {
            name: file_info
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string(),
            path: file_info.path.clone(),
            is_directory: file_info.is_dir,
            size: if file_info.is_dir {
                Some(
                    db.iter_dir(file_info.path)
                        .filter_map(|item| item.size)
                        .sum(),
                )
            } else {
                file_info.size
            },
            last_update: file_info.touched,
        }
    }
}
