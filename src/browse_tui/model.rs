use std::path::PathBuf;

use crate::discovery::DiscoveryResult;
use crate::file_info::FileInfo;

#[derive(Debug, PartialEq)]
pub(super) enum Tab {
    Projects,
    Tools,
}

#[derive(Debug)]
pub(super) enum BrowserFrame {
    Projects(ProjectsFrame),
    Directory(DirectoryFrame),
}

#[derive(Debug)]
pub(super) struct ProjectsFrame {
    pub(super) current_view: Tab,
    pub(super) projects: ProjectsFrameView,
    pub(super) tools: ProjectsFrameView,
}

impl ProjectsFrame {
    pub(super) fn get_mut_view(&mut self) -> &mut ProjectsFrameView {
        match self.current_view {
            Tab::Projects => &mut self.projects,
            Tab::Tools => &mut self.tools,
        }
    }

    pub(super) fn get_view(&self) -> &ProjectsFrameView {
        match self.current_view {
            Tab::Projects => &self.projects,
            Tab::Tools => &self.tools,
        }
    }
}

#[derive(Debug)]
pub(super) struct ProjectsFrameView {
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
