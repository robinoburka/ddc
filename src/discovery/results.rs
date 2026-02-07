use std::path::PathBuf;
use std::time::SystemTime;

use crate::discovery::Language;
use crate::files_db::FilesDB;

#[derive(Debug)]
pub struct DiscoveryResults {
    pub projects: Vec<ProjectResult>,
    pub tools: Vec<ToolingResult>,
    pub vcs: Vec<VcsResult>,
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
    pub description: &'static str,
    pub path: PathBuf,
    pub lang: Language,
    pub size: u64,
    pub last_update: Option<SystemTime>,
    pub info: Option<&'static str>,
}

#[derive(Debug)]
pub struct VcsResult {
    pub path: PathBuf,
    pub size: u64,
    pub last_update: Option<SystemTime>,
    pub vcs_size: u64,
}

#[derive(Clone, Debug)]
pub struct ParentInfo {
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Debug)]
pub(super) enum DiscoveryResultEnvelop {
    Project(ProjectResult),
    Tool(ToolingResult),
    Vcs(VcsResult),
}
