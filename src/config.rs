use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct PathDefinition {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub paths: Vec<PathDefinition>,
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error(
        "Configuration file not found. If this is the first run, call the 'generate-config' command first."
    )]
    ConfigurationFileNotFound,
    #[error("Configuration file can't be loaded: {inner}")]
    CantLoadConfigurationFile {
        #[from]
        inner: std::io::Error,
    },
    #[error("Wrong configuration file format: {inner}")]
    CannotParseConfigurationFile {
        #[from]
        inner: toml::de::Error,
    },
}

pub fn load_config_file(home_dir: &Path) -> Result<Config, ConfigError> {
    let candidates = get_config_file_candidates(home_dir);
    let Some(cfg_path) = find_config_file(&candidates) else {
        error!("Configuration file not found");
        Err(ConfigError::ConfigurationFileNotFound)?
    };

    debug!("Using configuration file: {}", cfg_path.display());
    let cfg_data = fs::read_to_string(&cfg_path)?;
    let config: Config = toml::from_str(cfg_data.as_str())?;

    Ok(config)
}

fn find_config_file(candidates: &[PathBuf]) -> Option<PathBuf> {
    for candidate in candidates {
        debug!("Looking for a configuration file: {}", candidate.display());
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }

    None
}

pub fn get_config_file_candidates(home_dir: &Path) -> Vec<PathBuf> {
    vec![
        PathBuf::from("ddc.toml"),
        home_dir.join(".config").join("ddc.toml"),
        home_dir.join(".ddc.toml"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_candidates_based_on_home_dir() {
        let home_path = PathBuf::from("/home/foo");
        let candidates = get_config_file_candidates(&home_path);
        assert_eq!(candidates.get(0), Some(&PathBuf::from("ddc.toml")));
        assert_eq!(
            candidates.get(1),
            Some(&PathBuf::from("/home/foo/.config/ddc.toml"))
        );
        assert_eq!(
            candidates.get(2),
            Some(&PathBuf::from("/home/foo/.ddc.toml"))
        );
        assert_eq!(candidates.get(4), None);
    }

    #[test]
    fn test_load_config_file() {
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        std::env::set_current_dir(&root_dir).unwrap();
        let cfg_data = r#"
[[paths]]
path = "projects/"
discovery = true
        "#;
        fs::write(&root_dir.join(".ddc.toml"), cfg_data).unwrap();

        let config = load_config_file(root_dir).unwrap();
        assert_eq!(config.paths.len(), 1);
    }

    #[test]
    fn test_load_config_file_detects_config() {
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        std::env::set_current_dir(&root_dir).unwrap();

        let result = load_config_file(root_dir);
        assert!(
            matches!(result, Err(ConfigError::ConfigurationFileNotFound)),
            "WARNING: If this test fail, there is a change that the ddc.toml exists in current working directory!"
        );
    }

    #[test]
    fn test_load_config_file_reads_raw_config() {
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        std::env::set_current_dir(&root_dir).unwrap();
        fs::create_dir_all(root_dir.join(".ddc.toml")).unwrap();

        let result = load_config_file(root_dir);
        assert!(matches!(
            result,
            Err(ConfigError::CantLoadConfigurationFile { inner: _ })
        ));
    }

    #[test]
    fn test_load_config_file_parses_config() {
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        std::env::set_current_dir(&root_dir).unwrap();
        fs::write(&root_dir.join(".ddc.toml"), "").unwrap();

        let result = load_config_file(root_dir);
        assert!(matches!(
            result,
            Err(ConfigError::CannotParseConfigurationFile { inner: _ })
        ));
    }
}
