use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use owo_colors::OwoColorize;
use tracing::{debug, error};

use crate::cli::{AnalyzeArgs, COMMAND_NAME};
use crate::config::{Config, get_config_file_candidates};
use crate::discovery::{DiscoveryManager, default_discovery_definitions};
use crate::display::print_results;

#[derive(thiserror::Error, Debug)]
pub enum AnalyzeError {
    #[error(
        "Configuration file not found. If this is the first run, call '{} generate-config' command first.",
        COMMAND_NAME
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
    #[error("No results found. Do you use one of the supported languages?")]
    NoResultsFound,
}

pub fn analyze(args: AnalyzeArgs, home_dir: &Path) -> Result<(), AnalyzeError> {
    analyze_inner(&mut io::stdout(), args, home_dir)
}

fn analyze_inner<W: Write>(
    out: &mut W,
    args: AnalyzeArgs,
    home_dir: &Path,
) -> Result<(), AnalyzeError> {
    if args.show_definitions {
        show_default_definitions(out, home_dir);
        return Ok(());
    }

    let candidates = get_config_file_candidates(home_dir);
    let Some(cfg_path) = find_config_file(&candidates) else {
        error!("Configuration file not found");
        Err(AnalyzeError::ConfigurationFileNotFound)?
    };

    debug!("Using configuration file: {}", cfg_path.display());
    let cfg_data = fs::read_to_string(&cfg_path)?;
    let config: Config = toml::from_str(cfg_data.as_str())?;

    let discovery_results = DiscoveryManager::with_default_loader(home_dir)
        .add_from_config(&config)
        .collect();

    if discovery_results.is_empty() {
        error!("No results found.");
        return Err(AnalyzeError::NoResultsFound);
    }
    print_results(out, discovery_results);

    Ok(())
}

fn show_default_definitions<W: Write>(out: &mut W, home: &Path) {
    default_discovery_definitions(home)
        .iter()
        .for_each(|definition| {
            writeln!(
                out,
                "{} {} ({}): {}",
                definition.lang.map(|l| l.to_string()).unwrap_or_default(),
                definition.description.bold(),
                if definition.discovery { "🔭" } else { "🧰" },
                definition.path.display().dimmed()
            )
            .expect("Failed to write to stdout");
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_requires_config() {
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();

        let result = analyze(AnalyzeArgs::default(), root_dir);
        assert!(
            matches!(result, Err(AnalyzeError::ConfigurationFileNotFound)),
            "WARNING: If this test fail, there is a change that the ddc.toml exists in current working directory!"
        );
    }

    #[test]
    fn test_analyze_must_read_config() {
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        fs::create_dir_all(root_dir.join(".ddc.toml")).unwrap();

        let result = analyze(AnalyzeArgs::default(), root_dir);
        assert!(matches!(
            result,
            Err(AnalyzeError::CantLoadConfigurationFile { inner: _ })
        ));
    }

    #[test]
    fn test_analyze_must_parse_config() {
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        fs::write(&root_dir.join(".ddc.toml"), "").unwrap();

        let result = analyze(AnalyzeArgs::default(), root_dir);
        assert!(matches!(
            result,
            Err(AnalyzeError::CannotParseConfigurationFile { inner: _ })
        ));
    }

    #[test]
    fn test_analyze_show_definitions() {
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();

        let mut buffer = Vec::new();
        let result = analyze_inner(
            &mut buffer,
            AnalyzeArgs {
                show_definitions: true,
            },
            root_dir,
        );
        assert_eq!(result.unwrap(), ());

        let output = String::from_utf8(buffer).unwrap();
        assert!(
            output.contains(
                root_dir
                    .join(".cargo/registry")
                    .display()
                    .to_string()
                    .as_str()
            )
        );
        assert!(output.contains(root_dir.join(".cache/uv").display().to_string().as_str()));
    }

    #[test]
    fn test_analyze_discovery() {
        let tmp = tempfile::tempdir().unwrap();
        let root_path = tmp.path();

        fs::create_dir_all(root_path.join("projects/python/venv/bin")).unwrap();
        fs::write(
            root_path.join("projects/python/venv/bin/python"),
            "Python executable mock",
        )
        .unwrap();
        fs::create_dir_all(root_path.join(".cache/uv")).unwrap();
        fs::write(
            root_path.join(".cache/uv/CACHEDIR.TAG"),
            "Signature: 8a477f597d28d172789f06886806bc55",
        )
        .unwrap();

        let cfc_data = r#"
[[paths]]
path = "projects/"
discovery = true
        "#;
        fs::write(&root_path.join(".ddc.toml"), cfc_data).unwrap();

        let mut buffer = Vec::new();
        let result = analyze_inner(&mut buffer, AnalyzeArgs::default(), root_path);
        assert_eq!(result.unwrap(), ());

        let output = String::from_utf8(buffer).unwrap();
        assert!(
            output.contains(
                root_path
                    .join("projects/python/venv")
                    .display()
                    .to_string()
                    .as_str()
            )
        );
        assert!(output.contains(root_path.join(".cache/uv").display().to_string().as_str()));
    }
}
