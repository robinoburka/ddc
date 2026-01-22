#![feature(btree_cursors)]
#![feature(duration_constructors)]

use anyhow::Context;
use clap::Parser;
use home::home_dir;
use tracing::{debug, debug_span};

use crate::analyze::{analyze, show_default_definitions};
use crate::browse::browse;
use crate::cli::{Args, Commands, UiConfig};
use crate::generate_config::generate_config;
use crate::logging::{LoggingLevel, setup_logging};

mod analyze;
mod browse;
mod browse_tui;
mod cli;
mod config;
mod discovery;
mod display;
mod display_tools;
mod file_info;
mod files_db;
mod generate_config;
mod loader;
mod logging;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    setup_logging(LoggingLevel::from(args.verbosity)).context("Failed to set up logging")?;
    {
        let _guard = debug_span!("creating_thread_pool").entered();
        rayon::ThreadPoolBuilder::new()
            .build_global()
            .context("Failed to create a thread pool")?;
    }

    let home_dir = home_dir().context("Couldn't identify your home directory.")?;
    debug!("Home directory resolved as: {}", &home_dir.display());

    let ui_config = UiConfig::from(&args);
    match args.command {
        Some(Commands::GenerateConfig) => generate_config(&home_dir)?,
        Some(Commands::ShowDefinitions) => show_default_definitions(&home_dir),
        Some(Commands::Analyze(_)) => analyze(&ui_config, &home_dir)?,
        Some(Commands::Browse(_)) => browse(&ui_config, &home_dir)?,
        None => analyze(&ui_config, &home_dir)?,
    };

    Ok(())
}
