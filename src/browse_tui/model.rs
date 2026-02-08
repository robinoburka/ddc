use std::path::PathBuf;
use std::time::SystemTime;

use ratatui::widgets::{ScrollbarState, TableState};

use crate::discovery::{ProjectResult, ToolingResult};
use crate::file_info::FileInfo;
use crate::files_db::FilesDB;

#[derive(Debug)]
pub(super) enum Tab {
    Projects(ResultsTab<ProjectResult>),
    Tooling(ResultsTab<ToolingResult>),
}

#[derive(Debug)]
pub(super) struct ResultsTab<T> {
    pub(super) state: TableState,
    pub(super) scroll_state: ScrollbarState,
    pub(super) results: Vec<T>,
}

#[derive(Debug)]
pub(super) struct Browser {
    pub(super) frames: Vec<DirectoryBrowserFrame>,
}

#[derive(Debug)]
pub(super) struct DirectoryBrowserFrame {
    pub(super) state: TableState,
    pub(super) scroll_state: ScrollbarState,
    pub(super) cwd: PathBuf,
    pub(super) directory_list: Vec<DirItem>,
}

#[derive(Debug, Clone)]
pub(super) struct DirItem {
    pub(super) name: String,
    pub(super) path: PathBuf,
    pub(super) is_directory: bool,
    pub(super) size: Option<u64>,
    pub(super) last_update: Option<SystemTime>,
}

impl DirItem {
    pub(super) fn from_file_info(file_info: &FileInfo, db: &FilesDB) -> Self {
        Self {
            name: file_info
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string(),
            path: file_info.path.clone(),
            is_directory: file_info.is_dir,
            size: if file_info.is_dir {
                Some(
                    db.iter_dir(file_info.path)
                        .filter_map(|item| item.size)
                        .sum(),
                )
            } else {
                file_info.size
            },
            last_update: file_info.touched,
        }
    }
}
