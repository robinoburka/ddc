use std::path::PathBuf;
use std::time::SystemTime;

use crate::files_db::FilesDB;
use crate::types::Language;

#[derive(Debug)]
pub struct DiscoveryResults {
    pub projects: Vec<ProjectResult>,
    pub tools: Vec<ToolingResult>,
    pub db: Option<FilesDB>,
}

#[derive(Debug)]
pub struct ProjectResult {
    pub path: PathBuf,
    pub lang: Language,
    pub size: u64,
    pub last_update: Option<SystemTime>,
    pub parent: Option<ParentInfo>,
}

#[derive(Debug)]
pub struct ToolingResult {
    pub description: String,
    pub path: PathBuf,
    pub lang: Option<Language>,
    pub size: u64,
    pub last_update: Option<SystemTime>,
    pub parent: Option<ParentInfo>,
}

#[derive(Clone, Debug)]
pub struct ParentInfo {
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Debug)]
pub struct DiscoveryDefinition {
    pub path: PathBuf,
    pub discovery: bool,
    pub description: String,
    pub lang: Option<Language>,
}
