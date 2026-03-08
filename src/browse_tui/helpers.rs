use std::sync::OnceLock;
use std::time::SystemTime;

use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Color, Line, Modifier, Span, Style};
use ratatui::widgets::Cell;

use crate::display_tools::{ColorCode, get_size_color_code, get_time_color_code};

static NOW: OnceLock<SystemTime> = OnceLock::new();

pub fn now() -> SystemTime {
    *NOW.get_or_init(SystemTime::now)
}

pub fn popup_area_clamped(
    area: Rect,
    min_width: u16,
    max_width: u16,
    width_percent: u16,
    min_height: u16,
    max_height: u16,
    height_percent: u16,
) -> Rect {
    fn clamp_percent(total: u16, percent: u16, min: u16, max: u16) -> u16 {
        if total == 0 {
            return 0;
        }

        let percent_size = total.saturating_mul(percent) / 100;
        let clamped = percent_size.clamp(min, max);
        clamped.min(total)
    }

    let width = clamp_percent(area.width, width_percent, min_width, max_width);
    let height = clamp_percent(area.height, height_percent, min_height, max_height);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .split(vertical[1]);

    horizontal[1]
}

pub fn size_cell(size: u64) -> Cell<'static> {
    let text = format_size(size, DECIMAL);
    Cell::from(text).style(size_cell_style(size))
}

fn size_cell_style(size: u64) -> Style {
    let color = match get_size_color_code(size) {
        ColorCode::None => Color::Gray,
        ColorCode::Low => Color::Green,
        ColorCode::Medium => Color::Yellow,
        ColorCode::High => Color::Red,
    };

    Style::default().fg(color)
}

pub fn dimmed_size_cell(size: u64) -> Cell<'static> {
    let text = format_size(size, DECIMAL);
    Cell::from(text).style(size_cell_style(size).add_modifier(Modifier::DIM))
}

pub fn last_update_cell(now: SystemTime, last: Option<SystemTime>) -> Cell<'static> {
    let text = last
        .map(|t| {
            DateTime::<Local>::from(t)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_default();

    let color = match get_time_color_code(&now, &last) {
        ColorCode::None => Color::Gray,
        ColorCode::Low => Color::Green,
        ColorCode::Medium => Color::Yellow,
        ColorCode::High => Color::Red,
    };

    Cell::from(text).style(Style::default().fg(color))
}

pub fn percent_bar(width: usize, percent: f64) -> Line<'static> {
    let filled_len = ((width as f64) * percent / 100.0).round() as usize;

    let mut spans = Vec::new();

    for _ in 0..filled_len {
        spans.push(Span::from("█"));
    }
    for _ in filled_len..width {
        spans.push(Span::from("░"));
    }

    Line::from(spans)
}
