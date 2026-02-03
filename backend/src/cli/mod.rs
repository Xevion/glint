pub mod seed;

use clap::{Parser, Subcommand};

/// Glint - Shader preview catalog and comparison tool
#[derive(Parser, Debug)]
#[command(name = "glint-backend")]
#[command(about = "Backend API server for shader screenshot catalog")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Seed the database with sample data
    Seed,
}
