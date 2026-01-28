use crate::core::content::{ContentType, FileContext};
use crate::core::file::FileNode;
use crate::ports::reader::FileReader;
use anyhow::{Result, Context};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use regex::Regex;

/// Implementation of FileReader that reads from the local filesystem.
#[derive(Default)]
pub struct FsReader;

impl FsReader {
    /// Creates a new instance of FsReader.
    pub fn new() -> Self {
        Self
    }

    /// Infers programming language from extension.
    fn detect_language(&self, path: &Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("text")
            .to_lowercase()
    }

    /// Simple heuristic for token counting (approximation).
    fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / 3
    }

    // --- PARSERS ---

    /// Extracts text from a PDF file.
    fn parse_pdf(&self, path: &Path) -> Result<String> {
        let bytes = fs::read(path)?;
        let text = pdf_extract::extract_text_from_mem(&bytes)
            .with_context(|| "Failed to extract text from PDF")?;
        Ok(text)
    }

    /// Extracts text from a DOCX file (unzipping and stripping XML).
    fn parse_docx(&self, path: &Path) -> Result<String> {
        let file = File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        // DOCX content is always in word/document.xml
        let mut document_xml = archive.by_name("word/document.xml")
            .with_context(|| "Could not find word/document.xml in docx")?;
        
        let mut xml_content = String::new();
        document_xml.read_to_string(&mut xml_content)?;

        // Simple regex to strip XML tags: <[^>]*>
        // Note: This is a "Dirty" parser. A proper XML parser is safer but slower/complex.
        let re = Regex::new(r"<[^>]*>").unwrap();
        let text = re.replace_all(&xml_content, " ").to_string();
        let clean_text = text.split_whitespace().collect::<Vec<&str>>().join(" ");
        
        Ok(clean_text)
    }
}

impl FileReader for FsReader {
    /// Reads the file from disk, routing to specific parsers based on extension.
    fn read_file(&self, node: &FileNode) -> FileContext {
        let extension = self.detect_language(&node.path);
        
        let (content, tokens) = match extension.as_str() {
            "pdf" => {
                match self.parse_pdf(&node.path) {
                    Ok(text) => {
                        let count = self.estimate_tokens(&text);
                        (ContentType::Text(text), count)
                    },
                    Err(e) => (ContentType::Error(e.to_string()), 0),
                }
            },
            "docx" => {
                match self.parse_docx(&node.path) {
                    Ok(text) => {
                        let count = self.estimate_tokens(&text);
                        (ContentType::Text(text), count)
                    },
                    Err(e) => (ContentType::Error(e.to_string()), 0),
                }
            },
            _ => {
                // Default: Try to read as plain text
                match fs::read_to_string(&node.path) {
                    Ok(text) => {
                        let count = self.estimate_tokens(&text);
                        (ContentType::Text(text), count)
                    }
                    Err(_) => {
                        // Fallback: Binary
                        (ContentType::Binary, 0)
                    }
                }
            }
        };

        FileContext::new(
            node.path.clone(),
            node.relative_path.clone(),
            content,
            extension,
            tokens,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_read_text_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let node = FileNode::new(file_path.clone(), file_path.clone());
        let reader = FsReader::new();
        let context = reader.read_file(&node);

        match context.content {
            ContentType::Text(s) => assert!(s.contains("fn main")),
            _ => panic!("Should be detected as text"),
        }
    }

    // Note: Testing PDF/DOCX requires creating valid binary files in tests.
    // That is complex without including assets. We skip unit testing binary formats 
    // here and rely on manual verification or integration tests with assets.
}