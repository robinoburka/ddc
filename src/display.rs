use chrono::{DateTime, Local};
use humansize::{format_size, DECIMAL};
use tabled::settings::{object::Cell, Color, Modify, Panel, Style};
use tabled::{Table, Tabled};

use crate::discovery::{DetectedResult, DiscoveryDefinition};

pub fn print_results(definitions: Vec<DiscoveryDefinition>) {
    let mut discovery_data = vec![];
    let mut static_data = vec![];

    for def in definitions {
        if def.results.len() == 0 {
            continue;
        }
        if def.discovery {
            discovery_data.extend(def.results.iter().map(Record::from));
        } else {
            static_data.extend(def.results.iter().map(|d| StaticRecord {
                description: def.description.clone(),
                record: Record::from(d),
            }));
        }
    }

    let table_static = Table::new(static_data)
        .with(Panel::header("Tooling"))
        .with(Style::modern_rounded())
        .with(Modify::new(Cell::new(0, 0)).with(Color::BOLD))
        .to_string();
    println!("{table_static}");

    let mut table_discovery_build = Table::new(&discovery_data);
    table_discovery_build.with(Panel::header("Projects"));
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
    #[tabled(rename = "Language")]
    lang: String,
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
            lang: value.lang.to_string(),
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
