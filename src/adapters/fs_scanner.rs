use anyhow::{Result};
use ignore::{DirEntry, WalkBuilder};
use tracing::{debug, warn};
use crate::core::config::ContextConfig;
use crate::core::file::FileNode;
use crate::ports::scanner::ProjectScanner;

/// Implementation of ProjectScanner using the 'ignore' crate (ripgrep engine).
#[derive(Default)]
pub struct FsScanner;

impl FsScanner {
    /// Creates a new instance of FsScanner.
    pub fn new() -> Self {
        Self
    }

    /// Determines if a specific entry should be ignored based on business rules
    /// (e.g., Lockfiles, noisy directories) regardless of gitignore.
    fn is_noise(entry: &DirEntry) -> bool {
        let file_name = entry.file_name().to_string_lossy();
        
        // Nivel 1 & 2: Hard Ignores & Ecosystem Noise
        const NOISE_FILES: &[&str] = &[
            // Rust
            "Cargo.lock",
            // JS/Node
            "package-lock.json", "yarn.lock", "pnpm-lock.yaml",
            // Python
            "poetry.lock", "Pipfile.lock",
            // System
            ".DS_Store", "Thumbs.db"
        ];

        const NOISE_DIRS: &[&str] = &[
            ".git", ".svn", ".hg",
            "target", "build", "dist",
            "node_modules",
            "__pycache__", ".venv", "venv",
            ".idea", ".vscode"
        ];

        if NOISE_FILES.contains(&file_name.as_ref()) {
            return true;
        }

        // Only check directory noise if it IS a directory to avoid false positives
        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) 
            && NOISE_DIRS.contains(&file_name.as_ref()) {
            return true;
        }

        false
    }
}

impl ProjectScanner for FsScanner {
    /// Scans the filesystem based on the provided configuration.
    fn scan(&self, config: &ContextConfig) -> Result<Vec<FileNode>> {
        let root = &config.root_path;
        debug!("Starting scan at: {:?} with depth: {:?}", root, config.max_depth);

        let mut builder = WalkBuilder::new(root);
        
        builder.standard_filters(true)
               .hidden(!config.include_hidden); 

        if let Some(depth) = config.max_depth {
            builder.max_depth(Some(depth));
        }

        builder.filter_entry(|entry| !Self::is_noise(entry));

        let mut files = Vec::new();

        for result in builder.build() {
            match result {
                Ok(entry) => {
                    // We only care about files for the final list
                    if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        continue;
                    }

                    let path = entry.path().to_path_buf();
                    
                    // Calculate relative path for display
                    let relative_path = match path.strip_prefix(root) {
                        Ok(p) => p.to_path_buf(),
                        Err(_) => path.clone(),
                    };

                    files.push(FileNode::new(path, relative_path));
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

    /// Tests that the scanner respects gitignore logic and depth.
    #[test]
    fn test_scan_fs_mechanics() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        // Create structure:
        // /root
        //   file1.txt
        //   .hidden_file
        //   ignored_dir/
        //     file2.txt
        //   .gitignore (content: "ignored_dir/")

        File::create(root.join("file1.txt"))?;
        File::create(root.join(".hidden_file"))?;
        fs::create_dir(root.join("ignored_dir"))?;
        File::create(root.join("ignored_dir/file2.txt"))?;
        fs::write(root.join(".gitignore"), "ignored_dir/")?;

        let scanner = FsScanner::new();

        // Case 1: Default scan (Should find file1.txt, ignore hidden, ignore gitignored)
        let config = ContextConfig::new(root.to_path_buf(), None, None, false, false);
        let results = scanner.scan(&config)?;

        let found_paths: Vec<_> = results
            .iter()
            .map(|n| n.relative_path.to_string_lossy().to_string())
            .collect();

        assert!(found_paths.contains(&"file1.txt".to_string()));
        assert!(!found_paths.contains(&".hidden_file".to_string()));
        assert!(!found_paths.contains(&"ignored_dir/file2.txt".to_string()));

        // Case 2: Include hidden
        let config_hidden = ContextConfig::new(root.to_path_buf(), None, None, true, false);
        let results_hidden = scanner.scan(&config_hidden)?;
        let found_paths_hidden: Vec<_> = results_hidden
            .iter()
            .map(|n| n.relative_path.to_string_lossy().to_string())
            .collect();

        assert!(found_paths_hidden.contains(&".hidden_file".to_string()));

        Ok(())
    }
}
