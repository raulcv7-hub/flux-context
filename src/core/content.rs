use serde::Serialize;
use std::path::PathBuf;

/// Enum representing the type of content found in a file.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ContentType {
    Text(String),
    Binary,
    Error(String),
}

/// Domain entity representing a processed file with its content and metadata.
#[derive(Debug, Clone, Serialize)]
pub struct FileContext {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub content: ContentType,
    pub language: String,
    pub token_count: usize,
}

impl FileContext {
    pub fn new(
        path: PathBuf,
        relative_path: PathBuf,
        content: ContentType,
        language: String,
        token_count: usize,
    ) -> Self {
        Self {
            path,
            relative_path,
            content,
            language,
            token_count,
        }
    }
}

/// Aggressively reduces content size.
pub fn minify_content(content: &str, language: &str) -> String {
    let indent_sensitive = ["py", "python", "yaml", "yml", "md", "markdown"];

    let is_sensitive = indent_sensitive.contains(&language.to_lowercase().as_str());

    let lines = content.lines();
    let mut minified = String::with_capacity(content.len());

    for line in lines {
        // 1. Remove Trailing Spaces (Manual trim is faster than Regex for single line)
        let trimmed_end = line.trim_end();

        // 2. Remove Empty Lines
        if trimmed_end.is_empty() {
            continue;
        }

        // 3. Strip Indentation (Only for non-sensitive languages)
        let final_line = if is_sensitive {
            trimmed_end
        } else {
            trimmed_end.trim_start()
        };

        minified.push_str(final_line);
        minified.push('\n');
    }

    minified
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify_aggressive_rust() {
        let input = r#"
fn main() {
    let x = 5;

    if x > 0 {
        println!("Hello");
    }
}
"#;
        let expected = "fn main() {\nlet x = 5;\nif x > 0 {\nprintln!(\"Hello\");\n}\n}\n";
        let result = minify_content(input, "rs");
        assert_eq!(result, expected);
    }
}
