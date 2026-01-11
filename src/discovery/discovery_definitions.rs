use std::path::PathBuf;
use std::time::SystemTime;

use crate::types::Language;

#[derive(Clone, Debug)]
pub struct DiscoveryResult {
    pub result_type: ResultType,
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

#[derive(Clone, Debug, PartialEq)]
pub enum ResultType {
    Discovery,
    Static(String),
}

#[derive(Debug)]
pub struct DiscoveryDefinition {
    pub path: PathBuf,
    pub discovery: bool,
    pub description: String,
    pub lang: Option<Language>,
}
