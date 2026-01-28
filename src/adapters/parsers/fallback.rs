use crate::adapters::parsers::FileParser;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct PlainTextParser;

impl PlainTextParser {
    pub fn new() -> Self {
        Self
    }
}

impl FileParser for PlainTextParser {
    fn parse(&self, path: &Path) -> Result<String> {
        let text = fs::read_to_string(path)?;
        Ok(text)
    }
}
