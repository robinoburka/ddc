use std::path::Path;

use crate::discovery::DiscoveryDefinition;
use crate::types::Language;

pub fn default_discovery_definitions(home: &Path) -> Vec<DiscoveryDefinition> {
    let mut definitions = vec![
        // Rust
        ////////////////////////////////////////
        // Cargo registry
        DiscoveryDefinition {
            lang: Some(Language::Rust),
            discovery: false,
            description: "Cargo registry".into(),
            path: ".cargo/registry".into(),
        },
        // Python
        ////////////////////////////////////////
        // Poetry on macOS - cache
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: false,
            description: "Poetry cache".into(),
            path: "Library/Caches/pypoetry".into(),
        },
        // Poetry on macOS - default virtualenvs
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: true,
            description: "Poetry virtualenvs".into(),
            path: "Library/Caches/pypoetry/virtualenvs".into(),
        },
        // Poetry on Unix - cache
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: false,
            description: "Poetry cache".into(),
            path: ".cache/pypoetry".into(),
        },
        // Poetry on Unix - default virtualenvs
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: true,
            description: "Poetry virtualenvs".into(),
            path: ".cache/pypoetry/virtualenvs".into(),
        },
        // uv on Linux/macOS - cache
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: false,
            description: "uv cache".into(),
            path: ".cache/uv".into(),
        },
    ];

    for def in definitions.iter_mut() {
        def.path = home.join(&def.path)
    }

    definitions
}
