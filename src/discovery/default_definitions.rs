use crate::discovery::DiscoveryDefinition;
use crate::types::Language;

pub fn default_discovery_definitions() -> Vec<DiscoveryDefinition> {
    vec![
        // Rust
        ////////////////////////////////////////
        // Cargo registry
        DiscoveryDefinition {
            lang: Some(Language::Rust),
            discovery: false,
            description: "Cargo registry".into(),
            path: ".cargo/registry".into(),
            results: vec![],
        },
        // Python
        ////////////////////////////////////////
        // Poetry on macOS - cache
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: false,
            description: "Poetry cache".into(),
            path: "Library/Caches/pypoetry".into(),
            results: vec![],
        },
        // Poetry on macOS - default virtualenvs
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: true,
            description: "Poetry virtualenvs".into(),
            path: "Library/Caches/pypoetry/virtualenvs".into(),
            results: vec![],
        },
        // Poetry on Unix - cache
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: false,
            description: "Poetry cache".into(),
            path: ".cache/pypoetry".into(),
            results: vec![],
        },
        // Poetry on Unix - default virtualenvs
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: true,
            description: "Poetry virtualenvs".into(),
            path: ".cache/pypoetry/virtualenvs".into(),
            results: vec![],
        },
        // uv on Linux/macOS - cache
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: false,
            description: "uv cache".into(),
            path: ".cache/uv".into(),
            results: vec![],
        },
    ]
}
