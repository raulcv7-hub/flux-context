use clap::ValueEnum;
use std::collections::HashSet;
use std::path::PathBuf;

/// Enum defining available output formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Xml,
    Markdown,
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Xml
    }
}

/// Configuration entity for the context extraction process.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextConfig {
    pub root_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub output_format: OutputFormat,
    pub max_depth: Option<usize>,
    pub include_hidden: bool,
    pub to_clipboard: bool,
    pub verbose: bool,
    pub include_extensions: HashSet<String>,
    pub exclude_extensions: HashSet<String>,
    pub include_paths: Vec<String>,
    pub exclude_paths: Vec<String>,
}

impl ContextConfig {
    /// Creates a new configuration with validated parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        root_path: PathBuf,
        output_path: Option<PathBuf>,
        output_format: OutputFormat,
        max_depth: Option<usize>,
        include_hidden: bool,
        to_clipboard: bool,
        verbose: bool,
        include_exts: Vec<String>,
        exclude_exts: Vec<String>,
        include_paths: Vec<String>,
        exclude_paths: Vec<String>,
    ) -> Self {
        let include_extensions = include_exts.into_iter().map(|e| e.to_lowercase()).collect();

        let exclude_extensions = exclude_exts.into_iter().map(|e| e.to_lowercase()).collect();

        Self {
            root_path,
            output_path,
            output_format,
            max_depth,
            include_hidden,
            to_clipboard,
            verbose,
            include_extensions,
            exclude_extensions,
            include_paths,
            exclude_paths,
        }
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            output_path: None,
            output_format: OutputFormat::default(),
            max_depth: None,
            include_hidden: false,
            to_clipboard: false,
            verbose: false,
            include_extensions: HashSet::new(),
            exclude_extensions: HashSet::new(),
            include_paths: Vec::new(),
            exclude_paths: Vec::new(),
        }
    }
}
