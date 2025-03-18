use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PathDefinition {
    pub path: PathBuf,
    #[serde(default)]
    pub discovery: bool,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub paths: Vec<PathDefinition>,
}

pub fn get_config_file_candidates(home_dir: &Path) -> Vec<PathBuf> {
    vec![
        home_dir.join(".config").join("ddc.toml"),
        home_dir.join(".ddc.toml"),
        PathBuf::from("ddc.toml"),
    ]
}
