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
use crate::discovery::{DetectedResult, DiscoveryManager};

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

    for def in definitions {
        if def.results.len() == 0 {
            continue;
        }
        let data = def.results.iter().map(Record::from).collect::<Vec<_>>();
        let table = Table::new(data)
            .with(Panel::header(def.description))
            .with(Style::modern_rounded())
            .to_string();
        println!("{table}");
    }

    Ok(())
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
