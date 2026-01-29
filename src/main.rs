//! Entry point for the Context Engine CLI.

use arboard::Clipboard;
use clap::Parser;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::PathBuf;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use context::adapters::fs_reader::FsReader;
use context::adapters::fs_scanner::FsScanner;
use context::adapters::output::json::JsonWriter;
use context::adapters::output::markdown::MarkdownWriter;
use context::adapters::output::xml::XmlWriter;
use context::core::config::{ContextConfig, OutputFormat};
use context::ports::reader::FileReader;
use context::ports::scanner::ProjectScanner;
use context::ports::writer::ContextWriter;
use context::ui::run_tui;

/// High-performance AI Context Generator.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the project root to scan.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Optional output file path.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format (xml, markdown, json).
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Xml)]
    format: OutputFormat,

    /// Copy the result to the system clipboard.
    #[arg(short, long, default_value_t = false)]
    clip: bool,

    /// Interactive mode (TUI) to select files manually.
    #[arg(short = 'I', long, default_value_t = false)]
    interactive: bool,

    /// Maximum depth to traverse.
    #[arg(short, long)]
    depth: Option<usize>,

    /// Include hidden files and directories.
    #[arg(long, default_value_t = false)]
    include_hidden: bool,

    /// Filter by extension (comma separated).
    #[arg(short = 'e', long, value_delimiter = ',')]
    extensions: Vec<String>,

    /// Exclude extensions (comma separated).
    #[arg(short = 'x', long, value_delimiter = ',')]
    exclude_extensions: Vec<String>,

    /// Only include paths containing this string.
    #[arg(short = 'i', long)]
    include_path: Vec<String>,

    /// Exclude paths containing this string.
    #[arg(short = 'X', long)]
    exclude_path: Vec<String>,

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
        cli.format,
        cli.depth,
        cli.include_hidden,
        cli.clip,
        cli.verbose > 0,
        cli.extensions,
        cli.exclude_extensions,
        cli.include_path,
        cli.exclude_path,
    );

    // 1. SCANNING
    info!("Phase 1: Scanning directory...");
    let scanner = FsScanner::new();
    let mut files = match scanner.scan(&config) {
        Ok(f) => f,
        Err(e) => {
            error!("Scanning failed: {}", e);
            return Err(e);
        }
    };
    info!("Found {} files.", files.len());

    // --- TUI INTERCEPTION ---
    if cli.interactive {
        info!("Launching Interactive Mode...");

        match run_tui(&files, &config.root_path) {
            Ok(Some(selected_paths)) => {
                let prev_count = files.len();

                files.retain(|node| selected_paths.contains(&node.relative_path));

                info!(
                    "Interactive selection: Kept {}/{} files.",
                    files.len(),
                    prev_count
                );
            }
            Ok(None) => {
                warn!("Interactive mode cancelled. Exiting.");
                return Ok(());
            }
            Err(e) => {
                error!("TUI failed: {}", e);
                return Err(e);
            }
        }
    }

    // 2. READING
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
    info!("Phase 3: Generating output ({:?})...", config.output_format);

    let buffer = generate_output_buffer(&contexts, &config)?;

    if config.to_clipboard {
        let output_str = String::from_utf8(buffer.clone())?;
        match Clipboard::new() {
            Ok(mut clipboard) => {
                if let Err(e) = clipboard.set_text(&output_str) {
                    error!("Failed to copy to clipboard: {}", e);
                } else {
                    info!("Output copied to clipboard! ({} chars)", output_str.len());
                }
            }
            Err(e) => error!("Could not access clipboard: {}", e),
        }

        if let Some(path) = &cli.output {
            let mut file = File::create(path)?;
            file.write_all(&buffer)?;
            info!("Context also written to: {:?}", path);
        } else {
            warn!("Output copied to clipboard. Suppressing stdout.");
        }
    } else {
        match &cli.output {
            Some(path) => {
                let file = File::create(path)?;
                let mut buf_writer = BufWriter::new(file);
                buf_writer.write_all(&buffer)?;
                info!("Context written to: {:?}", path);
            }
            None => {
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                handle.write_all(&buffer)?;
            }
        }
    }

    Ok(())
}

fn generate_output_buffer(
    files: &[context::core::content::FileContext],
    config: &ContextConfig,
) -> anyhow::Result<Vec<u8>> {
    let mut buffer = Vec::new();

    match config.output_format {
        OutputFormat::Xml => {
            let writer = XmlWriter::new();
            writer.write(files, config, &mut buffer)?;
        }
        OutputFormat::Markdown => {
            let writer = MarkdownWriter::new();
            writer.write(files, config, &mut buffer)?;
        }
        OutputFormat::Json => {
            let writer = JsonWriter::new();
            writer.write(files, config, &mut buffer)?;
        }
    }

    Ok(buffer)
}

fn init_logging(verbosity: u8) {
    let level = match verbosity {
        0 => Level::WARN,
        1 => Level::INFO,
        _ => Level::DEBUG,
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_writer(io::stderr)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
