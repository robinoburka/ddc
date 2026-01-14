use std::path::PathBuf;

use crate::discovery::DiscoveryResult;
use crate::file_info::FileInfo;
use crate::files_db::FilesDB;

#[derive(Debug, PartialEq)]
pub(super) enum Tab {
    Projects,
    Tools,
}

#[derive(Debug)]
pub(super) enum BrowserFrame {
    Projects(DiscoveryResults),
    Directory(DirectoryFrame),
}

#[derive(Debug)]
pub(super) struct DiscoveryResults {
    pub(super) current_view: Tab,
    pub(super) projects: DiscoveryResultsView,
    pub(super) tools: DiscoveryResultsView,
}

impl DiscoveryResults {
    pub(super) fn get_mut_view(&mut self) -> &mut DiscoveryResultsView {
        match self.current_view {
            Tab::Projects => &mut self.projects,
            Tab::Tools => &mut self.tools,
        }
    }

    pub(super) fn get_view(&self) -> &DiscoveryResultsView {
        match self.current_view {
            Tab::Projects => &self.projects,
            Tab::Tools => &self.tools,
        }
    }
}

#[derive(Debug)]
pub(super) struct DiscoveryResultsView {
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
        }
    }
}
