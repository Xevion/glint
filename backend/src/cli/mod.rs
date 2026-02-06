pub mod seed;

use clap::{Args, Parser, Subcommand};

/// Verbose logging arguments that can be flattened into any CLI parser.
///
/// Use `-v` for debug logging, `-vv` for trace logging.
#[derive(Args, Debug, Clone, Default)]
pub struct VerboseArgs {
    /// Increase log verbosity (-v for debug, -vv for trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// Glint - Shader preview catalog and comparison tool
#[derive(Parser, Debug)]
#[command(name = "glint")]
#[command(about = "Backend API server for shader screenshot catalog")]
#[command(version)]
pub struct Cli {
    #[command(flatten)]
    pub verbose: VerboseArgs,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Seed the database with sample data
    Seed,
}
