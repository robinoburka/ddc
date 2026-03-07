use std::path::PathBuf;

#[derive(Debug, Default, Copy, Clone)]
pub enum Tab {
    #[default]
    Projects,
    Tooling,
}

impl Tab {
    pub fn index(&self) -> usize {
        match self {
            Tab::Projects => 0,
            Tab::Tooling => 1,
        }
    }
}

#[derive(Debug)]
pub enum AppMessage {
    // Basic app controls
    Quit,
    Refresh,
    // Main views controls
    SelectTab(Tab),
    EnterBrowser(PathBuf),
    CloseBrowser,
    // Modals controls
    OpenHelp,
    OpenInfo(&'static str),
    OpenSort(&'static [SortBy]),
    RequestSort(SortBy),
    CloseModal,
    // Error reporting
    SetError(String),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SortBy {
    Project,
    Size,
    LastUpdate,
}

impl SortBy {
    pub fn key(&self) -> char {
        match self {
            SortBy::Project => 'p',
            SortBy::Size => 's',
            SortBy::LastUpdate => 'u',
        }
    }

    pub fn label(&self) -> &str {
        match self {
            SortBy::Project => "Project",
            SortBy::Size => "Size",
            SortBy::LastUpdate => "Last update",
        }
    }

    pub fn default_direction(&self) -> SortDirection {
        match self {
            SortBy::Project => SortDirection::Ascending,
            SortBy::Size => SortDirection::Descending,
            SortBy::LastUpdate => SortDirection::Ascending,
        }
    }
}
