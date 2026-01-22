use std::path::PathBuf;

use crate::discovery::Language;

#[derive(Debug)]
pub(super) enum DiscoveryDefinitionType {
    BuildIn(DiscoveryDefinition),
    External(ExternalDiscoveryDefinition),
}

#[derive(Debug, Default)]
pub struct DiscoveryDefinition {
    pub path: PathBuf,
    pub discovery: bool,
    pub description: String,
    pub lang: Language,
    pub info: Option<String>,
}

#[derive(Debug)]
pub struct ExternalDiscoveryDefinition {
    pub path: PathBuf,
}
