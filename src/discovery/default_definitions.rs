use std::path::Path;

use crate::discovery::Language;
use crate::discovery::discovery_definitions::DiscoveryDefinition;

pub fn default_discovery_definitions(home: &Path) -> Vec<DiscoveryDefinition> {
    let mut definitions = vec![
        // Rust
        ////////////////////////////////////////
        // Cargo registry
        DiscoveryDefinition {
            lang: Language::Rust,
            discovery: false,
            description: "Cargo registry".into(),
            path: ".cargo/registry".into(),
        },
        // Python
        ////////////////////////////////////////
        // Poetry on macOS - cache
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: false,
            description: "Poetry cache".into(),
            path: "Library/Caches/pypoetry".into(),
        },
        // Poetry on macOS - default virtualenvs
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: true,
            description: "Poetry virtualenvs".into(),
            path: "Library/Caches/pypoetry/virtualenvs".into(),
        },
        // Poetry on Unix - cache
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: false,
            description: "Poetry cache".into(),
            path: ".cache/pypoetry".into(),
        },
        // Poetry on Unix - default virtualenvs
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: true,
            description: "Poetry virtualenvs".into(),
            path: ".cache/pypoetry/virtualenvs".into(),
        },
        // uv on Linux/macOS - cache
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: false,
            description: "uv cache".into(),
            path: ".cache/uv".into(),
        },
        // uv alternative Python installation directory
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: false,
            description: "uv Python downloads".into(),
            path: ".local/share/uv/python".into(),
        },
        // JavaScript
        ////////////////////////////////////////
        // npm - cache
        DiscoveryDefinition {
            lang: Language::JS,
            discovery: false,
            description: "NPM cache".into(),
            path: ".npm".into(),
        },
    ];

    for def in definitions.iter_mut() {
        def.path = home.join(&def.path)
    }

    definitions
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_get_default_definitions_based_on_home_dir() {
        let home_path = PathBuf::from("/home/foo");
        let definitions = default_discovery_definitions(&home_path);

        for def in definitions {
            assert_eq!(def.path.starts_with("/home/foo"), true);
        }
    }
}
