use crate::core::config::ContextConfig;
use crate::core::content::FileContext;
use anyhow::Result;
use std::io::Write;

/// Strategy interface for output generation.
pub trait ContextWriter {
    /// Writes the processed context to the provided writer.
    fn write<W: Write>(
        &self,
        files: &[FileContext],
        config: &ContextConfig,
        writer: W,
    ) -> Result<()>;
}
