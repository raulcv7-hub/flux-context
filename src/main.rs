//! Entry point for the Context Engine CLI.
//! Parses arguments and orchestrates the execution.

use clap::Parser;
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use context_engine::core::config::ContextConfig;

/// Ingests project folders and serializes them into one file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the project root to scan. Defaults to current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Optional output file path. If not provided, prints to stdout.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Maximum depth to traverse.
    #[arg(short, long)]
    depth: Option<usize>,

    /// Include hidden files and directories (starting with dot).
    #[arg(long, default_value_t = false)]
    include_hidden: bool,

    /// Turn debugging information on.
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_logging(cli.verbose);

    info!("Starting Context Engine...");

    let _config = ContextConfig::new(
        cli.path,
        cli.output,
        cli.depth,
        cli.include_hidden,
        cli.verbose > 0,
    );

    info!("Configuration loaded. Scanning not yet implemented.");

    Ok(())
}

/// Initializes the global tracing subscriber for logging.
fn init_logging(verbosity: u8) {
    let level = match verbosity {
        0 => Level::WARN,
        1 => Level::INFO,
        _ => Level::DEBUG,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that the Clap CLI definition is valid.
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}