use anyhow::Result;
use chrono::Local;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use crate::core::config::ContextConfig;
use crate::core::content::{ContentType, FileContext, minify_content};
use crate::ports::writer::ContextWriter;

// --- Helper Logic for Tree (Same as others) ---
#[derive(Default)]
struct TreeNode {
    children: BTreeMap<String, TreeNode>,
}

impl TreeNode {
    fn insert(&mut self, path: &Path) {
        let mut current = self;
        for component in path.components() {
            let key = component.as_os_str().to_string_lossy().to_string();
            current = current.children.entry(key).or_default();
        }
    }

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

// --- Text Adapter Implementation ---

#[derive(Default)]
pub struct TextWriter;

impl TextWriter {
    pub fn new() -> Self {
        Self
    }

    fn generate_tree(&self, files: &[FileContext], root_name: &str) -> String {
        let mut root = TreeNode::default();
        for file in files {
            root.insert(&file.relative_path);
        }
        let mut output = String::new();
        output.push_str(&format!("{}\n", root_name));
        root.render("", &mut output);
        output
    }
}

impl ContextWriter for TextWriter {
    fn write<W: Write>(
        &self,
        files: &[FileContext],
        config: &ContextConfig,
        mut writer: W,
    ) -> Result<()> {
        let separator = "=".repeat(80);
        let sub_separator = "-".repeat(80);

        // 1. Header & Metadata
        writeln!(writer, "{}", separator)?;
        writeln!(writer, "PROJECT CONTEXT REPORT")?;
        writeln!(writer, "{}", separator)?;
        writeln!(writer, "Generated Date: {}", Local::now().format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(writer, "Project Root:   {}", config.root_path.display())?;
        
        let total_tokens: usize = files.iter().map(|f| f.token_count).sum();
        writeln!(writer, "Total Files:    {}", files.len())?;
        writeln!(writer, "Total Tokens:   {} (Estimated)", total_tokens)?;
        writeln!(writer, "\n")?;

        // 2. Directory Structure
        writeln!(writer, "DIRECTORY STRUCTURE")?;
        writeln!(writer, "{}", sub_separator)?;
        let root_name = config.root_path.file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|| ".".into());
            
        writeln!(writer, "{}", self.generate_tree(files, &root_name).trim_end())?;
        writeln!(writer, "\n\n")?;

        // 3. File Contents
        writeln!(writer, "FILE CONTENTS")?;
        writeln!(writer, "{}", separator)?;

        for file in files {
            writeln!(writer, "\nFILE: {}", file.relative_path.display())?;
            writeln!(writer, "LANGUAGE: {}", file.language)?;
            writeln!(writer, "{}", sub_separator)?;
            
            match &file.content {
                ContentType::Text(text) => {
                    let processed = if config.minify {
                        minify_content(text)
                    } else {
                        text.to_string()
                    };
                    writeln!(writer, "{}", processed)?;
                },
                ContentType::Binary => {
                    writeln!(writer, "[BINARY CONTENT SKIPPED]")?;
                },
                ContentType::Error(e) => {
                    writeln!(writer, "[ERROR READING FILE: {}]", e)?;
                }
            }
            writeln!(writer, "{}", sub_separator)?;
        }
        writeln!(writer, "END OF REPORT")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_text_output_structure() {
        let config = ContextConfig::default();
        let files = vec![
            FileContext::new(
                PathBuf::from("src/main.rs"),
                PathBuf::from("src/main.rs"),
                ContentType::Text("fn main() {}".into()),
                "rust".into(),
                10
            )
        ];

        let writer = TextWriter::new();
        let mut buffer = Vec::new();
        
        writer.write(&files, &config, &mut buffer).expect("Should write Text");
        let output = String::from_utf8(buffer).expect("Valid UTF-8");

        assert!(output.contains("PROJECT CONTEXT REPORT"));
        assert!(output.contains("DIRECTORY STRUCTURE"));
        assert!(output.contains("FILE: src/main.rs"));
        assert!(output.contains("fn main() {}"));
    }
}