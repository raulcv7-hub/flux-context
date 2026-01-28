use anyhow::Result;
use ignore::{DirEntry, WalkBuilder};
use std::path::Path;
use tracing::{debug, warn};

use crate::core::config::ContextConfig;
use crate::core::file::FileNode;
use crate::ports::scanner::ProjectScanner;

#[derive(Default)]
pub struct FsScanner;

impl FsScanner {
    pub fn new() -> Self {
        Self
    }

    fn is_noise(entry: &DirEntry) -> bool {
        let file_name = entry.file_name().to_string_lossy();

        const NOISE_FILES: &[&str] = &[
            "Cargo.lock",
            "package-lock.json",
            "yarn.lock",
            "pnpm-lock.yaml",
            "poetry.lock",
            "Pipfile.lock",
            ".DS_Store",
            "Thumbs.db",
        ];

        const NOISE_DIRS: &[&str] = &[
            ".git",
            ".svn",
            ".hg",
            "target",
            "build",
            "dist",
            "node_modules",
            "__pycache__",
            ".venv",
            "venv",
            ".idea",
            ".vscode",
        ];

        if NOISE_FILES.contains(&file_name.as_ref()) {
            return true;
        }

        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false)
            && NOISE_DIRS.contains(&file_name.as_ref())
        {
            return true;
        }

        false
    }

    /// Checks filters: Extensions and Paths.
    fn matches_filters(path: &Path, config: &ContextConfig) -> bool {
        let path_str = path.to_string_lossy();

        // 1. Path Filters
        // Exclude wins over include
        if !config.exclude_paths.is_empty() {
            for exclude in &config.exclude_paths {
                if path_str.contains(exclude) {
                    return false;
                }
            }
        }

        if !config.include_paths.is_empty() {
            let mut matched = false;
            for include in &config.include_paths {
                if path_str.contains(include) {
                    matched = true;
                    break;
                }
            }
            if !matched {
                return false;
            }
        }

        // 2. Extension Filters
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !config.exclude_extensions.is_empty() && config.exclude_extensions.contains(&ext) {
            return false;
        }

        if !config.include_extensions.is_empty() && !config.include_extensions.contains(&ext) {
            return false;
        }

        true
    }
}

impl ProjectScanner for FsScanner {
    fn scan(&self, config: &ContextConfig) -> Result<Vec<FileNode>> {
        let root = &config.root_path;
        debug!("Starting scan at: {:?} with filters", root);

        let mut builder = WalkBuilder::new(root);

        builder
            .standard_filters(true)
            .hidden(!config.include_hidden);

        if let Some(depth) = config.max_depth {
            builder.max_depth(Some(depth));
        }

        builder.filter_entry(|entry| !Self::is_noise(entry));

        let mut files = Vec::new();

        for result in builder.build() {
            match result {
                Ok(entry) => {
                    if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        continue;
                    }

                    let path = entry.path();

                    // Apply All Filters
                    if !Self::matches_filters(path, config) {
                        continue;
                    }

                    let path_buf = path.to_path_buf();
                    let relative_path = match path_buf.strip_prefix(root) {
                        Ok(p) => p.to_path_buf(),
                        Err(_) => path_buf.clone(),
                    };

                    files.push(FileNode::new(path_buf, relative_path));
                }
                Err(err) => {
                    warn!("Skipping file due to error: {}", err);
                }
            }
        }
        files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

        debug!("Scan complete. Found {} files.", files.len());
        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;
    use std::path::PathBuf; // Import necesario

    #[test]
    fn test_scan_path_filtering() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        // Structure:
        // src/main.rs
        // src/adapters/mod.rs
        // docs/info.md
        
        fs::create_dir(root.join("src"))?;
        fs::create_dir(root.join("src/adapters"))?;
        fs::create_dir(root.join("docs"))?;

        File::create(root.join("src/main.rs"))?;
        File::create(root.join("src/adapters/mod.rs"))?;
        File::create(root.join("docs/info.md"))?;

        let scanner = FsScanner::new();

        // Construimos las rutas esperadas dinámicamente según el SO (Windows/Linux)
        let main_rs = PathBuf::from("src").join("main.rs").to_string_lossy().to_string();
        let adapters_mod = PathBuf::from("src").join("adapters").join("mod.rs").to_string_lossy().to_string();
        let docs_info = PathBuf::from("docs").join("info.md").to_string_lossy().to_string();

        // 1. Include "src"
        let mut config_src = ContextConfig::default();
        config_src.root_path = root.to_path_buf();
        config_src.include_paths.push("src".into());

        let res_src = scanner.scan(&config_src)?;
        let paths: Vec<_> = res_src.iter().map(|f| f.relative_path.to_string_lossy().to_string()).collect();
        
        assert!(paths.contains(&main_rs), "Expected {:?} in {:?}", main_rs, paths);
        assert!(paths.contains(&adapters_mod)); 
        assert!(!paths.contains(&docs_info));

        // 2. Exclude "adapters"
        let mut config_no_adapt = ContextConfig::default();
        config_no_adapt.root_path = root.to_path_buf();
        config_no_adapt.exclude_paths.push("adapters".into());

        let res_no = scanner.scan(&config_no_adapt)?;
        let paths_no: Vec<_> = res_no.iter().map(|f| f.relative_path.to_string_lossy().to_string()).collect();

        assert!(paths_no.contains(&main_rs));
        assert!(paths_no.contains(&docs_info));
        assert!(!paths_no.contains(&adapters_mod));

        Ok(())
    }
}