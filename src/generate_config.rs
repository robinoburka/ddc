use std::fs;
use std::io::{Write, stdin, stdout};
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
    let example_config = include_str!("../assets/example_config.toml");

    let path = obtain_path(home_dir).ok_or(GenerateConfigError::Interrupted)?;
    println!("{}", path.display().dimmed());
    write_to_path(&path)?;

    debug!("Using configuration file: {}", path.display());
    fs::write(&path, example_config)?;

    println!();
    println!("{}", "Configuration file was successfully created!".bold());
    println!(
        "Go to the file ({}) and adjust the content based on your needs.",
        path.display().dimmed()
    );

    Ok(())
}

fn obtain_path(home_dir: &Path) -> Option<PathBuf> {
    let candidates = get_config_file_candidates(home_dir);

    println!("Select preferred location for the configuration file:");
    for (i, path) in candidates.iter().enumerate() {
        println!(
            "  {}: {}",
            i.bold().bright_yellow(),
            path.display().bright_cyan()
        );
    }
    println!("{}",
        "Note: The relative path choice is suitable for development purposes. Prefer any 'dotfile' variant for production.".dimmed()
    );

    loop {
        let mut input = String::new();
        print!(
            "\n{}",
            format!("Choose a path ([0-{}]/q)> ", candidates.len() - 1).bold()
        );
        stdout().flush().expect("Failed to flush stdout");
        stdin().read_line(&mut input).expect("Failed to read line");
        let choice: usize = match input.trim().to_lowercase().as_str() {
            "q" => break None,
            number => match number.parse() {
                Err(_) => {
                    println!("{}", "Please enter a valid number.".bright_red());
                    continue;
                }
                Ok(num) if num >= candidates.len() => {
                    println!("{}", "Please enter a valid choice.".bright_red());
                    continue;
                }
                Ok(num) => num,
            },
        };
        break Some(candidates[choice].clone());
    }
}

fn write_to_path(path: &Path) -> Result<(), GenerateConfigError> {
    debug!("Looking for a configuration file: {}", path.display());
    if !path.exists() {
        return Ok(());
    }
    print!("\n{}", "File already exists. Overwrite? (y/N)> ".bold());
    stdout().flush().expect("Failed to flush stdout");
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read line");
    match input.trim().to_lowercase().as_str() {
        "y" => Ok(()),
        "n" => Err(GenerateConfigError::AlreadyExist),
        _ => Err(GenerateConfigError::AlreadyExist),
    }
}
