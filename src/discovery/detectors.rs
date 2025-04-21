use std::path::Path;

use crate::discovery::discovery_manager::DynamicDetector;
use crate::files_db::FilesDB;
use crate::types::Language;

#[derive(Default)]
pub struct PythonVenvDetector;

impl DynamicDetector for PythonVenvDetector {
    const LANG: Language = Language::Python;

    fn detect(&self, db: &FilesDB, path: &Path) -> bool {
        db.exists(&path.join("bin/python"))
    }
}

#[derive(Default)]
pub struct RustBuildDirDetector;

impl DynamicDetector for RustBuildDirDetector {
    const LANG: Language = Language::Rust;

    fn detect(&self, db: &FilesDB, path: &Path) -> bool {
        path.ends_with("target")
            && (db.is_dir(&path.join("debug/build")) || db.is_dir(&path.join("release/build")))
    }
}
