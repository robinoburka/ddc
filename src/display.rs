use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use tabled::settings::object::Rows;
use tabled::settings::{Alignment, Color, Modify, Panel, Style, object::Cell};
use tabled::{Table, Tabled};
use tracing::instrument;

use crate::discovery::{DiscoveryResult, ResultType};

#[instrument(level = "debug", skip(discovery_results))]
pub fn print_results(discovery_results: Vec<DiscoveryResult>) {
    let mut discovery_data = vec![];
    let mut static_data = vec![];

    let mut discovery_sum: u64 = 0;
    let mut static_sum: u64 = 0;

    for result in discovery_results {
        match result.result_type {
            ResultType::Discovery => {
                discovery_sum += result.size;
                discovery_data.push(Record::from(result))
            }
            ResultType::Static(ref description) => {
                static_sum += result.size;
                if result.size != 0 {
                    static_data.push(StaticRecord {
                        description: description.clone(),
                        record: Record::from(result),
                    });
                }
            }
        }
    }

    let now = SystemTime::now();

    let mut table_static_build = Table::new(&static_data);
    table_static_build.with(Panel::header("Tooling"));
    table_static_build.with(Panel::footer(format_size(static_sum, DECIMAL)));
    table_static_build.with(Modify::new(Rows::last()).with(Color::BOLD));
    table_static_build.with(Modify::new(Rows::last()).with(Alignment::right()));
    table_static_build.with(Modify::new(Cell::new(0, 0)).with(Color::BOLD));
    table_static_build.with(Style::modern_rounded());
    static_data.iter().enumerate().for_each(|(i, d)| {
        table_static_build
            .with(Modify::new(Cell::new(i + 2, 3)).with(time_color_coded(&now, &d.record.time)));
        table_static_build
            .with(Modify::new(Cell::new(i + 2, 4)).with(size_color_coded(d.record.size)));
    });
    let table_static = table_static_build.to_string();
    println!("{table_static}");

    let mut table_discovery_build = Table::new(&discovery_data);
    table_discovery_build.with(Panel::header("Projects"));
    table_discovery_build.with(Panel::footer(format_size(discovery_sum, DECIMAL)));
    table_discovery_build.with(Modify::new(Rows::last()).with(Color::BOLD));
    table_discovery_build.with(Modify::new(Rows::last()).with(Alignment::right()));
    table_discovery_build.with(Modify::new(Cell::new(0, 0)).with(Color::BOLD));
    table_discovery_build.with(Style::modern_rounded());
    discovery_data.iter().enumerate().for_each(|(i, d)| {
        table_discovery_build
            .with(Modify::new(Cell::new(i + 2, 2)).with(time_color_coded(&now, &d.time)));
        table_discovery_build.with(Modify::new(Cell::new(i + 2, 3)).with(size_color_coded(d.size)));
    });
    let table_discovery = table_discovery_build.to_string();
    println!("{table_discovery}");
}

#[derive(Tabled)]
struct Record {
    #[tabled(rename = "Language", display("tabled::derive::display::option", ""))]
    lang: Option<String>,
    #[tabled(rename = "Path")]
    path: String,
    #[tabled(rename = "Last change", display("tabled::derive::display::option", ""))]
    human_time: Option<String>,
    #[tabled(skip)]
    time: Option<SystemTime>,
    #[tabled(rename = "Size")]
    human_size: String,
    #[tabled(skip)]
    size: u64,
}

#[derive(Tabled)]
struct StaticRecord {
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(inline)]
    record: Record,
}

impl From<DiscoveryResult> for Record {
    fn from(value: DiscoveryResult) -> Self {
        Self {
            lang: value.lang.map(|l| l.to_string()),
            time: value.last_update,
            human_time: value.last_update.map(|t| {
                DateTime::<Local>::from(t)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            }),
            path: value.path.display().to_string(),
            human_size: format_size(value.size, DECIMAL),
            size: value.size,
        }
    }
}

fn size_color_coded(size: u64) -> Color {
    if size < 1000 * 1000 * 90 {
        Color::FG_GREEN
    } else if size < 1000 * 1000 * 900 {
        Color::FG_YELLOW
    } else {
        Color::FG_RED
    }
}

fn time_color_coded(now: &SystemTime, time: &Option<SystemTime>) -> Color {
    match time {
        None => Color::FG_WHITE, // Wouldn't be displayed anyway
        Some(system_time) => match now.duration_since(*system_time) {
            Err(_) => Color::FG_WHITE, // Future time; shouldn't happen
            Ok(duration) => {
                if duration < Duration::from_days(14) {
                    Color::FG_GREEN
                } else if duration < Duration::from_days(60) {
                    Color::FG_YELLOW
                } else {
                    Color::FG_RED
                }
            }
        },
    }
}
