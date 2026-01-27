use std::path::PathBuf;

/// Configuration entity for the context extraction process.
/// Encapsulates all constraints and target definitions.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextConfig {
    pub root_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub max_depth: Option<usize>,
    pub include_hidden: bool,
    pub verbose: bool,
}

impl ContextConfig {
    /// Creates a new configuration with validated parameters.
    pub fn new(
        root_path: PathBuf,
        output_path: Option<PathBuf>,
        max_depth: Option<usize>,
        include_hidden: bool,
        verbose: bool,
    ) -> Self {
        Self {
            root_path,
            output_path,
            max_depth,
            include_hidden,
            verbose,
        }
    }
}

impl Default for ContextConfig {
    /// Provides safe defaults for the configuration.
    fn default() -> Self {
        Self {
            root_path: PathBuf::from("."),
            output_path: None,
            max_depth: None,
            include_hidden: false,
            verbose: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that the default configuration points to current directory.
    #[test]
    fn test_default_config() {
        let config = ContextConfig::default();
        assert_eq!(config.root_path, PathBuf::from("."));
        assert_eq!(config.include_hidden, false);
    }

    /// Tests manual construction of configuration.
    #[test]
    fn test_new_config() {
        let path = PathBuf::from("/tmp");
        let config = ContextConfig::new(path.clone(), None, Some(5), true, true);
        
        assert_eq!(config.root_path, path);
        assert_eq!(config.max_depth, Some(5));
        assert!(config.include_hidden);
    }
}