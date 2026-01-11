use std::path::{Path, PathBuf};
use std::{fs, io};

use crossbeam::sync::WaitGroup;
use tracing::{debug, error};

use crate::browse_tui::App;
use crate::cli::BrowseArgs;
use crate::cli::COMMAND_NAME;
use crate::config::{Config, get_config_file_candidates};
use crate::discovery::{DiscoveryManager, DiscoveryResult};
use crate::display::display_progress_bar;
use crate::files_db::FilesDB;

#[derive(thiserror::Error, Debug)]
pub enum BrowseError {
    #[error(
        "Configuration file not found. If this is the first run, call '{} generate-config' command first.",
        COMMAND_NAME
    )]
    ConfigurationFileNotFound,
    #[error("Configuration file can't be loaded: {inner}")]
    CantLoadConfigurationFile {
        #[from]
        inner: std::io::Error,
    },
    #[error("Wrong configuration file format: {inner}")]
    CannotParseConfigurationFile {
        #[from]
        inner: toml::de::Error,
    },
    #[error("No results found. Do you use one of the supported languages?")]
    NoResultsFound,
    #[error("Unable to get DB from Discovery Manager. Programmer error?")]
    ProgrammerError,
    #[error("Unable to setup TUI application: {inner}")]
    UiError { inner: std::io::Error },
}

pub fn browse(_args: BrowseArgs, home_dir: &Path) -> Result<(), BrowseError> {
    let candidates = get_config_file_candidates(home_dir);
    let Some(cfg_path) = find_config_file(&candidates) else {
        error!("Configuration file not found");
        Err(BrowseError::ConfigurationFileNotFound)?
    };

    debug!("Using configuration file: {}", cfg_path.display());
    let cfg_data = fs::read_to_string(&cfg_path)?;
    let config: Config = toml::from_str(cfg_data.as_str())?;

    let discovery_manager =
        DiscoveryManager::with_default_loader(home_dir).add_from_config(&config);

    let wg = WaitGroup::new();
    let pg_worker = wg.clone();
    let progress_channel = discovery_manager.subscribe();
    rayon::spawn(move || {
        display_progress_bar(progress_channel);
        drop(pg_worker);
    });

    let (discovery_results, db) = discovery_manager.collect_and_get_db();
    let db = db.ok_or(BrowseError::ProgrammerError)?;
    wg.wait();

    if discovery_results.is_empty() {
        error!("No results found.");
        return Err(BrowseError::NoResultsFound);
    }

    start_tui(db, discovery_results).map_err(|e| BrowseError::UiError { inner: e })
}

fn find_config_file(candidates: &[PathBuf]) -> Option<PathBuf> {
    for candidate in candidates {
        debug!("Looking for a configuration file: {}", candidate.display());
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }

    None
}

fn start_tui(db: FilesDB, discovery_results: Vec<DiscoveryResult>) -> io::Result<()> {
    ratatui::run(|terminal| App::new(db, discovery_results).run(terminal))
}
