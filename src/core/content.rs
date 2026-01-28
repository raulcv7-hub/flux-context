use std::path::PathBuf;

/// Enum representing the type of content found in a file.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    Text(String),
    Binary,
    Error(String),
}

/// Domain entity representing a processed file with its content and metadata.
#[derive(Debug, Clone)]
pub struct FileContext {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub content: ContentType,
    pub language: String,
    pub token_count: usize,
}

impl FileContext {
    /// Creates a new FileContext.
    pub fn new(
        path: PathBuf,
        relative_path: PathBuf,
        content: ContentType,
        language: String,
        token_count: usize,
    ) -> Self {
        Self {
            path,
            relative_path,
            content,
            language,
            token_count,
        }
    }
}