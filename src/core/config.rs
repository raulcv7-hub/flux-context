use std::collections::HashSet;
use std::path::PathBuf;

/// Configuration entity for the context extraction process.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextConfig {
    pub root_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub max_depth: Option<usize>,
    pub include_hidden: bool,
    pub to_clipboard: bool,
    pub verbose: bool,
    pub include_extensions: HashSet<String>,
    pub exclude_extensions: HashSet<String>,
}

impl ContextConfig {
    /// Creates a new configuration with validated parameters.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        root_path: PathBuf,
        output_path: Option<PathBuf>,
        max_depth: Option<usize>,
        include_hidden: bool,
        to_clipboard: bool,
        verbose: bool,
        include_exts: Vec<String>,
        exclude_exts: Vec<String>,
    ) -> Self {
        // Normalize extensions to lowercase for consistent matching
        let include_extensions = include_exts.into_iter().map(|e| e.to_lowercase()).collect();

        let exclude_extensions = exclude_exts.into_iter().map(|e| e.to_lowercase()).collect();

        Self {
            root_path,
            output_path,
            max_depth,
            include_hidden,
            to_clipboard,
            verbose,
            include_extensions,
            exclude_extensions,
        }
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            output_path: None,
            max_depth: None,
            include_hidden: false,
            to_clipboard: false,
            verbose: false,
            include_extensions: HashSet::new(),
            exclude_extensions: HashSet::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_normalization() {
        let config = ContextConfig::new(
            PathBuf::from("."),
            None,
            None,
            false,
            false,
            false,
            vec!["RS".to_string(), "ToMl".to_string()],
            vec![],
        );

        assert!(config.include_extensions.contains("rs"));
        assert!(config.include_extensions.contains("toml"));
    }
}
