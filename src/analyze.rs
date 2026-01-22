use std::io::{self, Write};
use std::path::Path;

use owo_colors::OwoColorize;
use tracing::error;

use crate::cli::UiConfig;
use crate::config::{ConfigError, load_config_file};
use crate::discovery::{
    DiscoveryManager, ExternalDiscoveryDefinition, default_discovery_definitions,
};
use crate::display::{display_progress_bar, print_results};

#[derive(thiserror::Error, Debug)]
pub enum AnalyzeError {
    #[error("Unable to load configuration file. See details for more information: {inner}")]
    ConfigError {
        #[from]
        inner: ConfigError,
    },
    #[error("No results found. Do you use one of the supported languages?")]
    NoResultsFound,
}

pub fn analyze(ui_config: &UiConfig, home_dir: &Path) -> Result<(), AnalyzeError> {
    analyze_inner(&mut io::stdout(), ui_config, home_dir)
}

fn analyze_inner<W: Write>(
    out: &mut W,
    ui_config: &UiConfig,
    home_dir: &Path,
) -> Result<(), AnalyzeError> {
    let config = load_config_file(home_dir)?;
    let definitions = config
        .paths
        .into_iter()
        .map(|p| ExternalDiscoveryDefinition { path: p.path })
        .collect::<Vec<_>>();

    let discovery_manager = DiscoveryManager::new(home_dir).add_definitions(&definitions);

    if ui_config.show_progress {
        let progress_channel = discovery_manager.subscribe();
        rayon::spawn(move || {
            display_progress_bar(progress_channel);
        });
    }

    let discovery_results = discovery_manager.collect();
    if discovery_results.projects.is_empty() && discovery_results.tools.len() == 1 {
        error!("No results found.");
        return Err(AnalyzeError::NoResultsFound);
    }
    print_results(out, discovery_results);

    Ok(())
}

pub fn show_default_definitions(home_dir: &Path) {
    show_default_definitions_inner(&mut io::stdout(), home_dir)
}

fn show_default_definitions_inner<W: Write>(out: &mut W, home: &Path) {
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
        });
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_show_default_definitions() {
        let tmp = tempfile::tempdir().unwrap();
        let root_dir = tmp.path();
        std::env::set_current_dir(&root_dir).unwrap();

        let mut buffer = Vec::new();
        show_default_definitions_inner(&mut buffer, root_dir);

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
        std::env::set_current_dir(&root_path).unwrap();

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

        let cfg_data = r#"
[[paths]]
path = "projects/"
discovery = true
        "#;
        fs::write(&root_path.join(".ddc.toml"), cfg_data).unwrap();

        let mut buffer = Vec::new();
        let result = analyze_inner(&mut buffer, &UiConfig::default(), root_path);
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
