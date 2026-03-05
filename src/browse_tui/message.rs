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
    // Modals controls
    OpenHelp,
    OpenInfo(&'static str),
    CloseModal,
    // Error reporting
    SetError(String),
}
