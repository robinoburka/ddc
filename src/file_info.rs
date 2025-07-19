use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq)]
pub struct FileInfo {
    pub path: PathBuf,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub touched: Option<SystemTime>,
}

impl TryFrom<&PathBuf> for FileInfo {
    type Error = std::io::Error;

    fn try_from(p: &PathBuf) -> Result<Self, Self::Error> {
        let metadata = fs::metadata(p).or(fs::symlink_metadata(p))?;
        Ok(Self {
            path: p.clone(),
            is_dir: metadata.is_dir(),
            size: Some(metadata.len()),
            touched: metadata.modified().ok(),
        })
    }
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

        let file_info = FileInfo::try_from(&file_path).unwrap();
        assert_eq!(file_info.path, file_path);
        assert!(!file_info.is_dir);
        assert_eq!(file_info.size, Some(13));
        assert!(file_info.touched.is_some());

        let file_info = FileInfo::try_from(&dir_path).unwrap();
        assert_eq!(file_info.path, dir_path);
        assert!(file_info.is_dir);
        assert_eq!(file_info.size, Some(dir_size));
        assert!(file_info.touched.is_some());
    }
}
