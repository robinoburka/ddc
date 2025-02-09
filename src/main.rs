#![feature(btree_cursors)]

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use home;
use toml;

use crate::config::Config;
use crate::discovery::DiscoveryManager;
use crate::display::print_results;

mod config;
mod discovery;
mod display;
mod file_info;
mod files_db;
mod loader;

fn main() -> Result<()> {
    let cfg = PathBuf::from("ddc.toml");
    let cfg_data = fs::read_to_string(&cfg).context("Couldn't read a configuration file.")?;
    let config: Config =
        toml::from_str(cfg_data.as_str()).context("Couldn't parse a configuration file.")?;

    let home_dir = home::home_dir().context("Couldn't identify your home directory.")?;

    let definitions = DiscoveryManager::with_default_loader(&home_dir)
        .add_from_config(&config)
        .collect();

    print_results(definitions);

    Ok(())
}
