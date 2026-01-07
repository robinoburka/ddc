use std::path::PathBuf;

use crate::discovery::DiscoveryResult;
use crate::file_info::FileInfo;

#[derive(Debug)]
pub(super) enum BrowserFrame {
    Projects(ProjectsFrame),
    Directory(DirectoryFrame),
}

#[derive(Debug)]
pub(super) struct ProjectsFrame {
    pub(super) current_item: usize,
    pub(super) results: Vec<DiscoveryResult>,
}

#[derive(Debug)]
pub(super) struct DirectoryFrame {
    pub(super) current_item: usize,
    pub(super) cwd: PathBuf,
    pub(super) directory_list: Vec<DirItem>,
}

#[derive(Debug, Clone)]
pub(super) struct DirItem {
    pub(super) name: String,
    pub(super) path: PathBuf,
    pub(super) is_directory: bool,
    pub(super) size: Option<u64>,
}

impl From<FileInfo<'_>> for DirItem {
    fn from(value: FileInfo) -> Self {
        Self {
            name: value
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string(),
            path: value.path.clone(),
            is_directory: value.is_dir,
            size: value.size,
        }
    }
}
