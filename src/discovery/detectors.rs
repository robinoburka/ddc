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

#[derive(Default)]
pub struct JsNpmDetector;

impl DynamicDetector for JsNpmDetector {
    const LANG: Language = Language::JS;

    fn detect(&self, db: &FilesDB, path: &Path) -> bool {
        if path.ends_with("node_modules") && db.is_dir(&path.join(".bin")) {
            let cnt = path
                .components()
                .filter(|c| c.as_os_str() == "node_modules")
                .count();
            cnt == 1
        } else {
            false
        }
    }
}
