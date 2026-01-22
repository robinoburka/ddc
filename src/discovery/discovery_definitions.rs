use std::path::PathBuf;

use crate::discovery::Language;

#[derive(Debug)]
pub(super) enum DiscoveryDefinitionType {
    BuildIn(DiscoveryDefinition),
    External(ExternalDiscoveryDefinition),
}

#[derive(Debug)]
pub struct DiscoveryDefinition {
    pub path: PathBuf,
    pub discovery: bool,
    pub description: String,
    pub lang: Option<Language>,
}

#[derive(Debug)]
pub struct ExternalDiscoveryDefinition {
    pub path: PathBuf,
}
