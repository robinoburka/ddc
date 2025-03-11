use clap::{ArgAction, Parser};

pub static COMMAND_NAME: &str = "ddc";

#[derive(Parser, Debug)]
#[command(name = COMMAND_NAME)]
pub struct Args {
    /// Sets the level of verbosity (--verbose/-v, -vv)
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count)]
    pub verbosity: u8,
}
