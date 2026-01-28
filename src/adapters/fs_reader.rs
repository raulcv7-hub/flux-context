use crate::core::content::{ContentType, FileContext};
use crate::core::file::FileNode;
use crate::ports::reader::FileReader;
use std::fs;
use std::path::Path;

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
}

impl FileReader for FsReader {
    /// Reads the file from disk, handling text/binary distinction.
    fn read_file(&self, node: &FileNode) -> FileContext {
        let language = self.detect_language(&node.path);

        // Attempt to read file as String
        let (content, tokens) = match fs::read_to_string(&node.path) {
            Ok(text) => {
                let count = self.estimate_tokens(&text);
                (ContentType::Text(text), count)
            }
            Err(_) => {
                // If read_to_string fails, it's likely binary or permission error.
                // We assume binary for now to be safe.
                (ContentType::Binary, 0)
            }
        };

        FileContext::new(
            node.path.clone(),
            node.relative_path.clone(),
            content,
            language,
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

    /// Tests reading a valid text file.
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
        assert_eq!(context.language, "rs");
    }

    /// Tests behavior on non-utf8 files (simulated binary).
    #[test]
    fn test_read_binary_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("image.png");
        let mut file = File::create(&file_path).unwrap();
        // Write invalid UTF-8 bytes
        file.write_all(&[0x80, 0x81, 0xFF]).unwrap();

        let node = FileNode::new(file_path.clone(), file_path.clone());
        let reader = FsReader::new();
        let context = reader.read_file(&node);

        match context.content {
            ContentType::Binary => assert!(true),
            _ => panic!("Should be detected as binary"),
        }
        assert_eq!(context.language, "png");
    }
}
