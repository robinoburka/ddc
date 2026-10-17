use std::path::PathBuf;
use std::rc::Rc;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use iced::widget::{column, progress_bar, row, scrollable, text};
use iced::{Color, Element, Length, Task};

use super::list::{self, ListScroll, Nav};
use super::table::{size_color, time_color};
use crate::file_info::FileInfo;
use crate::files_db::FilesDB;

#[derive(Debug, Clone)]
pub enum Message {
    Nav(Nav),
    Select(usize),
    /// Double click. The app selects the row, then enters it.
    Open(usize),
    Scrolled(scrollable::Viewport),
}

/// What the browser wants the app to do, beyond changing its own state.
#[derive(Debug)]
pub enum Event {
    /// The last directory was left.
    Close,
    Error(String),
}

pub struct DirectoryBrowser {
    db: Rc<FilesDB>,
    frames: Vec<Frame>,
    now: SystemTime,
}

struct Frame {
    cwd: PathBuf,
    items: Vec<DirItem>,
    sum: u64,
    selected: usize,
    scroll: ListScroll,
}

struct DirItem {
    name: String,
    path: PathBuf,
    is_directory: bool,
    size: Option<u64>,
    last_update: Option<SystemTime>,
}

impl DirectoryBrowser {
    pub fn new(db: Rc<FilesDB>, path: PathBuf) -> Result<Self, String> {
        let frame = open_path(&db, path)?;
        Ok(Self {
            db,
            frames: vec![frame],
            now: SystemTime::now(),
        })
    }

    pub fn current_path(&self) -> Option<&PathBuf> {
        self.frames.last().map(|f| &f.cwd)
    }

    pub fn update(&mut self, message: Message) -> (Task<Message>, Option<Event>) {
        let Some(frame) = self.frames.last_mut() else {
            return (Task::none(), Some(Event::Close));
        };
        match message {
            Message::Nav(nav) => {
                let page = frame.scroll.page_size();
                frame.selected = list::navigate(nav, frame.selected, frame.items.len(), page);
                (frame.scroll.reveal(frame.selected), None)
            }
            Message::Select(pos) | Message::Open(pos) => {
                frame.selected = pos;
                (Task::none(), None)
            }
            Message::Scrolled(viewport) => {
                frame.scroll.on_scrolled(viewport);
                (Task::none(), None)
            }
        }
    }

    /// Enters the selected directory.
    pub fn enter(&mut self) -> Option<Event> {
        let frame = self.frames.last()?;
        let Some(item) = frame.items.get(frame.selected) else {
            return Some(Event::Error(String::from("No item selected.")));
        };
        if !item.is_directory {
            return Some(Event::Error(String::from("Item is not a directory.")));
        }

        match open_path(&self.db, item.path.clone()) {
            Ok(frame) => {
                self.frames.push(frame);
                None
            }
            Err(msg) => Some(Event::Error(msg)),
        }
    }

    pub fn back(&mut self) -> Option<Event> {
        self.frames.pop();
        self.frames.is_empty().then_some(Event::Close)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let Some(frame) = self.frames.last() else {
            return text("").into();
        };

        let header = row![
            text("").width(Length::Fixed(40.0)),
            text("Item").width(Length::FillPortion(6)),
            text("").width(Length::Fixed(60.0)),
            text("").width(Length::Fixed(120.0)),
            text("Size").width(Length::Fixed(100.0)),
            text("Last modified").width(Length::Fixed(180.0)),
        ]
        .spacing(8)
        .padding([0, 8]);

        let rows = column(
            frame
                .items
                .iter()
                .enumerate()
                .map(|(pos, item)| self.row(frame, pos, item)),
        );

        column![
            header,
            scrollable(rows)
                .id(frame.scroll.id())
                .on_scroll(Message::Scrolled)
                .height(Length::Fill)
                .width(Length::Fill),
            text(format_size(frame.sum, DECIMAL)),
        ]
        .spacing(8)
        .into()
    }

    fn row<'a>(&self, frame: &Frame, pos: usize, item: &'a DirItem) -> Element<'a, Message> {
        let percent = match (item.size, frame.sum) {
            (Some(size), sum) if sum > 0 => size as f64 * 100.0 / sum as f64,
            _ => 0.0,
        };
        let name_color = item.is_directory.then_some(Color::from_rgb(0.4, 0.8, 0.9));
        let last_update = item
            .last_update
            .map(|t| {
                DateTime::<Local>::from(t)
                    .format("%Y-%m-%d %H:%M")
                    .to_string()
            })
            .unwrap_or_default();
        let size = match item.size {
            Some(size) => text(format_size(size, DECIMAL)).color(size_color(size)),
            None => text("?"),
        };

        let content = row![
            text(if item.is_directory { "📁" } else { "📄" }).width(Length::Fixed(40.0)),
            text(&item.name)
                .color_maybe(name_color)
                .width(Length::FillPortion(6)),
            text(format!("{percent:>5.1}%")).width(Length::Fixed(60.0)),
            progress_bar(0.0..=100.0, percent as f32)
                .length(Length::Fixed(120.0))
                .girth(10),
            size.width(Length::Fixed(100.0)),
            text(last_update)
                .color(time_color(&self.now, &item.last_update))
                .width(Length::Fixed(180.0)),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        list::row(
            content,
            frame.selected == pos,
            Message::Select(pos),
            Message::Open(pos),
        )
    }
}

fn open_path(db: &FilesDB, path: PathBuf) -> Result<Frame, String> {
    let items: Vec<_> = db
        .iter_level(&path)
        .map(|fi| DirItem::from_file_info(&fi, db))
        .collect();

    if items.is_empty() {
        return Err(String::from("Directory is empty."));
    }

    Ok(Frame {
        cwd: path,
        sum: items.iter().filter_map(|i| i.size).sum(),
        items,
        selected: 0,
        scroll: ListScroll::new(),
    })
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
