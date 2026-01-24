use std::io;
use std::path::Path;

use crossbeam::sync::WaitGroup;
use tracing::error;

use crate::browse_tui::App;
use crate::cli::{BrowseArgs, UiConfig};
use crate::config::{ConfigError, load_config_file};
use crate::discovery::DiscoveryResults;
use crate::discovery::{DiscoveryManager, ExternalDiscoveryDefinition};
use crate::display::display_progress_bar;

#[derive(thiserror::Error, Debug)]
pub enum BrowseError {
    #[error("Unable to load configuration file. See details for more information: {inner}")]
    ConfigError {
        #[from]
        inner: ConfigError,
    },
    #[error("No results found. Do you use one of the supported languages?")]
    NoResultsFound,
    #[error(
        "Unable to get DB from Discovery Manager. Programmer error? Try to rerun in production version."
    )]
    ProgrammerError,
    #[error("Unable to setup TUI application: {inner}")]
    UiError {
        #[from]
        inner: std::io::Error,
    },
}

pub fn browse(
    cmd_args: &BrowseArgs,
    ui_config: &UiConfig,
    home_dir: &Path,
) -> Result<(), BrowseError> {
    let config = load_config_file(home_dir, cmd_args.shared.config.as_deref())?;
    let definitions = config
        .paths
        .into_iter()
        .map(|p| ExternalDiscoveryDefinition { path: p.path })
        .collect::<Vec<_>>();

    let discovery_manager = DiscoveryManager::new(home_dir).add_definitions(&definitions);

    let wg = WaitGroup::new();
    if ui_config.show_progress {
        let pg_worker = wg.clone();
        let progress_channel = discovery_manager.subscribe();
        rayon::spawn(move || {
            display_progress_bar(progress_channel);
            drop(pg_worker);
        });
    }

    let discovery_results = discovery_manager.collect();
    // Don't remove the following check, or rewrite .unwrap() lines in start_tui
    if discovery_results.db.is_none() {
        return Err(BrowseError::ProgrammerError);
    }

    wg.wait();

    if discovery_results.projects.is_empty() && discovery_results.tools.is_empty() {
        error!("No results found.");
        return Err(BrowseError::NoResultsFound);
    }

    start_tui(discovery_results)?;

    Ok(())
}

fn start_tui(discovery_results: DiscoveryResults) -> io::Result<()> {
    ratatui::run(|terminal| {
        App::new(
            discovery_results.projects,
            discovery_results.tools,
            // Already checked in browse()
            discovery_results.db.unwrap(),
        )
        .run(terminal)
    })
}
