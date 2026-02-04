//! Shared CLI argument definitions

use clap::Args;

/// Verbose logging arguments that can be flattened into any CLI parser.
///
/// Use `-v` for debug logging, `-vv` for trace logging.
#[derive(Args, Debug, Clone, Default)]
pub struct VerboseArgs {
    /// Increase log verbosity (-v for debug, -vv for trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}
