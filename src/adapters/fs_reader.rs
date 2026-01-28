use crate::core::content::{ContentType, FileContext};
use crate::core::file::FileNode;
use crate::ports::reader::FileReader;
use std::path::Path;

// Import strategies
use crate::adapters::parsers::FileParser;
use crate::adapters::parsers::pdf::PdfParser;
use crate::adapters::parsers::docx::DocxParser;
use crate::adapters::parsers::fallback::PlainTextParser;

/// Implementation of FileReader that acts as a Router for specific parsers.
pub struct FsReader {
    pdf_parser: PdfParser,
    docx_parser: DocxParser,
    text_parser: PlainTextParser,
}

impl Default for FsReader {
    fn default() -> Self {
        Self::new()
    }
}

impl FsReader {
    /// Creates a new instance of FsReader with all parsers initialized.
    pub fn new() -> Self {
        Self {
            pdf_parser: PdfParser::new(),
            docx_parser: DocxParser::new(),
            text_parser: PlainTextParser::new(),
        }
    }

    /// Infers programming language from extension.
    fn detect_language(&self, path: &Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("text")
            .to_lowercase()
    }

    /// Simple heuristic for token counting.
    fn estimate_tokens(&self, text: &str) -> usize {
        text.len() / 3
    }
}

impl FileReader for FsReader {
    /// Reads the file from disk, routing to specific parsers based on extension.
    fn read_file(&self, node: &FileNode) -> FileContext {
        let extension = self.detect_language(&node.path);
        
        let parser_result = match extension.as_str() {
            "pdf" => self.pdf_parser.parse(&node.path),
            "docx" => self.docx_parser.parse(&node.path),
            _ => self.text_parser.parse(&node.path),
        };

        let (content, tokens) = match parser_result {
            Ok(text) => {
                let count = self.estimate_tokens(&text);
                (ContentType::Text(text), count)
            },
            Err(e) => {
                if extension == "pdf" || extension == "docx" {
                    (ContentType::Error(e.to_string()), 0)
                } else {
                    (ContentType::Binary, 0)
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

    /// Tests that the router correctly handles a text file using the Fallback parser.
    #[test]
    fn test_read_routing_text() {
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
}