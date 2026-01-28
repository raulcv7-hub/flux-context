use anyhow::Result;
use std::path::Path;

pub mod pdf;
pub mod docx;
pub mod excel;
pub mod fallback;

/// Strategy interface for parsing specific file formats.
pub trait FileParser: Send + Sync {
    /// Extracts text content from the file at the given path.
    fn parse(&self, path: &Path) -> Result<String>;
}