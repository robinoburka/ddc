use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

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
}

pub fn generate_config(home_dir: &Path) -> Result<(), GenerateConfigError> {
    generate_config_inner(&mut io::stdin().lock(), &mut io::stdout(), home_dir)
}

fn generate_config_inner<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    home_dir: &Path,
) -> Result<(), GenerateConfigError> {
    let example_config = include_str!("../assets/example_config.toml");

    let path = obtain_path(input, out, home_dir).ok_or(GenerateConfigError::Interrupted)?;
    writeln!(out, "{}", path.display().dimmed()).expect("Failed to write to stdout");
    write_to_path(input, out, &path)?;

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
        path.display().dimmed()
    )
    .expect("Failed to write to stdout");

    Ok(())
}

fn obtain_path<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    home_dir: &Path,
) -> Option<PathBuf> {
    let candidates = get_config_file_candidates(home_dir);

    writeln!(out, "Select preferred location for the configuration file:")
        .expect("Failed to write to stdout");
    for (i, path) in candidates.iter().enumerate() {
        writeln!(
            out,
            "  {}: {}",
            i.bold().bright_yellow(),
            path.display().bright_cyan()
        )
        .expect("Failed to write to stdout");
    }
    writeln!(out, "{}",
        "Note: The relative path choice is suitable for development purposes. Prefer any 'dotfile' variant for production.".dimmed()
    ).expect("Failed to write to stdout");

    loop {
        let mut response = String::new();
        write!(
            out,
            "\n{}",
            format!("Choose a path ([0-{}]/q)> ", candidates.len() - 1).bold()
        )
        .expect("Failed to write to stdout");
        out.flush().expect("Failed to flush stdout");
        input.read_line(&mut response).expect("Failed to read line");
        let choice: usize = match response.trim().to_lowercase().as_str() {
            "q" => break None,
            number => match number.parse() {
                Err(_) => {
                    writeln!(out, "{}", "Please enter a valid number.".bright_red())
                        .expect("Failed to write to stdout");
                    continue;
                }
                Ok(num) if num >= candidates.len() => {
                    writeln!(out, "{}", "Please enter a valid choice.".bright_red())
                        .expect("Failed to write to stdout");
                    continue;
                }
                Ok(num) => num,
            },
        };
        break Some(candidates[choice].clone());
    }
}

fn write_to_path<R: BufRead, W: Write>(
    input: &mut R,
    out: &mut W,
    path: &Path,
) -> Result<(), GenerateConfigError> {
    debug!("Looking for a configuration file: {}", path.display());
    if !path.exists() {
        return Ok(());
    }
    write!(
        out,
        "\n{}",
        "File already exists. Overwrite? (y/N)> ".bold()
    )
    .expect("Failed to write to stdout");
    out.flush().expect("Failed to flush stdout");
    let mut response = String::new();
    input.read_line(&mut response).expect("Failed to read line");
    match response.trim().to_lowercase().as_str() {
        "y" => Ok(()),
        "n" => Err(GenerateConfigError::AlreadyExist),
        _ => Err(GenerateConfigError::AlreadyExist),
    }
}

#[cfg(test)]
mod tests {
    use toml;

    use super::*;
    use crate::config::Config;

    #[test]
    fn test_generate_config_on_path() {
        let tmp = tempfile::tempdir().unwrap();
        let root_path = tmp.path();

        let input = "1\n";
        let mut reader = io::Cursor::new(input);

        let mut buffer = Vec::new();
        let results = generate_config_inner(&mut reader, &mut buffer, root_path);
        assert_eq!(results.unwrap(), ());

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Select preferred location for the configuration file"));
        assert!(output.contains("Choose a path"));
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

        let input = "1\ny\n";
        let mut reader = io::Cursor::new(input);

        let mut buffer = Vec::new();
        let results = generate_config_inner(&mut reader, &mut buffer, root_path);
        assert_eq!(results.unwrap(), ());

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Select preferred location for the configuration file"));
        assert!(output.contains("Choose a path"));
        assert!(output.contains("File already exists. Overwrite?"));
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

        let input = "1\nn\n";
        let mut reader = io::Cursor::new(input);

        let mut buffer = Vec::new();
        let results = generate_config_inner(&mut reader, &mut buffer, root_path);
        assert!(matches!(results, Err(GenerateConfigError::AlreadyExist)));

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Select preferred location for the configuration file"));
        assert!(output.contains("Choose a path"));
        assert!(output.contains("File already exists. Overwrite?"));

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

        let input = "1\n\n";
        let mut reader = io::Cursor::new(input);

        let mut buffer = Vec::new();
        let results = generate_config_inner(&mut reader, &mut buffer, root_path);
        assert!(matches!(results, Err(GenerateConfigError::AlreadyExist)));

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Select preferred location for the configuration file"));
        assert!(output.contains("Choose a path"));
        assert!(output.contains("File already exists. Overwrite?"));

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

        let input = "q\n";
        let mut reader = io::Cursor::new(input);

        let mut buffer = Vec::new();
        let results = generate_config_inner(&mut reader, &mut buffer, root_path);
        assert!(matches!(results, Err(GenerateConfigError::Interrupted)));

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Select preferred location for the configuration file"));
        assert!(output.contains("Choose a path"));

        let cfg_data = fs::read_to_string(&root_path.join(".ddc.toml")).unwrap();
        assert!(matches!(
            toml::from_str::<Config>(cfg_data.as_str()),
            Err(_)
        ));
    }

    #[test]
    fn test_generate_config_on_path_invalid_inputs() {
        let tmp = tempfile::tempdir().unwrap();
        let root_path = tmp.path();

        let input = "x\n42\nq\n";
        let mut reader = io::Cursor::new(input);

        let mut buffer = Vec::new();
        let results = generate_config_inner(&mut reader, &mut buffer, root_path);
        assert!(matches!(results, Err(GenerateConfigError::Interrupted)));

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("Select preferred location for the configuration file"));
        assert!(output.contains("Choose a path"));
        assert!(output.contains("Please enter a valid number."));
        assert!(output.contains("Please enter a valid choice."));
    }
}
