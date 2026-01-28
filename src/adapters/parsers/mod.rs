use anyhow::Result;
use std::path::Path;

pub mod docx;
pub mod excel;
pub mod fallback;
pub mod pdf;

/// Strategy interface for parsing specific file formats.
pub trait FileParser: Send + Sync {
    /// Extracts text content from the file at the given path.
    fn parse(&self, path: &Path) -> Result<String>;
}
