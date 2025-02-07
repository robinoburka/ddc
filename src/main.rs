#![feature(btree_cursors)]

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use home;
use humansize::{format_size, DECIMAL};
use toml;

use crate::config::Config;
use crate::discovery::DiscoveryManager;

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
        println!("{}", def.description);
        println!("==============");
        for res in def.results {
            let formatted_time = res
                .last_update
                .map(|t| {
                    DateTime::<Local>::from(t)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
                .unwrap_or("".into());

            println!(
                "{:?} ({}) {}: {}",
                res.lang,
                formatted_time,
                res.path.display(),
                format_size(res.size, DECIMAL)
            );
        }
        println!();
    }

    // let scan_paths = discovery
    //     .iter()
    //     .map(|pd| &pd.path)
    //     .map(|p| {
    //         if p.is_relative() {
    //             home_dir.join(p)
    //         } else {
    //             p.clone()
    //         }
    //     })
    //     .collect::<Vec<_>>();
    // println!("{:#?}", scan_paths);

    // let db = load_multiple_paths(&scan_paths)?;
    // println!("Collected {} results", db.len());
    //
    // for path in scan_paths.iter() {
    //     let size: u64 = db.iter_dir(path).filter_map(|fi| fi.size).sum();
    //     println!("{}: {}", path.display(), format_size(size, DECIMAL));
    // }

    Ok(())
}
