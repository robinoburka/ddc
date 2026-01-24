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

pub fn load_config_file(
    home_dir: &Path,
    requested_path: Option<&Path>,
) -> Result<Config, ConfigError> {
    let cfg_data = if let Some(path) = requested_path {
        debug!("Using configuration file: {}", path.display());
        fs::read_to_string(path)?
    } else {
        let candidates = get_config_file_candidates(home_dir);
        let Some(cfg_path) = find_config_file(&candidates) else {
            error!("Configuration file not found");
            Err(ConfigError::ConfigurationFileNotFound)?
        };

        debug!("Using configuration file: {}", cfg_path.display());
        fs::read_to_string(&cfg_path)?
    };
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
        home_dir.join(".config").join("ddc.toml"),
        home_dir.join(".ddc.toml"),
        PathBuf::from("ddc.toml"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_candidates_based_on_home_dir() {
        let home_path = PathBuf::from("/home/foo");
        let candidates = get_config_file_candidates(&home_path);
        assert_eq!(
            candidates.get(0),
            Some(&PathBuf::from("/home/foo/.config/ddc.toml"))
        );
        assert_eq!(
            candidates.get(1),
            Some(&PathBuf::from("/home/foo/.ddc.toml"))
        );
        assert_eq!(candidates.get(2), Some(&PathBuf::from("ddc.toml")));
        assert_eq!(candidates.get(3), None);
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

        let config = load_config_file(root_dir, None).unwrap();
        assert_eq!(config.paths.len(), 1);
    }

    #[test]
    fn test_load_config_file_from_param() {
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        std::env::set_current_dir(&root_dir).unwrap();
        let cfg_data = r#"
[[paths]]
path = "projects/"
discovery = true
        "#;
        fs::write(&root_dir.join("custom.toml"), cfg_data).unwrap();

        let config =
            load_config_file(root_dir, Some(root_dir.join("custom.toml").as_path())).unwrap();
        assert_eq!(config.paths.len(), 1);
    }

    #[test]
    fn test_load_config_file_detects_config() {
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        std::env::set_current_dir(&root_dir).unwrap();

        let result = load_config_file(root_dir, None);
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

        let result = load_config_file(root_dir, None);
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

        let result = load_config_file(root_dir, None);
        assert!(matches!(
            result,
            Err(ConfigError::CannotParseConfigurationFile { inner: _ })
        ));
    }
}
