use std::fs;
use std::path::{Path, PathBuf};

use tracing::{debug, error};

use crate::cli::COMMAND_NAME;
use crate::config::{Config, get_config_file_candidates};
use crate::discovery::DiscoveryManager;
use crate::display::print_results;
#[derive(thiserror::Error, Debug)]
pub enum AnalyzeError {
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
}
pub fn analyze(home_dir: &Path) -> Result<(), AnalyzeError> {
    let candidates = get_config_file_candidates(home_dir);
    let Some(cfg_path) = _find_config_file(&candidates) else {
        error!("Configuration file not found");
        Err(AnalyzeError::ConfigurationFileNotFound)?
    };

    debug!("Using configuration file: {}", cfg_path.display());
    let cfg_data = fs::read_to_string(&cfg_path)?;
    let config: Config = toml::from_str(cfg_data.as_str())?;

    let definitions = DiscoveryManager::with_default_loader(home_dir)
        .add_from_config(&config)
        .collect();

    print_results(definitions);

    Ok(())
}

pub fn _find_config_file(candidates: &[PathBuf]) -> Option<PathBuf> {
    for candidate in candidates {
        debug!("Looking for a configuration file: {}", candidate.display());
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }

    None
}
