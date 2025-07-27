use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct FileInfo {
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub touched: Option<SystemTime>,
}

impl FileInfo {
    pub fn new(path: &Path, metadata: &fs::Metadata) -> Self {
        Self {
            path: path.to_path_buf(),
            is_dir: metadata.is_dir(),
            size: Some(metadata.len()),
            touched: metadata.modified().ok(),
        }
    }
}

pub fn get_file_info(path: &Path) -> Result<FileInfo, std::io::Error> {
    let metadata = fs::metadata(path).or(fs::symlink_metadata(path))?;
    Ok(FileInfo::new(path, &metadata))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_info_file() {
        let tmp = tempfile::tempdir().unwrap();
        let dir_path = tmp.path().join("test_dir");
        let file_path = dir_path.join("test_file.txt");
        fs::create_dir_all(&dir_path).unwrap();
        fs::write(&file_path, "Hello, World!").unwrap();
        let dir_size = fs::metadata(&dir_path).unwrap().len();

        let file_info = get_file_info(&file_path).unwrap();
        assert_eq!(file_info.path, file_path);
        assert!(!file_info.is_dir);
        assert_eq!(file_info.size, Some(13));
        assert!(file_info.touched.is_some());

        let file_info = get_file_info(&dir_path).unwrap();
        assert_eq!(file_info.path, dir_path);
        assert!(file_info.is_dir);
        assert_eq!(file_info.size, Some(dir_size));
        assert!(file_info.touched.is_some());
    }
}
