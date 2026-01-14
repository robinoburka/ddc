use std::io;
use std::path::Path;

use crossbeam::sync::WaitGroup;
use tracing::error;

use crate::browse_tui::App;
use crate::cli::UiConfig;
use crate::config::{ConfigError, load_config_file};
use crate::discovery::{DiscoveryManager, DiscoveryResult};
use crate::display::display_progress_bar;
use crate::files_db::FilesDB;

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

pub fn browse(ui_config: &UiConfig, home_dir: &Path) -> Result<(), BrowseError> {
    let config = load_config_file(home_dir)?;

    let discovery_manager =
        DiscoveryManager::with_default_loader(home_dir).add_from_config(&config);

    let wg = WaitGroup::new();
    if ui_config.show_progress {
        let pg_worker = wg.clone();
        let progress_channel = discovery_manager.subscribe();
        rayon::spawn(move || {
            display_progress_bar(progress_channel);
            drop(pg_worker);
        });
    }

    let (discovery_results, db) = discovery_manager.collect_and_get_db();
    let db = db.ok_or(BrowseError::ProgrammerError)?;

    wg.wait();

    if discovery_results.is_empty() {
        error!("No results found.");
        return Err(BrowseError::NoResultsFound);
    }

    start_tui(db, discovery_results)?;

    Ok(())
}

fn start_tui(db: FilesDB, discovery_results: Vec<DiscoveryResult>) -> io::Result<()> {
    ratatui::run(|terminal| App::new(db, discovery_results).run(terminal))
}
