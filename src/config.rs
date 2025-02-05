use std::path::PathBuf;

use serde::Deserialize;

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
