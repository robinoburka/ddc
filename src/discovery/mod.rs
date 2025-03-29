mod default_definitions;
mod detectors;
mod discovery_definitions;
mod discovery_manager;

pub use discovery_definitions::{DetectedResult, DiscoveryDefinition};
pub use discovery_manager::{DiscoveryManager, PathLoader};
