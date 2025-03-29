use std::path::PathBuf;
use std::time::SystemTime;

use crate::types::Language;

#[derive(Debug)]
pub struct DetectedResult {
    pub lang: Option<Language>,
    pub path: PathBuf,
    pub size: u64,
    pub last_update: Option<SystemTime>,
}

#[derive(Debug)]
pub struct DiscoveryDefinition {
    pub path: PathBuf,
    pub discovery: bool,
    pub description: String,
    pub lang: Option<Language>,
    pub results: Vec<DetectedResult>,
}
