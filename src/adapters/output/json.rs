use anyhow::Result;
use chrono::Local;
use serde::Serialize;
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use crate::core::config::ContextConfig;
use crate::core::content::{minify_content, ContentType, FileContext};
use crate::ports::writer::ContextWriter;

#[derive(Serialize)]
struct JsonReport<'a> {
    metadata: JsonMetadata,
    files: &'a [FileContext],
}

#[derive(Serialize)]
struct JsonMetadata {
    project_root: String,
    scan_time: String,
    stats: JsonStats,
    directory_tree: String,
}

#[derive(Serialize)]
struct JsonStats {
    total_files: usize,
    total_tokens: usize,
}

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

#[derive(Default)]
pub struct JsonWriter;

impl JsonWriter {
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

impl ContextWriter for JsonWriter {
    fn write<W: Write>(
        &self,
        files: &[FileContext],
        config: &ContextConfig,
        writer: W,
    ) -> Result<()> {
        let total_tokens: usize = files.iter().map(|f| f.token_count).sum();
        let root_name = config
            .root_path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_else(|| ".".into());

        let processed_files: Vec<FileContext> = if config.minify {
            files
                .iter()
                .map(|f| {
                    let mut new_f = f.clone();
                    if let ContentType::Text(ref t) = f.content {
                        new_f.content = ContentType::Text(minify_content(t, &f.language));
                    }
                    new_f
                })
                .collect()
        } else {
            files.to_vec()
        };

        let report = JsonReport {
            metadata: JsonMetadata {
                project_root: config.root_path.to_string_lossy().to_string(),
                scan_time: Local::now().to_rfc3339(),
                stats: JsonStats {
                    total_files: files.len(),
                    total_tokens,
                },
                directory_tree: self.generate_tree(files, &root_name),
            },
            files: &processed_files,
        };

        serde_json::to_writer_pretty(writer, &report)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::content::ContentType;
    use std::path::PathBuf;

    #[test]
    fn test_json_output() {
        let config = ContextConfig::default();
        let files = vec![FileContext::new(
            PathBuf::from("main.rs"),
            PathBuf::from("main.rs"),
            ContentType::Text("code".into()),
            "rust".into(),
            10,
        )];

        let writer = JsonWriter::new();
        let mut buffer = Vec::new();

        writer
            .write(&files, &config, &mut buffer)
            .expect("Should write JSON");
        let output = String::from_utf8(buffer).expect("Valid UTF-8");

        assert!(output.contains("\"project_root\": \".\""));
        assert!(output.contains("\"type\": \"Text\""));
        assert!(output.contains("\"data\": \"code\""));
    }
}
