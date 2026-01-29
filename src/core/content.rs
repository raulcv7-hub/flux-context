use regex::Regex;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::OnceLock;

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
    /// Creates a new FileContext.
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

static RE_NEWLINES: OnceLock<Regex> = OnceLock::new();
static RE_TRAILING_SPACES: OnceLock<Regex> = OnceLock::new();

/// Reduces whitespace to save tokens.
/// 1. Trims start/end of file.
/// 2. Removes trailing spaces from every line.
/// 3. Collapses 3+ newlines into 2 (Paragraph breaks).
pub fn minify_content(content: &str) -> String {
    let re_trailing = RE_TRAILING_SPACES.get_or_init(|| Regex::new(r"(?m)[ \t]+$").unwrap());
    let no_trailing = re_trailing.replace_all(content, "");

    let re_newlines = RE_NEWLINES.get_or_init(|| Regex::new(r"\n{3,}").unwrap());
    let collapsed = re_newlines.replace_all(&no_trailing, "\n\n");

    collapsed.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minify_content() {
        let input = "Line 1    \n\n\n\nLine 2 \t \n\nLine 3";
        let expected = "Line 1\n\nLine 2\n\nLine 3";
        assert_eq!(minify_content(input), expected);
    }
}
