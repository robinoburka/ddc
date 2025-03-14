use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct CustomPathDefinition {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub discovery: bool,
}

#[derive(Debug, Deserialize)]
pub struct ProjectsPathDefinition {
    #[serde(default)]
    pub name: Option<String>,
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rust: Option<Vec<CustomPathDefinition>>,
    pub python: Option<Vec<CustomPathDefinition>>,
    pub projects: Vec<ProjectsPathDefinition>,
}

pub const CONFIG_DIR: &str = ".config";
pub const CONFIG_FILE_NAME: &str = "ddc.toml";
pub const HOME_CONFIG_FILE_NAME: &str = ".ddc.toml";

pub fn find_config_file(home_dir: &Path) -> Option<PathBuf> {
    let candidate = home_dir.join(CONFIG_DIR).join(CONFIG_FILE_NAME);
    debug!("Looking for a configuration file: {}", candidate.display());
    if candidate.exists() {
        return Some(candidate);
    }

    let candidate = home_dir.join(HOME_CONFIG_FILE_NAME);
    debug!("Looking for a configuration file: {}", candidate.display());
    if candidate.exists() {
        return Some(candidate);
    }

    let candidate = PathBuf::from(CONFIG_FILE_NAME);
    debug!("Looking for configuration file: {}", candidate.display());
    if candidate.exists() {
        return Some(candidate);
    }

    None
}
