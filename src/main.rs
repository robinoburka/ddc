#![feature(btree_cursors)]

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use home::home_dir;

use crate::config::Config;
use crate::discovery::DiscoveryManager;
use crate::display::print_results;
use crate::logging::{LoggingLevel, setup_logging};

mod config;
mod discovery;
mod display;
mod file_info;
mod files_db;
mod loader;
mod logging;

fn main() -> Result<()> {
    setup_logging(LoggingLevel::Traces).context("Failed to set up logging")?;

    let cfg = PathBuf::from("ddc.toml");
    let cfg_data = fs::read_to_string(&cfg).context("Couldn't read a configuration file.")?;
    let config: Config =
        toml::from_str(cfg_data.as_str()).context("Couldn't parse a configuration file.")?;

    let home_dir = home_dir().context("Couldn't identify your home directory.")?;

    let definitions = DiscoveryManager::with_default_loader(&home_dir)
        .add_from_config(&config)
        .collect();

    print_results(definitions);

    Ok(())
}
