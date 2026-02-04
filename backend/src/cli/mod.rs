pub mod seed;

use clap::{Parser, Subcommand};

/// Glint - Shader preview catalog and comparison tool
#[derive(Parser, Debug)]
#[command(name = "glint-backend")]
#[command(about = "Backend API server for shader screenshot catalog")]
#[command(version)]
pub struct Cli {
    /// Increase log verbosity (-v for debug, -vv for trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Seed the database with sample data
    Seed,
}
