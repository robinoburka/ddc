use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PathDefinition {
    pub path: PathBuf,
    #[serde(default)]
    pub discovery: bool,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub paths: Vec<PathDefinition>,
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
}
