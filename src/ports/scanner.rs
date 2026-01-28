use anyhow::Result;
use crate::core::config::ContextConfig;
use crate::core::file::FileNode;

/// Interface for scanning a directory structure.
pub trait ProjectScanner {
    /// Scans the project defined in config and returns a list of files.
    fn scan(&self, config: &ContextConfig) -> Result<Vec<FileNode>>;
}