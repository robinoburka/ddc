#![feature(btree_cursors)]

use std::fs;

use anyhow::Context;
use clap::Parser;
use home::home_dir;
use tracing::{debug, error};

use crate::cli::{Args, COMMAND_NAME};
use crate::config::Config;
use crate::discovery::DiscoveryManager;
use crate::display::print_results;
use crate::logging::{LoggingLevel, setup_logging};

mod cli;
mod config;
mod discovery;
mod display;
mod file_info;
mod files_db;
mod loader;
mod logging;

const CONFIG_DIR: &str = ".config";
const CONFIG_FILE_NAME: &str = "ddc.toml";

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(
        "Configuration file not found. If this is the first run, call '{} generate-config' command first.",
        COMMAND_NAME
    )]
    ConfigurationFileNotFound,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    setup_logging(LoggingLevel::from(args.verbosity)).context("Failed to set up logging")?;

    let home_dir = home_dir().context("Couldn't identify your home directory.")?;
    debug!("Home directory resolved as: {}", &home_dir.display());

    let cfg_path = home_dir.join(CONFIG_DIR).join(CONFIG_FILE_NAME);
    debug!("Looking for a configuration file: {}", cfg_path.display());
    if !cfg_path.exists() {
        error!("Configuration file not found: {}", cfg_path.display());
        Err(AppError::ConfigurationFileNotFound)?;
    }

    let cfg_data = fs::read_to_string(&cfg_path).context("Couldn't read a configuration file.")?;
    let config: Config =
        toml::from_str(cfg_data.as_str()).context("Couldn't parse a configuration file.")?;

    let definitions = DiscoveryManager::with_default_loader(&home_dir)
        .add_from_config(&config)
        .collect();

    print_results(definitions);

    Ok(())
}
