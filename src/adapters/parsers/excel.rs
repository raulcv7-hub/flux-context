use anyhow::{Context, Result};
use calamine::{Reader, Xlsx, open_workbook, Data};
use std::path::Path;
use crate::adapters::parsers::FileParser;

pub struct ExcelParser;

impl ExcelParser {
    pub fn new() -> Self {
        Self
    }
}

impl FileParser for ExcelParser {
    fn parse(&self, path: &Path) -> Result<String> {
        let mut workbook: Xlsx<_> = open_workbook(path)
            .with_context(|| "Cannot open Excel file")?;

        let mut output = String::new();

        for sheet_name in workbook.sheet_names().to_owned() {
            output.push_str(&format!("\n--- Sheet: {} ---\n", sheet_name));
            
            if let Ok(range) = workbook.worksheet_range(&sheet_name) {
                for row in range.rows() {
                    let row_str: Vec<String> = row.iter().map(|c| match c {
                        Data::String(s) => s.to_string(),
                        Data::Float(f) => f.to_string(),
                        Data::Int(i) => i.to_string(),
                        Data::Bool(b) => b.to_string(),
                        Data::Error(e) => format!("ERR: {:?}", e),
                        Data::Empty => "".to_string(),
                        _ => "".to_string(), 
                    }).collect();
                    
                    output.push_str(&row_str.join(" | "));
                    output.push('\n');
                }
            }
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_excel_asset() {
        let path = PathBuf::from("tests/assets/test.xlsx");
        if !path.exists() {
            println!("SKIPPING: Excel test asset not found at {:?}", path);
            return;
        }

        let parser = ExcelParser::new();
        let result = parser.parse(&path).expect("Should parse Excel");
        assert!(!result.is_empty());
    }
}