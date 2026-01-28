use std::path::PathBuf;

/// Domain entity representing a file found in the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileNode {
    pub path: PathBuf,
    pub relative_path: PathBuf,
}

impl FileNode {
    /// Creates a new FileNode.
    pub fn new(path: PathBuf, relative_path: PathBuf) -> Self {
        Self {
            path,
            relative_path,
        }
    }
}