use anyhow::{Context, Result};
use std::path::Path;
use std::fs;
use crate::adapters::parsers::FileParser;

pub struct PdfParser;

impl PdfParser {
    pub fn new() -> Self {
        Self
    }
}

impl FileParser for PdfParser {
    fn parse(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path)?;
        let text = pdf_extract::extract_text_from_mem(&bytes)
            .with_context(|| "Failed to extract text from PDF")?;
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_pdf_asset() {
        let path = PathBuf::from("tests/assets/test.pdf");
        if !path.exists() {
            println!("SKIPPING: PDF test asset not found at {:?}", path);
            return;
        }

        let parser = PdfParser::new();
        let result = parser.parse(&path).expect("Should parse PDF");
        assert!(!result.is_empty());
    }
}