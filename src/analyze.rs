use std::fs;
use std::path::{Path, PathBuf};

use owo_colors::OwoColorize;
use tracing::{debug, error};

use crate::cli::{AnalyzeArgs, COMMAND_NAME};
use crate::config::{Config, get_config_file_candidates};
use crate::discovery::{DiscoveryManager, default_discovery_definitions};
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
    #[error("No results found. Do you use one of the supported languages?")]
    NoResultsFound,
}
pub fn analyze(args: AnalyzeArgs, home_dir: &Path) -> Result<(), AnalyzeError> {
    if args.show_definitions {
        show_default_definitions(home_dir);
        return Ok(());
    }

    let candidates = get_config_file_candidates(home_dir);
    let Some(cfg_path) = find_config_file(&candidates) else {
        error!("Configuration file not found");
        Err(AnalyzeError::ConfigurationFileNotFound)?
    };

    debug!("Using configuration file: {}", cfg_path.display());
    let cfg_data = fs::read_to_string(&cfg_path)?;
    let config: Config = toml::from_str(cfg_data.as_str())?;

    let discovery_results = DiscoveryManager::with_default_loader(home_dir)
        .add_from_config(&config)
        .collect();

    if discovery_results.is_empty() {
        error!("No results found.");
        return Err(AnalyzeError::NoResultsFound);
    }
    print_results(discovery_results);

    Ok(())
}

fn show_default_definitions(home: &Path) {
    default_discovery_definitions(home)
        .iter()
        .for_each(|definition| {
            println!(
                "{} {} ({}): {}",
                definition.lang.map(|l| l.to_string()).unwrap_or_default(),
                definition.description.bold(),
                if definition.discovery { "🔭" } else { "🧰" },
                definition.path.display().dimmed()
            );
        })
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
