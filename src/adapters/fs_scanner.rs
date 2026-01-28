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

    /// Checks if a file path passes the extension filters defined in config.
    fn matches_extension_filter(path: &Path, config: &ContextConfig) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 1. Blacklist check (Exclude)
        if !config.exclude_extensions.is_empty() && config.exclude_extensions.contains(&ext) {
            return false;
        }

        // 2. Whitelist check (Include)
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

                    // Apply Extension Filter
                    if !Self::matches_extension_filter(path, config) {
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
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_scan_extension_filtering() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        // Setup:
        // root/
        //   main.rs
        //   script.py
        //   data.json
        //   README.md

        File::create(root.join("main.rs"))?;
        File::create(root.join("script.py"))?;
        File::create(root.join("data.json"))?;
        File::create(root.join("README.md"))?;

        let scanner = FsScanner::new();

        // Case 1: Whitelist "rs" and "py"
        let mut config_inc = ContextConfig::default();
        config_inc.root_path = root.to_path_buf();
        config_inc.include_extensions.insert("rs".into());
        config_inc.include_extensions.insert("py".into());

        let results_inc = scanner.scan(&config_inc)?;
        let paths_inc: Vec<_> = results_inc
            .iter()
            .map(|f| f.relative_path.to_str().unwrap())
            .collect();

        assert!(paths_inc.contains(&"main.rs"));
        assert!(paths_inc.contains(&"script.py"));
        assert!(!paths_inc.contains(&"README.md"));
        assert!(!paths_inc.contains(&"data.json"));

        // Case 2: Blacklist "json"
        let mut config_exc = ContextConfig::default();
        config_exc.root_path = root.to_path_buf();
        config_exc.exclude_extensions.insert("json".into());

        let results_exc = scanner.scan(&config_exc)?;
        let paths_exc: Vec<_> = results_exc
            .iter()
            .map(|f| f.relative_path.to_str().unwrap())
            .collect();

        assert!(paths_exc.contains(&"main.rs"));
        assert!(paths_exc.contains(&"README.md"));
        assert!(!paths_exc.contains(&"data.json"));

        Ok(())
    }
}
