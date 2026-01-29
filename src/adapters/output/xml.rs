use anyhow::Result;
use chrono::Local;
use quick_xml::events::{BytesCData, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use crate::core::config::ContextConfig;
use crate::core::content::{minify_content, ContentType, FileContext};
use crate::ports::writer::ContextWriter;

/// Internal struct to represent the directory tree in memory before printing.
#[derive(Default)]
struct TreeNode {
    is_file: bool,
    children: BTreeMap<String, TreeNode>,
}

impl TreeNode {
    /// Inserts a path into the tree structure.
    fn insert(&mut self, path: &Path) {
        let mut current_node = self;
        for component in path.components() {
            let key = component.as_os_str().to_string_lossy().to_string();
            current_node = current_node
                .children
                .entry(key)
                .or_insert_with(TreeNode::default);
        }
        current_node.is_file = true;
    }

    /// Recursively renders the tree to a string.
    fn render(&self, prefix: &str, buffer: &mut String) {
        let count = self.children.len();
        for (i, (name, node)) in self.children.iter().enumerate() {
            let is_last = i == count - 1;

            let connector = if is_last { "└── " } else { "├── " };

            buffer.push_str(&format!("{}{}{}\n", prefix, connector, name));

            let new_prefix = if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            node.render(&new_prefix, buffer);
        }
    }
}

/// Implementation of ContextWriter that outputs XML format.
#[derive(Default)]
pub struct XmlWriter;

impl XmlWriter {
    /// Creates a new XmlWriter.
    pub fn new() -> Self {
        Self
    }

    /// Generates a recursive ASCII tree representation.
    fn generate_tree(&self, files: &[FileContext], root_name: &str) -> String {
        let mut root_node = TreeNode::default();

        for file in files {
            root_node.insert(&file.relative_path);
        }

        let mut output = String::new();
        output.push_str(&format!("{}\n", root_name));
        root_node.render("", &mut output);
        output
    }

    /// Sanitizes content to be safely included in CDATA blocks.
    fn sanitize_content(&self, content: &str) -> String {
        if content.contains("]]]]><![CDATA[>") {
            content.replace("]]]]><![CDATA[>", "]]]]]]><![CDATA[><![CDATA[>")
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

        xml_writer.write_event(Event::Start(BytesStart::new("context")))?;

        // 1. Metadata
        xml_writer.write_event(Event::Start(BytesStart::new("metadata")))?;
        xml_writer
            .create_element("project_root")
            .write_text_content(BytesText::new(&config.root_path.to_string_lossy()))?;
        xml_writer
            .create_element("scan_time")
            .write_text_content(BytesText::new(&Local::now().to_rfc3339()))?;

        xml_writer.write_event(Event::Start(BytesStart::new("stats")))?;
        xml_writer
            .create_element("total_files")
            .write_text_content(BytesText::new(&files.len().to_string()))?;
        let total_tokens: usize = files.iter().map(|f| f.token_count).sum();
        xml_writer
            .create_element("total_tokens")
            .write_text_content(BytesText::new(&total_tokens.to_string()))?;
        xml_writer.write_event(Event::End(BytesEnd::new("stats")))?;

        let root_name = config
            .root_path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|| ".".into());
        let tree_view = self.generate_tree(files, &root_name);
        xml_writer
            .create_element("directory_structure")
            .write_text_content(BytesText::new(&tree_view))?;

        xml_writer.write_event(Event::End(BytesEnd::new("metadata")))?;

        // 2. Files
        xml_writer.write_event(Event::Start(BytesStart::new("files")))?;

        for file in files {
            let mut elem = BytesStart::new("file");
            elem.push_attribute(("path", file.relative_path.to_string_lossy().as_ref()));
            elem.push_attribute(("language", file.language.as_str()));

            xml_writer.write_event(Event::Start(elem))?;

            match &file.content {
                ContentType::Text(text) => {
                    let processed = if config.minify {
                        minify_content(text, &file.language)
                    } else {
                        text.clone()
                    };

                    let sanitized = self.sanitize_content(&processed);
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

    /// Tests that the tree generation produces expected ASCII structure.
    #[test]
    fn test_recursive_tree_generation() {
        let files = vec![
            FileContext::new(
                PathBuf::from("src/main.rs"),
                PathBuf::from("src/main.rs"),
                ContentType::Text("".into()),
                "rs".into(),
                0,
            ),
            FileContext::new(
                PathBuf::from("src/core/mod.rs"),
                PathBuf::from("src/core/mod.rs"),
                ContentType::Text("".into()),
                "rs".into(),
                0,
            ),
            FileContext::new(
                PathBuf::from("Cargo.toml"),
                PathBuf::from("Cargo.toml"),
                ContentType::Text("".into()),
                "toml".into(),
                0,
            ),
        ];

        let writer = XmlWriter::new();
        let tree = writer.generate_tree(&files, "my_project");

        println!("Generated tree:\n{}", tree);

        // Verification logic based on alphabetical sorting:
        // 1. Cargo.toml (First -> ├──)
        // 2. src        (Last  -> └──)
        //    Inside src:
        //    1. core    (First -> ├──)
        //    2. main.rs (Last  -> └──)

        assert!(tree.contains("my_project"));

        // Check Cargo.toml (Top level, first)
        assert!(tree.contains("├── Cargo.toml"));

        // Check src (Top level, last)
        assert!(tree.contains("└── src"));

        // Check nesting
        assert!(tree.contains("    ├── core")); // Indented child of src
        assert!(tree.contains("    └── main.rs")); // Indented last child of src
    }
}
