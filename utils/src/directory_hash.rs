use anyhow::Result;

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct DirectoryHash {
    pub hash: String,
    pub last_modified: SystemTime,
}

impl DirectoryHash {
    pub fn new(directory_path: String, ignore_dot_files: bool) -> Result<Self> {
        let path = Path::new(&directory_path);

        if !path.exists() {
            return Err(anyhow::anyhow!("Directory does not exist: {:?}", path));
        }

        if !path.is_dir() {
            return Err(anyhow::anyhow!("Path is not a directory: {:?}", path));
        }

        let mut file_hashes = BTreeMap::new();
        let mut latest_modified = SystemTime::UNIX_EPOCH;

        // Recursively walk through all files in directory
        Self::process_directory(path, &mut file_hashes, &mut latest_modified, ignore_dot_files)?;

        let mut hasher = Sha256::new();
        for (file_path, file_info) in file_hashes {
            hasher.update(file_path.as_bytes());
            hasher.update(file_info.size.to_le_bytes());
            hasher.update(file_info.modified_timestamp.to_le_bytes());
            hasher.update(&file_info.content_hash);
        }

        let hash = format!("{:x}", hasher.finalize());

        Ok(DirectoryHash {
            hash,
            last_modified: latest_modified,
        })
    }

    fn process_directory(
        dir_path: &Path,
        file_hashes: &mut BTreeMap<String, FileInfo>,
        latest_modified: &mut SystemTime,
        ignore_dot_files: bool,
    ) -> Result<()> {
        let entries = fs::read_dir(dir_path)?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // Check if we should ignore dot files/directories
            if ignore_dot_files {
                if let Some(file_name) = path.file_name() {
                    if file_name.to_string_lossy().starts_with('.') {
                        continue; // Skip dot files/directories
                    }
                }
            }

            if path.is_file() {
                let metadata = entry.metadata()?;
                let size = metadata.len();
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

                if modified > *latest_modified {
                    *latest_modified = modified;
                }

                let content = fs::read(&path)?;
                let mut hasher = Sha256::new();
                hasher.update(&content);
                let content_hash = hasher.finalize().to_vec();

                let relative_path = path
                    .strip_prefix(dir_path)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .to_string();

                let file_info = FileInfo {
                    size,
                    modified_timestamp: modified
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    content_hash,
                };

                file_hashes.insert(relative_path, file_info);
            } else if path.is_dir() {
                Self::process_directory(&path, file_hashes, latest_modified, ignore_dot_files)?;
            }
        }

        Ok(())
    }

    pub fn save_to_file(&self, file_path: String) -> Result<()> {
        fs::write(file_path, &self.hash)?;
        Ok(())
    }

    pub fn read_from_file(file_path: String) -> Result<String> {
        let content = fs::read_to_string(file_path)?;
        Ok(content.trim().to_string())
    }
}

#[derive(Debug)]
struct FileInfo {
    size: u64,
    modified_timestamp: u64,
    content_hash: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_directory_hash_creation() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Create test files
        fs::write(format!("{dir_path}/file1.txt"), "content1").unwrap();
        fs::write(format!("{dir_path}/file2.txt"), "content2").unwrap();

        let hash = DirectoryHash::new(dir_path, true).unwrap();

        //assert_eq!(hash.file_count, 2);
        //assert!(hash.total_size > 0);
        assert!(!hash.hash.is_empty());
    }

    #[test]
    fn test_directory_hash_changes() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().to_string_lossy().to_string();

        // Create initial file
        fs::write(format!("{dir_path}/file1.txt"), "content1").unwrap();
        let hash1 = DirectoryHash::new(dir_path.clone(), true).unwrap();

        // Add another file
        fs::write(format!("{dir_path}/file2.txt"), "content2").unwrap();
        let hash2 = DirectoryHash::new(dir_path, true).unwrap();

        assert_ne!(hash1.hash, hash2.hash);
    }
}
