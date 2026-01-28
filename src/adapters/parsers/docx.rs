use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use regex::Regex;
use crate::adapters::parsers::FileParser;

pub struct DocxParser;

impl DocxParser {
    pub fn new() -> Self {
        Self
    }
}

impl FileParser for DocxParser {
    fn parse(&self, path: &Path) -> Result<String> {
        let file = File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        let mut document_xml = archive.by_name("word/document.xml")
            .with_context(|| "Could not find word/document.xml in docx")?;
        
        let mut xml_content = String::new();
        document_xml.read_to_string(&mut xml_content)?;

        // Regex to strip XML tags
        let re = Regex::new(r"<[^>]*>").unwrap();
        let text = re.replace_all(&xml_content, " ").to_string();
        let clean_text = text.split_whitespace().collect::<Vec<&str>>().join(" ");
        
        Ok(clean_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_docx_asset() {
        let path = PathBuf::from("tests/assets/test.docx");
        if !path.exists() {
            println!("SKIPPING: DOCX test asset not found at {:?}", path);
            return;
        }

        let parser = DocxParser::new();
        let result = parser.parse(&path).expect("Should parse DOCX");
        assert!(!result.is_empty());
    }
}