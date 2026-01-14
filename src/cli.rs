use clap::{ArgAction, Parser, Subcommand};

use crate::logging::LoggingLevel;

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
    Analyze(AnalyserSharedArgs),
    /// Interactive browser of the analyzed data
    Browse(AnalyserSharedArgs),
}

#[derive(Parser, Debug, Default)]
pub struct AnalyserSharedArgs {
    /// Do not display the progress bar
    #[arg(short = 'p', long)]
    pub no_progress: bool,
}

#[derive(Debug, Default)]
pub struct UiConfig {
    pub show_progress: bool,
}

impl From<&Args> for UiConfig {
    fn from(args: &Args) -> Self {
        let level = LoggingLevel::from(args.verbosity);
        let show = match args.command {
            Some(Commands::Analyze(ref cmd_args)) => !cmd_args.no_progress,
            _ => true,
        };
        let show_progress = match (level, show) {
            (LoggingLevel::Traces, _) => false,
            (_, true) => true,
            (_, false) => false,
        };

        Self { show_progress }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_config_respects_commands_args() {
        // Enabled by default
        let args = Args {
            verbosity: 0,
            command: Some(Commands::Analyze(AnalyserSharedArgs { no_progress: false })),
        };
        assert_eq!(UiConfig::from(&args).show_progress, true);

        // Disabled
        let args = Args {
            verbosity: 0,
            command: Some(Commands::Analyze(AnalyserSharedArgs { no_progress: true })),
        };
        assert_eq!(UiConfig::from(&args).show_progress, false);
    }

    #[test]
    fn test_ui_config_hides_progress_on_high_log_level() {
        let args = Args {
            verbosity: 2,
            command: Some(Commands::Analyze(AnalyserSharedArgs { no_progress: false })),
        };
        assert_eq!(UiConfig::from(&args).show_progress, false);
    }
}
