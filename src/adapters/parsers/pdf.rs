use crate::adapters::parsers::FileParser;
use anyhow::{Context, Result};
use lopdf::Document;
use regex::Regex;
use std::panic;
use std::path::Path;
use tracing::{debug, warn};

pub struct PdfParser;

impl PdfParser {
    pub fn new() -> Self {
        Self
    }

    /// Tubería de limpieza avanzada para texto extraído de PDF.
    /// Diseñada específicamente para libros técnicos como O'Reilly.
    fn sanitize_pdf_text(&self, raw_text: &str) -> String {
        let re_cid = Regex::new(r"\(cid:\d+\)").unwrap();
        let text_no_cid = re_cid.replace_all(raw_text, "");

        let lines: Vec<&str> = text_no_cid.lines().collect();
        let mut clean_lines: Vec<String> = Vec::with_capacity(lines.len());

        let re_pagination =
            Regex::new(r"^(\d+|[xivXIV]+)(\s*\|\s*.*)?$|^(.*\|\s*)?(\d+|[xivXIV]+)$").unwrap();

        for line in lines {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if let Some(last) = clean_lines.last() {
                    if !last.is_empty() {
                        clean_lines.push(String::new());
                    }
                }
                continue;
            }

            if re_pagination.is_match(trimmed) {
                if trimmed.len() < 40 {
                    continue;
                }
            }

            if trimmed.len() < 3 && !trimmed.starts_with('-') && !trimmed.starts_with('•') {
                continue;
            }

            clean_lines.push(trimmed.to_string());
        }

        let mut reconstructed = String::new();
        let mut iter = clean_lines.iter().peekable();

        while let Some(line) = iter.next() {
            if line.is_empty() {
                reconstructed.push_str("\n\n");
                continue;
            }

            if line.ends_with('-') {
                let stripped = &line[..line.len() - 1];
                reconstructed.push_str(stripped);
            } else {
                reconstructed.push_str(line);
                if let Some(next) = iter.peek() {
                    if !next.is_empty() {
                        reconstructed.push(' ');
                    }
                }
            }
        }

        let re_spaces = Regex::new(r"[ \t]+").unwrap();
        let final_text = re_spaces.replace_all(&reconstructed, " ");

        let re_vertical = Regex::new(r"\n{3,}").unwrap();
        let final_text_squashed = re_vertical.replace_all(&final_text, "\n\n");

        final_text_squashed.trim().to_string()
    }
}

impl FileParser for PdfParser {
    fn parse(&self, path: &Path) -> Result<String> {
        debug!("Parsing PDF using lopdf: {:?}", path);
        let path_buf = path.to_path_buf();

        let result = panic::catch_unwind(move || {
            let doc = Document::load(&path_buf).context("Failed to load PDF document")?;
            let pages = doc.get_pages();
            let mut full_text = String::new();

            let mut page_numbers: Vec<_> = pages.keys().cloned().collect();
            page_numbers.sort();

            for page_num in page_numbers {
                match doc.extract_text(&[page_num]) {
                    Ok(text) => {
                        full_text.push_str(&text);
                        full_text.push('\n');
                    }
                    Err(e) => {
                        warn!("Skipping page {} due to extraction error: {}", page_num, e);
                    }
                }
            }
            Ok(full_text)
        });

        match result {
            Ok(extraction_result) => match extraction_result {
                Ok(text) => {
                    let clean_text = self.sanitize_pdf_text(&text);
                    Ok(clean_text)
                }
                Err(e) => Err(e),
            },
            Err(_) => Err(anyhow::anyhow!(
                "Critical: PDF Parser panicked. File might be corrupt or encrypted."
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_cleaning_logic() {
        let parser = PdfParser::new();

        let raw_input = "
        This is a sentence that is cut-
        off in the middle.
        
        14 | Chapter 1
        
        Here is another paragraph
        with    too    many spaces.
        
        
        
        And a huge vertical gap.
        ";

        let cleaned = parser.sanitize_pdf_text(raw_input);

        // 1. De-hyphenation
        assert!(cleaned.contains("cutoff in the middle"));

        // 2. Pagination removal
        assert!(!cleaned.contains("14 | Chapter 1"));

        // 3. Space normalization
        assert!(cleaned.contains("with too many spaces"));

        // 4. Vertical squash
        assert!(!cleaned.contains("\n\n\n"));

        println!("Cleaned Output:\n{}", cleaned);
    }
}
