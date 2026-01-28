//! Entry point for the Context Engine CLI.
//! Parses arguments and orchestrates the execution.

use clap::Parser;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufWriter};
use std::path::PathBuf;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use context_engine::adapters::fs_reader::FsReader;
use context_engine::adapters::fs_scanner::FsScanner;
use context_engine::adapters::output::xml::XmlWriter;
use context_engine::core::config::ContextConfig;
use context_engine::ports::reader::FileReader;
use context_engine::ports::scanner::ProjectScanner;
use context_engine::ports::writer::ContextWriter;

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

    let config = ContextConfig::new(
        cli.path,
        cli.output.clone(),
        cli.depth,
        cli.include_hidden,
        cli.verbose > 0,
    );

    // 1. SCANNING
    info!("Phase 1: Scanning directory...");
    let scanner = FsScanner::new();
    let files = match scanner.scan(&config) {
        Ok(f) => f,
        Err(e) => {
            error!("Scanning failed: {}", e);
            return Err(e);
        }
    };
    info!("Found {} files.", files.len());

    // 2. READING (Parallel)
    info!("Phase 2: Reading content...");
    let reader = FsReader::new();

    let contexts: Vec<_> = files
        .par_iter()
        .map(|node| reader.read_file(node))
        .collect();

    let total_tokens: usize = contexts.iter().map(|c| c.token_count).sum();
    info!(
        "Processed {} files. Total estimated tokens: {}",
        contexts.len(),
        total_tokens
    );

    // 3. OUTPUT
    info!("Phase 3: Generating output...");
    let writer_strategy = XmlWriter::new();

    match &cli.output {
        Some(path) => {
            let file = File::create(path)?;
            let mut buf_writer = BufWriter::new(file);
            writer_strategy.write(&contexts, &config, &mut buf_writer)?;
            info!("Context written to: {:?}", path);
        }
        None => {
            // Write to stdout if no file is specified
            let stdout = io::stdout();
            let handle = stdout.lock();
            let mut buf_writer = BufWriter::new(handle);
            writer_strategy.write(&contexts, &config, &mut buf_writer)?;
        }
    }

    Ok(())
}

/// Initializes the global tracing subscriber for logging.
fn init_logging(verbosity: u8) {
    let level = match verbosity {
        0 => Level::WARN,
        1 => Level::INFO,
        _ => Level::DEBUG,
    };

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
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
