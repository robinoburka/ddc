use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Select};
use owo_colors::OwoColorize;
use tracing::debug;

use crate::config::get_config_file_candidates;

#[derive(thiserror::Error, Debug)]
pub enum GenerateConfigError {
    #[error("Interrupted by user")]
    Interrupted,
    #[error("File already exists. Not overwriting.")]
    AlreadyExist,
    #[error("Config couldn't be saved: {inner}")]
    CannotWriteFile {
        #[from]
        inner: std::io::Error,
    },
    #[error("Failed to obtain info from the user: {inner}")]
    CannotObtainData {
        #[from]
        inner: dialoguer::Error,
    },
}

pub fn generate_config(home_dir: &Path) -> Result<(), GenerateConfigError> {
    let mut interaction = DialoguerInteraction;
    generate_config_inner(&mut io::stdout(), &mut interaction, home_dir)
}

fn generate_config_inner<W: Write, I: GenerateConfigInteraction>(
    out: &mut W,
    interaction: &mut I,
    home_dir: &Path,
) -> Result<(), GenerateConfigError> {
    let example_config = include_str!("../assets/example_config.toml");
    let candidates = get_config_file_candidates(home_dir);

    let path = interaction.select_path(&candidates)?;
    debug!("Looking for a configuration file: {}", path.display());
    if path.exists() {
        let confirmation = interaction.confirm_overwrite()?;
        if !confirmation {
            return Err(GenerateConfigError::AlreadyExist);
        }
    }

    debug!("Using configuration file: {}", path.display());
    fs::write(&path, example_config)?;

    writeln!(
        out,
        "\n{}",
        "Configuration file was successfully created!".bold()
    )
    .expect("Failed to write to stdout");
    writeln!(
        out,
        "Go to the file ({}) and adjust the content based on your needs.",
        path.display().green()
    )
    .expect("Failed to write to stdout");

    Ok(())
}

trait GenerateConfigInteraction {
    fn select_path(&mut self, candidates: &[PathBuf]) -> Result<PathBuf, GenerateConfigError>;
    fn confirm_overwrite(&mut self) -> Result<bool, GenerateConfigError>;
}

struct DialoguerInteraction;

impl GenerateConfigInteraction for DialoguerInteraction {
    fn select_path(&mut self, candidates: &[PathBuf]) -> Result<PathBuf, GenerateConfigError> {
        let candidates_to_display = candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>();

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select preferred location for the configuration file:")
            .items(&candidates_to_display)
            .default(0)
            .report(true)
            .clear(true)
            .interact_opt()?
            .ok_or(GenerateConfigError::Interrupted)?;

        let requested_path = candidates
            .get(selection)
            .expect("Obtained option ot ouf the range. Programmer error?");

        Ok(requested_path.clone())
    }

    fn confirm_overwrite(&mut self) -> Result<bool, GenerateConfigError> {
        let confirmation = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("File already exists. Overwrite?")
            .default(false)
            .show_default(true)
            .report(true)
            .wait_for_newline(false)
            .interact_opt()?
            .ok_or(GenerateConfigError::Interrupted)?;

        Ok(confirmation)
    }
}

#[cfg(test)]
mod tests {
    use toml;

    use super::*;
    use crate::config::Config;

    struct TestsInteraction {
        select: usize,
        confirmation: Option<Result<bool, GenerateConfigError>>,
    }

    impl GenerateConfigInteraction for TestsInteraction {
        fn select_path(&mut self, candidates: &[PathBuf]) -> Result<PathBuf, GenerateConfigError> {
            Ok(candidates.get(self.select).unwrap().clone())
        }

        fn confirm_overwrite(&mut self) -> Result<bool, GenerateConfigError> {
            self.confirmation.take().unwrap()
        }
    }

    #[test]
    fn test_generate_config_on_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root_path = tmp.path();

        let mut buffer = Vec::new();
        let mut interaction = TestsInteraction {
            select: 1,
            confirmation: Some(Ok(false)),
        };

        let results = generate_config_inner(&mut buffer, &mut interaction, root_path);
        assert_eq!(results.unwrap(), ());

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Configuration file was successfully created!"));
        assert!(output.contains("Go to the file"));
        assert!(output.contains(".ddc.toml"));

        let cfg_data = fs::read_to_string(&root_path.join(".ddc.toml")).unwrap();
        assert!(matches!(toml::from_str::<Config>(cfg_data.as_str()), Ok(_)));
    }

    #[test]
    fn test_generate_config_on_path_rewrite_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let root_path = tmp.path();

        fs::write(&root_path.join(".ddc.toml"), "Hello, World!").unwrap();

        let mut buffer = Vec::new();
        let mut interaction = TestsInteraction {
            select: 1,
            confirmation: Some(Ok(true)),
        };

        let results = generate_config_inner(&mut buffer, &mut interaction, root_path);
        assert_eq!(results.unwrap(), ());

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Configuration file was successfully created!"));
        assert!(output.contains("Go to the file"));
        assert!(output.contains(".ddc.toml"));

        let cfg_data = fs::read_to_string(&root_path.join(".ddc.toml")).unwrap();
        assert!(matches!(toml::from_str::<Config>(cfg_data.as_str()), Ok(_)));
    }

    #[test]
    fn test_generate_config_on_path_keep_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let root_path = tmp.path();

        fs::write(&root_path.join(".ddc.toml"), "Hello, World!").unwrap();

        let mut buffer = Vec::new();
        let mut interaction = TestsInteraction {
            select: 1,
            confirmation: Some(Ok(false)),
        };

        let results = generate_config_inner(&mut buffer, &mut interaction, root_path);
        assert!(matches!(results, Err(GenerateConfigError::AlreadyExist)));

        let cfg_data = fs::read_to_string(&root_path.join(".ddc.toml")).unwrap();
        assert!(matches!(
            toml::from_str::<Config>(cfg_data.as_str()),
            Err(_)
        ));
    }

    #[test]
    fn test_generate_config_on_path_keep_existing_as_default() {
        let tmp = tempfile::tempdir().unwrap();
        let root_path = tmp.path();

        fs::write(&root_path.join(".ddc.toml"), "Hello, World!").unwrap();

        let mut buffer = Vec::new();
        let mut interaction = TestsInteraction {
            select: 1,
            confirmation: Some(Ok(false)),
        };

        let results = generate_config_inner(&mut buffer, &mut interaction, root_path);
        assert!(matches!(results, Err(GenerateConfigError::AlreadyExist)));

        let cfg_data = fs::read_to_string(&root_path.join(".ddc.toml")).unwrap();
        assert!(matches!(
            toml::from_str::<Config>(cfg_data.as_str()),
            Err(_)
        ));
    }

    #[test]
    fn test_generate_config_on_path_can_be_interrupted() {
        let tmp = tempfile::tempdir().unwrap();
        let root_path = tmp.path();

        fs::write(&root_path.join(".ddc.toml"), "Hello, World!").unwrap();

        let mut buffer = Vec::new();
        let mut interaction = TestsInteraction {
            select: 1,
            confirmation: Some(Err(GenerateConfigError::Interrupted)),
        };

        let results = generate_config_inner(&mut buffer, &mut interaction, root_path);
        assert!(matches!(results, Err(GenerateConfigError::Interrupted)));

        let cfg_data = fs::read_to_string(&root_path.join(".ddc.toml")).unwrap();
        assert!(matches!(
            toml::from_str::<Config>(cfg_data.as_str()),
            Err(_)
        ));
    }
}
