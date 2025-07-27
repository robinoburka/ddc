use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug)]
pub struct FileMeta {
    pub is_dir: bool,
    pub size: Option<u64>,
    pub touched: Option<SystemTime>,
}

impl From<&fs::Metadata> for FileMeta {
    fn from(metadata: &fs::Metadata) -> Self {
        Self {
            is_dir: metadata.is_dir(),
            size: Some(metadata.len()),
            touched: metadata.modified().ok(),
        }
    }
}

pub fn get_file_meta(path: &Path) -> Result<FileMeta, std::io::Error> {
    let metadata = fs::metadata(path).or(fs::symlink_metadata(path))?;
    Ok(FileMeta::from(&metadata))
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FileInfo<'a> {
    pub path: &'a PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub touched: Option<SystemTime>,
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

        let file_info = get_file_meta(&file_path).unwrap();
        assert!(!file_info.is_dir);
        assert_eq!(file_info.size, Some(13));
        assert!(file_info.touched.is_some());

        let file_info = get_file_meta(&dir_path).unwrap();
        assert!(file_info.is_dir);
        assert_eq!(file_info.size, Some(dir_size));
        assert!(file_info.touched.is_some());
    }
}
