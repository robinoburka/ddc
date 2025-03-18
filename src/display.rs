use chrono::{DateTime, Local};
use humansize::{DECIMAL, format_size};
use tabled::settings::object::Rows;
use tabled::settings::{Alignment, Color, Modify, Panel, Style, object::Cell};
use tabled::{Table, Tabled};
use tracing::instrument;

use crate::discovery::{DetectedResult, DiscoveryDefinition};

#[instrument(level = "debug", skip(definitions))]
pub fn print_results(definitions: Vec<DiscoveryDefinition>) {
    let mut discovery_data = vec![];
    let mut static_data = vec![];

    let mut discovery_sum: u64 = 0;
    let mut static_sum: u64 = 0;

    for def in definitions {
        if def.results.is_empty() {
            continue;
        }
        if def.discovery {
            discovery_sum += def.results.iter().map(|r| r.size).sum::<u64>();
            discovery_data.extend(def.results.iter().map(Record::from));
        } else {
            static_sum += def.results.iter().map(|r| r.size).sum::<u64>();
            static_data.extend(def.results.iter().map(|d| StaticRecord {
                description: def.description.clone(),
                record: Record::from(d),
            }));
        }
    }

    let table_static = Table::new(static_data)
        .with(Panel::header("Tooling"))
        .with(Panel::footer(format_size(static_sum, DECIMAL)))
        .with(Modify::new(Rows::last()).with(Color::BOLD))
        .with(Modify::new(Rows::last()).with(Alignment::right()))
        .with(Style::modern_rounded())
        .with(Modify::new(Cell::new(0, 0)).with(Color::BOLD))
        .to_string();
    println!("{table_static}");

    let mut table_discovery_build = Table::new(&discovery_data);
    table_discovery_build.with(Panel::header("Projects"));
    table_discovery_build.with(Panel::footer(format_size(discovery_sum, DECIMAL)));
    table_discovery_build.with(Modify::new(Rows::last()).with(Color::BOLD));
    table_discovery_build.with(Modify::new(Rows::last()).with(Alignment::right()));
    table_discovery_build.with(Modify::new(Cell::new(0, 0)).with(Color::BOLD));
    table_discovery_build.with(Style::modern_rounded());
    discovery_data.iter().enumerate().for_each(|(i, d)| {
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
    time: Option<String>,
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

impl From<&DetectedResult> for Record {
    fn from(value: &DetectedResult) -> Self {
        Self {
            lang: value.lang.map(|l| l.to_string()),
            time: value.last_update.map(|t| {
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
