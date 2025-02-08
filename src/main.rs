#![feature(btree_cursors)]

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use home;
use humansize::{format_size, DECIMAL};
use tabled::settings::{Panel, Style};
use tabled::{Table, Tabled};
use toml;

use crate::config::Config;
use crate::discovery::{DetectedResult, DiscoveryDefinition, DiscoveryManager};

mod config;
mod discovery;
mod file_info;
mod files_db;
mod loader;

fn main() -> Result<()> {
    let cfg = PathBuf::from("ddc.toml");
    let cfg_data = fs::read_to_string(&cfg)?;
    let config: Config = toml::from_str(cfg_data.as_str())?;
    println!("{:#?}", config);

    let home_dir = home::home_dir().context("Couldn't identify your home directory.")?;

    let definitions = DiscoveryManager::with_default_loader(&home_dir)
        .add_from_config(&config)
        .collect();

    print_results(definitions);

    Ok(())
}

fn print_results(definitions: Vec<DiscoveryDefinition>) {
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
        .to_string();
    println!("{table_static}");

    let table_discovery = Table::new(discovery_data)
        .with(Panel::header("Projects"))
        .with(Style::modern_rounded())
        .to_string();
    println!("{table_discovery}");
}

#[derive(Tabled)]
struct Record {
    #[tabled(rename = "Language")]
    lang: String,
    #[tabled(rename = "Last change", display("tabled::derive::display::option", ""))]
    time: Option<String>,
    #[tabled(rename = "Path")]
    path: String,
    #[tabled(rename = "Size")]
    human_size: String,
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
        }
    }
}

#[derive(Tabled)]
struct StaticRecord {
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(inline)]
    record: Record,
}
