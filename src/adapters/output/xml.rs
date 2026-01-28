use anyhow::Result;
use chrono::Local;
use quick_xml::events::{BytesCData, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::io::Write;

use crate::core::config::ContextConfig;
use crate::core::content::{ContentType, FileContext};
use crate::ports::writer::ContextWriter;

/// Implementation of ContextWriter that outputs XML format.
#[derive(Default)]
pub struct XmlWriter;

impl XmlWriter {
    /// Creates a new XmlWriter.
    pub fn new() -> Self {
        Self
    }

    /// Helper to generate a simple ASCII tree representation.
    fn generate_tree(&self, files: &[FileContext]) -> String {
        let mut tree = String::from(".\n");
        for file in files {
            tree.push_str(&format!("├── {}\n", file.relative_path.display()));
        }
        tree
    }

    /// Sanitizes content to be safely included in CDATA blocks.
    /// Replaces "]]>" with "]]]]><![CDATA[>" to avoid breaking XML structure.
    fn sanitize_content(&self, content: &str) -> String {
        if content.contains("]]>") {
            content.replace("]]>", "]]]]><![CDATA[>")
        } else {
            content.to_string()
        }
    }
}

impl ContextWriter for XmlWriter {
    fn write<W: Write>(
        &self,
        files: &[FileContext],
        config: &ContextConfig,
        writer: W,
    ) -> Result<()> {
        let mut xml_writer = Writer::new_with_indent(writer, b' ', 4);

        // Root Element <context>
        xml_writer.write_event(Event::Start(BytesStart::new("context")))?;

        // 1. Metadata
        xml_writer.write_event(Event::Start(BytesStart::new("metadata")))?;

        // <project_root>
        xml_writer
            .create_element("project_root")
            .write_text_content(BytesText::new(&config.root_path.to_string_lossy()))?;

        // <scan_time>
        xml_writer
            .create_element("scan_time")
            .write_text_content(BytesText::new(&Local::now().to_rfc3339()))?;

        // <stats>
        xml_writer.write_event(Event::Start(BytesStart::new("stats")))?;
        xml_writer
            .create_element("total_files")
            .write_text_content(BytesText::new(&files.len().to_string()))?;

        let total_tokens: usize = files.iter().map(|f| f.token_count).sum();
        xml_writer
            .create_element("total_tokens")
            .write_text_content(BytesText::new(&total_tokens.to_string()))?;
        xml_writer.write_event(Event::End(BytesEnd::new("stats")))?;

        // <directory_structure> (Tree)
        let tree_view = self.generate_tree(files);
        xml_writer
            .create_element("directory_structure")
            .write_text_content(BytesText::new(&tree_view))?;

        xml_writer.write_event(Event::End(BytesEnd::new("metadata")))?;

        // 2. Files
        xml_writer.write_event(Event::Start(BytesStart::new("files")))?;

        for file in files {
            let mut elem = BytesStart::new("file");
            // Fix borrow warning: explicit casting or handling of the Cow str
            elem.push_attribute(("path", file.relative_path.to_string_lossy().as_ref()));
            elem.push_attribute(("language", file.language.as_str()));

            xml_writer.write_event(Event::Start(elem))?;

            match &file.content {
                ContentType::Text(text) => {
                    let sanitized = self.sanitize_content(text);
                    xml_writer.write_event(Event::CData(BytesCData::new(&sanitized)))?;
                }
                ContentType::Binary => {
                    xml_writer
                        .write_event(Event::CData(BytesCData::new("[BINARY CONTENT SKIPPED]")))?;
                }
                ContentType::Error(e) => {
                    xml_writer.write_event(Event::CData(BytesCData::new(format!(
                        "[ERROR READING FILE: {}]",
                        e
                    ))))?;
                }
            }

            xml_writer.write_event(Event::End(BytesEnd::new("file")))?;
        }

        xml_writer.write_event(Event::End(BytesEnd::new("files")))?;
        xml_writer.write_event(Event::End(BytesEnd::new("context")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Tests XML generation with a mock file list.
    #[test]
    fn test_xml_output_structure() {
        let config = ContextConfig::default();
        let files = vec![FileContext::new(
            PathBuf::from("src/main.rs"),
            PathBuf::from("src/main.rs"),
            ContentType::Text("fn main() {}".to_string()),
            "rust".to_string(),
            10,
        )];

        let writer = XmlWriter::new();
        let mut buffer = Vec::new();

        writer
            .write(&files, &config, &mut buffer)
            .expect("Should write XML");

        let output = String::from_utf8(buffer).expect("Valid UTF-8");

        assert!(output.contains("<context>"));
        assert!(output.contains("<file path=\"src/main.rs\" language=\"rust\">"));
        assert!(output.contains("<![CDATA[fn main() {}]]>"));
        assert!(output.contains("directory_structure"));
    }

    /// Tests that CDATA is sanitized if it contains "]]>".
    #[test]
    fn test_xml_sanitization() {
        let config = ContextConfig::default();
        let malicious_content = "code with ]]> in it";
        let files = vec![FileContext::new(
            PathBuf::from("bad.rs"),
            PathBuf::from("bad.rs"),
            ContentType::Text(malicious_content.to_string()),
            "rust".to_string(),
            5,
        )];

        let writer = XmlWriter::new();
        let mut buffer = Vec::new();
        writer.write(&files, &config, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(!output.contains("]]>]]>")); // Should not contain raw break
        assert!(output.contains("]]]]><![CDATA[>")); // Should contain escaped version
    }
}
