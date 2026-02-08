use std::path::Path;

use crate::discovery::Language;
use crate::discovery::discovery_definitions::DiscoveryDefinition;

const CARGO_REGISTRY_INFO: &str = r#"It's considered to be safe to delete the whole `.cargo/registry` directory.

Cargo will just need to download data to during the next compiles.

Alternatively, you can check `cargo-cache` binary crate."#;

const UV_PYTHON_INSTALLATIONS_INFO: &str = r#"Use `uv python list` to inspect installed Python version.

Then you can remove unnecessary instalations with: `uv python uninstall VERSION`."#;
const UV_CACHE_INFO: &str = r#"Use `uv cache clean` to remove all cache entries.

Alternatively you can use `uv cache prune` to remove just outdated records. E.g. entries from previous uv versions."#;
const POETRY_CACHE_INFO: &str =
    r#"Use `poetry cache list` and then `poetry cache clear [--all] CACHE_NAME`"#;

pub fn default_discovery_definitions(home: &Path) -> Vec<DiscoveryDefinition> {
    let mut definitions = vec![
        // Rust
        ////////////////////////////////////////
        // Cargo registry
        DiscoveryDefinition {
            lang: Language::Rust,
            discovery: false,
            description: "Cargo registry",
            path: ".cargo/registry".into(),
            info: Some(CARGO_REGISTRY_INFO),
        },
        // Python
        ////////////////////////////////////////
        // Poetry on macOS - cache
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: false,
            description: "Poetry cache",
            path: "Library/Caches/pypoetry".into(),
            info: Some(POETRY_CACHE_INFO),
        },
        // Poetry on macOS - default virtualenvs
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: true,
            description: "Poetry virtualenvs",
            path: "Library/Caches/pypoetry/virtualenvs".into(),
            info: None,
        },
        // Poetry on Unix - cache
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: false,
            description: "Poetry cache",
            path: ".cache/pypoetry".into(),
            info: Some(POETRY_CACHE_INFO),
        },
        // Poetry on Unix - default virtualenvs
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: true,
            description: "Poetry virtualenvs",
            path: ".cache/pypoetry/virtualenvs".into(),
            info: None,
        },
        // uv on Linux/macOS - cache
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: false,
            description: "uv cache",
            path: ".cache/uv".into(),
            info: Some(UV_CACHE_INFO),
        },
        // uv alternative Python installation directory
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: false,
            description: "uv Python downloads",
            path: ".local/share/uv/python".into(),
            info: Some(UV_PYTHON_INSTALLATIONS_INFO),
        },
        // JavaScript
        ////////////////////////////////////////
        // npm - cache
        DiscoveryDefinition {
            lang: Language::JS,
            discovery: false,
            description: "NPM cache",
            path: ".npm".into(),
            info: None,
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
