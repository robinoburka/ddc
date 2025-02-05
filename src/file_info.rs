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
