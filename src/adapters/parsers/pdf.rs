use crate::adapters::parsers::FileParser;
use anyhow::Result;
use regex::Regex;
use std::panic;
use std::path::Path;
use tracing::{debug, warn};

pub struct PdfParser;

impl PdfParser {
    pub fn new() -> Self {
        Self
    }

    /// Advanced cleaning pipeline for PDF text.
    fn sanitize_pdf_text(&self, raw_text: &str) -> String {
        // Step 1: Remove "CID" font artifacts (common in PDF extraction)
        let re_cid = Regex::new(r"\(cid:\d+\)").unwrap();
        let text_no_cid = re_cid.replace_all(raw_text, "");

        // Step 2: Line-by-line semantic filtering
        let lines: Vec<&str> = text_no_cid.lines().collect();
        let mut clean_lines: Vec<String> = Vec::new();

        for line in lines {
            let trimmed = line.trim();

            // FILTER: Skip empty lines
            if trimmed.is_empty() {
                clean_lines.push(String::new());
                continue;
            }

            // FILTER: Pagination noise (e.g., "14", "Page 14", "14 / 200")
            let re_pagination = Regex::new(r"^(?i)(page\s+)?\d+(\s*/\s*\d+)?$").unwrap();
            if re_pagination.is_match(trimmed) {
                continue;
            }

            // FILTER: Very short garbage lines (often OCR artifacts), unless it looks like a bullet point
            if trimmed.len() < 3 && !trimmed.starts_with('-') && !trimmed.starts_with('•') {
                continue;
            }

            clean_lines.push(trimmed.to_string());
        }

        // Step 3: Reconstruct paragraphs (De-hyphenation)
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

        // Step 4: Final whitespace normalization
        let re_spaces = Regex::new(r"[ \t]+").unwrap();
        let final_text = re_spaces.replace_all(&reconstructed, " ");

        // Step 5: Trim limit (just in case)
        final_text.trim().to_string()
    }
}

impl FileParser for PdfParser {
    fn parse(&self, path: &Path) -> Result<String> {
        debug!("Parsing PDF: {:?}", path);

        let path_buf = path.to_path_buf();

        // Catch panics from external library
        let result = panic::catch_unwind(move || {
            let bytes = std::fs::read(&path_buf).expect("File read failed inside thread");
            pdf_extract::extract_text_from_mem(&bytes)
        });

        match result {
            Ok(extraction_result) => match extraction_result {
                Ok(text) => {
                    let clean_text = self.sanitize_pdf_text(&text);
                    Ok(clean_text)
                }
                Err(e) => {
                    warn!("PDF Logic Error: {}", e);
                    Err(anyhow::anyhow!(
                        "PDF Parsing failed (Encrypted or Malformed): {}",
                        e
                    ))
                }
            },
            Err(_) => {
                warn!("PDF Parser PANIC! (Corrupt file?)");
                Err(anyhow::anyhow!(
                    "Critical Failure: PDF Parser crashed on this file."
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_sanitization_advanced() {
        let parser = PdfParser::new();

        // Simulate a messy PDF output
        let raw = "
            This is a com-
            plete sentence.
            
            14
            
            This is paragraph two
            with weird    spacing.
            (cid:123)
            Page 15
        ";

        let clean = parser.sanitize_pdf_text(raw);

        println!("CLEANED: '{}'", clean);

        // 1. Dehyphenation
        assert!(clean.contains("complete sentence"));
        assert!(!clean.contains("com-"));

        // 2. Pagination removal
        assert!(!clean.contains("14"));
        assert!(!clean.contains("Page 15"));

        // 3. CID removal
        assert!(!clean.contains("cid:"));

        // 4. Paragraph preservation
        assert!(clean.contains("\n\nThis is paragraph two"));
    }
}
