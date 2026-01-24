use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand};

use crate::logging::LoggingLevel;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
    /// Sets the level of verbosity (--verbose/-v, -vv for tracing output)
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
    /// Interactive browser of the analyzed data
    Browse(BrowseArgs),
}

#[derive(Args, Debug, Default)]
pub struct AnalysisSharedArgs {
    /// Do not display the progress bar
    #[arg(short = 'p', long)]
    pub no_progress: bool,
    /// Use the following config file instead of autodiscovery process
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}
#[derive(Parser, Debug, Default)]
pub struct AnalyzeArgs {
    #[command(flatten)]
    pub shared: AnalysisSharedArgs,
}

#[derive(Parser, Debug)]
pub struct BrowseArgs {
    #[command(flatten)]
    pub shared: AnalysisSharedArgs,
}

#[derive(Debug, Default)]
pub struct UiConfig {
    pub show_progress: bool,
}

impl From<&CliArgs> for UiConfig {
    fn from(args: &CliArgs) -> Self {
        let level = LoggingLevel::from(args.verbosity);
        let show = match args.command {
            Some(Commands::Analyze(ref cmd_args)) => !cmd_args.shared.no_progress,
            Some(Commands::Browse(ref cmd_args)) => !cmd_args.shared.no_progress,
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
        let mut cmd_args = AnalyzeArgs::default();
        cmd_args.shared.no_progress = false;
        let args = CliArgs {
            verbosity: 0,
            command: Some(Commands::Analyze(cmd_args)),
        };
        assert_eq!(UiConfig::from(&args).show_progress, true);

        // Disabled
        let mut cmd_args = AnalyzeArgs::default();
        cmd_args.shared.no_progress = true;
        let args = CliArgs {
            verbosity: 0,
            command: Some(Commands::Analyze(cmd_args)),
        };
        assert_eq!(UiConfig::from(&args).show_progress, false);
    }

    #[test]
    fn test_ui_config_hides_progress_on_high_log_level() {
        let mut cmd_args = AnalyzeArgs::default();
        cmd_args.shared.no_progress = false;

        let args = CliArgs {
            verbosity: 2,
            command: Some(Commands::Analyze(cmd_args)),
        };
        assert_eq!(UiConfig::from(&args).show_progress, false);
    }
}
