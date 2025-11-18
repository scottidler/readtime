use eyre::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::counter;
use crate::estimator;

/// Information about a processed file
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: PathBuf,
    pub read_time_minutes: usize,
}

/// Process a single file or directory
pub fn process_path<P: AsRef<Path>>(
    path: P,
    extensions: &[String],
    words_per_minute: usize,
) -> Result<Vec<FileInfo>> {
    let path = path.as_ref();

    if !path.exists() {
        return Err(eyre::eyre!("Path does not exist: {}", path.display()));
    }

    let extensions_set: HashSet<_> = extensions.iter().map(|s| s.as_str()).collect();

    if path.is_file() {
        // Process single file - check extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions_set.contains(ext) {
                let file_info = process_file(path, words_per_minute)?;
                Ok(vec![file_info])
            } else {
                Err(eyre::eyre!(
                    "File {} has unsupported extension. Supported: {:?}",
                    path.display(),
                    extensions
                ))
            }
        } else {
            Err(eyre::eyre!(
                "File {} has no extension. Supported: {:?}",
                path.display(),
                extensions
            ))
        }
    } else {
        // Process directory recursively
        let mut results = Vec::new();

        for entry in WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden files and directories (but not the root)
                if e.depth() == 0 {
                    true
                } else {
                    !e.file_name()
                        .to_str()
                        .map(|s| s.starts_with('.'))
                        .unwrap_or(false)
                }
            })
        {
            let entry = entry.wrap_err("Failed to read directory entry")?;

            if !entry.file_type().is_file() {
                continue;
            }

            let file_path = entry.path();

            // Check if file has matching extension
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str())
                && extensions_set.contains(ext)
            {
                match process_file(file_path, words_per_minute) {
                    Ok(file_info) => results.push(file_info),
                    Err(e) => {
                        log::warn!("Failed to process {}: {}", file_path.display(), e);
                    }
                }
            }
        }

        if results.is_empty() {
            log::warn!("No matching files found in {}", path.display());
        }

        Ok(results)
    }
}

/// Process a single file and calculate its reading time
fn process_file<P: AsRef<Path>>(path: P, words_per_minute: usize) -> Result<FileInfo> {
    let path = path.as_ref();

    let word_count = counter::count_words(path)
        .wrap_err_with(|| format!("Failed to count words in {}", path.display()))?;

    let read_time_minutes = estimator::estimate_reading_time(word_count, words_per_minute);

    Ok(FileInfo {
        path: path.to_path_buf(),
        read_time_minutes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_process_single_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_path = temp_dir.path().join("test.txt");

        let mut file = fs::File::create(&file_path)?;
        writeln!(file, "Hello world this is a test")?;

        let results = process_path(&file_path, &["txt".to_string()], 200)?;

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].read_time_minutes, 1);

        Ok(())
    }

    #[test]
    fn test_process_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create test files
        let file1 = temp_dir.path().join("file1.md");
        fs::write(&file1, "Hello world")?;

        let file2 = temp_dir.path().join("file2.txt");
        fs::write(&file2, "Another test file")?;

        let file3 = temp_dir.path().join("ignored.rs");
        fs::write(&file3, "This should be ignored")?;

        let results = process_path(temp_dir.path(), &["md".to_string(), "txt".to_string()], 200)?;

        assert_eq!(results.len(), 2);

        Ok(())
    }

    #[test]
    fn test_process_nested_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create nested structure
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir)?;

        let file1 = temp_dir.path().join("file1.md");
        fs::write(&file1, "Root file")?;

        let file2 = subdir.join("file2.md");
        fs::write(&file2, "Nested file")?;

        let results = process_path(temp_dir.path(), &["md".to_string()], 200)?;

        assert_eq!(results.len(), 2);

        Ok(())
    }
}
