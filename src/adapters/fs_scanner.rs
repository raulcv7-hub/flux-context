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
        debug!(
            "Starting scan at: {:?}. NoIgnore: {}, Hidden: {}",
            root, config.no_ignore, config.include_hidden
        );

        let mut builder = WalkBuilder::new(root);

        builder
            .standard_filters(true)
            .hidden(!config.include_hidden)
            .git_ignore(!config.no_ignore)
            .git_global(!config.no_ignore)
            .ignore(!config.no_ignore);

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
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::tempdir;

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

        let main_rs = PathBuf::from("src")
            .join("main.rs")
            .to_string_lossy()
            .to_string();
        let adapters_mod = PathBuf::from("src")
            .join("adapters")
            .join("mod.rs")
            .to_string_lossy()
            .to_string();
        let docs_info = PathBuf::from("docs")
            .join("info.md")
            .to_string_lossy()
            .to_string();

        // 1. Include "src"
        let mut config_src = ContextConfig::default();
        config_src.root_path = root.to_path_buf();
        config_src.include_paths.push("src".into());

        let res_src = scanner.scan(&config_src)?;
        let paths: Vec<_> = res_src
            .iter()
            .map(|f| f.relative_path.to_string_lossy().to_string())
            .collect();

        assert!(
            paths.contains(&main_rs),
            "Expected {:?} in {:?}",
            main_rs,
            paths
        );
        assert!(paths.contains(&adapters_mod));
        assert!(!paths.contains(&docs_info));

        Ok(())
    }

    #[test]
    fn test_scan_gitignore_respect_and_bypass() -> Result<()> {
        let dir = tempdir()?;
        let root = dir.path();

        // Create a secret file and a .gitignore
        File::create(root.join("secret.env"))?;
        File::create(root.join("public.rs"))?;

        let mut gitignore = File::create(root.join(".gitignore"))?;
        writeln!(gitignore, "*.env")?;

        let scanner = FsScanner::new();

        // Case 1: Default (Respect gitignore)
        let mut config_default = ContextConfig::default();
        config_default.root_path = root.to_path_buf();
        let files_default = scanner.scan(&config_default)?;
        let paths_default: Vec<_> = files_default
            .iter()
            .map(|f| f.relative_path.file_name().unwrap().to_str().unwrap())
            .collect();

        assert!(paths_default.contains(&"public.rs"));
        assert!(!paths_default.contains(&"secret.env"));

        // Case 2: No Ignore (Bypass gitignore)
        let mut config_no_ignore = ContextConfig::default();
        config_no_ignore.root_path = root.to_path_buf();
        config_no_ignore.no_ignore = true; // ACTIVATE FLAG

        let files_ignored = scanner.scan(&config_no_ignore)?;
        let paths_ignored: Vec<_> = files_ignored
            .iter()
            .map(|f| f.relative_path.file_name().unwrap().to_str().unwrap())
            .collect();

        assert!(paths_ignored.contains(&"public.rs"));
        assert!(paths_ignored.contains(&"secret.env"));

        Ok(())
    }
}