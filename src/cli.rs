use clap::{ArgAction, Parser, Subcommand};

pub static COMMAND_NAME: &str = "ddc";

#[derive(Parser, Debug)]
#[command(name = COMMAND_NAME)]
pub struct Args {
    /// Sets the level of verbosity (--verbose/-v, -vv)
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub verbosity: u8,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generates a skeleton of the configuration file
    GenerateConfig,
    /// Show default paths that are explored
    ShowDefinitions,
    /// Analyzes data (default command)
    Analyze(AnalyzeArgs),
}

#[derive(Parser, Debug, Default)]
pub struct AnalyzeArgs {}
