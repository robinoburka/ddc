use crate::discovery::DiscoveryDefinition;
use crate::types::Language;

pub fn default_discovery_definitions() -> Vec<DiscoveryDefinition> {
    vec![
        DiscoveryDefinition {
            lang: Some(Language::Rust),
            discovery: false,
            description: "Cargo registry".into(),
            path: ".cargo/registry".into(),
            results: vec![],
        },
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: false,
            description: "Poetry cache".into(),
            path: "Library/Caches/pypoetry".into(),
            results: vec![],
        },
        DiscoveryDefinition {
            lang: Some(Language::Python),
            discovery: false,
            description: "uv cache".into(),
            path: ".cache/uv".into(),
            results: vec![],
        },
    ]
}
