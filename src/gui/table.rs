//! A sortable, filterable, selectable table. Every tab is an instance of it.

use std::path::Path;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use iced::widget::{Text, button, column, row, scrollable, text, text_input};
use iced::{Color, Element, Length, Task};

use super::list::{self, ListScroll, Nav};
use crate::display_tools::{ColorCode, get_size_color_code, get_time_color_code};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Project,
    Size,
    LastUpdate,
    DetectedProjects,
}

impl SortBy {
    pub fn label(self) -> &'static str {
        match self {
            SortBy::Project => "Project",
            SortBy::Size => "Size",
            SortBy::LastUpdate => "Last update",
            SortBy::DetectedProjects => "Detected projects",
        }
    }

    /// Biggest first for sizes and counts, the others from the smallest.
    fn default_descending(self) -> bool {
        matches!(self, SortBy::Size | SortBy::DetectedProjects)
    }
}

/// What the table needs to know about a row to filter and sort it.
pub trait Record {
    fn path(&self) -> &Path;
    fn size(&self) -> u64;
    fn last_update(&self) -> Option<SystemTime>;
    fn detected_projects(&self) -> usize {
        0
    }
}

pub struct Column<R> {
    pub title: &'static str,
    pub width: Length,
    /// Clicking the header sorts by this when set.
    pub sort: Option<SortBy>,
    pub cell: for<'a> fn(&'a R, SystemTime) -> Text<'a>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FilterChanged(String),
    SortBy(SortBy),
    Nav(Nav),
    Select(usize),
    /// Double click. The app selects the row, then enters it.
    Open(usize),
    Scrolled(scrollable::Viewport),
}

pub struct Table<R> {
    columns: Vec<Column<R>>,
    sort_options: &'static [SortBy],
    noun: &'static str,
    results: Vec<R>,
    /// Lowercased paths, so filtering doesn't allocate on every keystroke.
    filter_paths: Vec<String>,
    /// Indices into `results` that are currently visible, in display order.
    view: Vec<usize>,
    filter: String,
    sort: Option<(SortBy, bool)>,
    selected: Option<usize>,
    sum: u64,
    now: SystemTime,
    scroll: ListScroll,
}

impl<R: Record> Table<R> {
    pub fn new(
        columns: Vec<Column<R>>,
        sort_options: &'static [SortBy],
        noun: &'static str,
        results: Vec<R>,
    ) -> Self {
        let filter_paths = results
            .iter()
            .map(|r| r.path().to_string_lossy().to_lowercase())
            .collect();
        let mut table = Self {
            columns,
            sort_options,
            noun,
            results,
            filter_paths,
            view: Vec::new(),
            filter: String::new(),
            sort: None,
            selected: None,
            sum: 0,
            now: SystemTime::now(),
            scroll: ListScroll::new(),
        };
        table.refresh();
        table
    }

    pub fn len(&self) -> usize {
        self.results.len()
    }

    pub fn sort_options(&self) -> &'static [SortBy] {
        self.sort_options
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::FilterChanged(filter) => {
                self.set_filter(filter);
                self.reveal_selected()
            }
            Message::SortBy(sort_by) => {
                self.sort = match self.sort {
                    Some((current, desc)) if current == sort_by => Some((sort_by, !desc)),
                    _ => Some((sort_by, sort_by.default_descending())),
                };
                self.refresh();
                self.reveal_selected()
            }
            Message::Nav(nav) => {
                if let Some(selected) = self.selected {
                    let page = self.scroll.page_size();
                    self.selected = Some(list::navigate(nav, selected, self.view.len(), page));
                }
                self.reveal_selected()
            }
            Message::Select(pos) | Message::Open(pos) => {
                self.selected = Some(pos);
                Task::none()
            }
            Message::Scrolled(viewport) => {
                self.scroll.on_scrolled(viewport);
                Task::none()
            }
        }
    }

    pub fn set_filter(&mut self, filter: String) {
        self.filter = filter;
        self.refresh();
    }

    pub fn selected_record(&self) -> Option<&R> {
        self.selected
            .and_then(|pos| self.view.get(pos))
            .map(|&idx| &self.results[idx])
    }

    pub fn view(&self) -> Element<'_, Message> {
        let filter = text_input("Filter by path…", &self.filter)
            .on_input(Message::FilterChanged)
            .on_submit(Message::Open(self.selected.unwrap_or_default()))
            .padding(8);

        let header = row(self.columns.iter().map(|col| self.header_cell(col)))
            .spacing(8)
            .padding([0, 8]);

        let rows = column(
            self.view
                .iter()
                .enumerate()
                .map(|(pos, &idx)| self.row(pos, &self.results[idx])),
        );

        let footer = row![
            text(format!("{} {}", self.view.len(), self.noun)),
            text(format_size(self.sum, DECIMAL)).font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..iced::Font::DEFAULT
            }),
        ]
        .spacing(16);

        column![
            filter,
            header,
            scrollable(rows)
                .id(self.scroll.id())
                .on_scroll(Message::Scrolled)
                .height(Length::Fill)
                .width(Length::Fill),
            footer,
        ]
        .spacing(8)
        .into()
    }

    fn header_cell(&self, col: &Column<R>) -> Element<'_, Message> {
        let Some(sort_by) = col.sort else {
            return text(col.title).width(col.width).into();
        };
        let arrow = match self.sort {
            Some((s, false)) if s == sort_by => " ▲",
            Some((s, true)) if s == sort_by => " ▼",
            _ => "",
        };
        button(text(format!("{}{arrow}", col.title)))
            .on_press(Message::SortBy(sort_by))
            .padding(0)
            .width(col.width)
            .style(button::text)
            .into()
    }

    fn row<'a>(&self, pos: usize, record: &'a R) -> Element<'a, Message> {
        let content = row(self
            .columns
            .iter()
            .map(|col| (col.cell)(record, self.now).width(col.width).into()))
        .spacing(8);

        list::row(
            content,
            self.selected == Some(pos),
            Message::Select(pos),
            Message::Open(pos),
        )
    }

    fn refresh(&mut self) {
        let needle = self.filter.trim().to_lowercase();
        self.view = self
            .filter_paths
            .iter()
            .enumerate()
            .filter_map(|(idx, path)| path.contains(&needle).then_some(idx))
            .collect();

        if let Some((sort_by, descending)) = self.sort {
            let results = &self.results;
            match sort_by {
                SortBy::Project => self
                    .view
                    .sort_by(|&a, &b| results[a].path().cmp(results[b].path())),
                SortBy::Size => self.view.sort_by_key(|&i| results[i].size()),
                SortBy::LastUpdate => self.view.sort_by_key(|&i| results[i].last_update()),
                SortBy::DetectedProjects => {
                    self.view.sort_by_key(|&i| results[i].detected_projects())
                }
            }
            if descending {
                self.view.reverse();
            }
        }

        self.sum = self.view.iter().map(|&i| self.results[i].size()).sum();
        self.selected = match (self.view.len(), self.selected) {
            (0, _) => None,
            (len, Some(s)) => Some(s.min(len - 1)),
            (_, None) => Some(0),
        };
    }

    fn reveal_selected(&mut self) -> Task<Message> {
        match self.selected {
            Some(selected) => self.scroll.reveal(selected),
            None => Task::none(),
        }
    }
}

// Cell helpers, shared by the tab definitions and the directory browser

pub fn code_color(code: ColorCode) -> Color {
    match code {
        ColorCode::None => Color::from_rgb(0.7, 0.7, 0.7),
        ColorCode::Low => Color::from_rgb(0.4, 0.8, 0.4),
        ColorCode::Medium => Color::from_rgb(0.9, 0.8, 0.3),
        ColorCode::High => Color::from_rgb(0.9, 0.4, 0.4),
    }
}

pub fn size_color(size: u64) -> Color {
    code_color(get_size_color_code(size))
}

pub fn time_color(now: &SystemTime, last: &Option<SystemTime>) -> Color {
    code_color(get_time_color_code(now, last))
}

pub fn size_text<'a>(size: u64) -> Text<'a> {
    text(format_size(size, DECIMAL)).color(size_color(size))
}

pub fn dimmed_size_text<'a>(size: u64) -> Text<'a> {
    text(format_size(size, DECIMAL)).color(Color {
        a: 0.6,
        ..size_color(size)
    })
}

pub fn last_update_text<'a>(now: SystemTime, last: Option<SystemTime>) -> Text<'a> {
    let formatted = last
        .map(|t| {
            DateTime::<Local>::from(t)
                .format("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_default();
    text(formatted).color(time_color(&now, &last))
}
