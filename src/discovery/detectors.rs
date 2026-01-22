use std::path::Path;

use crate::discovery::Language;
use crate::discovery::discovery_manager::DynamicDetector;
use crate::files_db::FilesDB;

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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::file_info::FileMeta;

    fn add_record(db: &mut FilesDB, path: &str) {
        db.add(
            PathBuf::from(path),
            FileMeta {
                is_dir: true,
                size: None,
                touched: None,
            },
        );
    }

    fn get_virtual_layout() -> FilesDB {
        let mut db = FilesDB::new();
        add_record(&mut db, "projects");
        add_record(&mut db, "projects/python");
        add_record(&mut db, "projects/python/venv");
        add_record(&mut db, "projects/python/venv/bin");
        add_record(&mut db, "projects/python/venv/bin/python");
        add_record(&mut db, "projects/python/wrong_venv");
        add_record(&mut db, "projects/python/wrong_venv/bin");
        add_record(&mut db, "projects/rust");
        add_record(&mut db, "projects/rust/target");
        add_record(&mut db, "projects/rust/target/debug");
        add_record(&mut db, "projects/rust/target/debug/build");
        add_record(&mut db, "projects/rust/target/release");
        add_record(&mut db, "projects/rust/target/release/build");
        add_record(&mut db, "projects/rust_only_debug/");
        add_record(&mut db, "projects/rust_only_debug/target");
        add_record(&mut db, "projects/rust_only_debug/target/debug");
        add_record(&mut db, "projects/rust_only_debug/target/debug/build");
        add_record(&mut db, "projects/rust_only_release/");
        add_record(&mut db, "projects/rust_only_release/target");
        add_record(&mut db, "projects/rust_only_release/target/release");
        add_record(&mut db, "projects/rust_only_release/target/release/build");
        add_record(&mut db, "projects/node");
        add_record(&mut db, "projects/node/node_modules");
        add_record(&mut db, "projects/node/node_modules/.bin");
        add_record(&mut db, "projects/node/node_modules/.bin/foo");
        add_record(&mut db, "projects/node/node_modules/.bin/foo/node_modules");
        add_record(
            &mut db,
            "projects/node/node_modules/.bin/foo/node_modules/.bin",
        );

        db
    }
    #[test]
    fn test_python_detector() {
        let db = get_virtual_layout();
        let detector = PythonVenvDetector::default();

        assert_eq!(
            detector.detect(&db, &PathBuf::from("projects/python/venv")),
            true
        );
        assert_eq!(
            detector.detect(&db, &PathBuf::from("projects/python/wrong_venv")),
            false
        );
    }

    #[test]
    fn test_rust_detector_debug() {
        let db = get_virtual_layout();
        let detector = RustBuildDirDetector::default();

        assert_eq!(
            detector.detect(&db, &PathBuf::from("projects/rust_only_debug/target")),
            true
        );
        assert_eq!(
            detector.detect(&db, &PathBuf::from("projects/rust/target")),
            true
        );
    }

    #[test]
    fn test_rust_detector_release() {
        let db = get_virtual_layout();
        let detector = RustBuildDirDetector::default();

        assert_eq!(
            detector.detect(&db, &PathBuf::from("projects/rust_only_release/target")),
            true
        );
        assert_eq!(
            detector.detect(&db, &PathBuf::from("projects/rust/target")),
            true
        );
    }

    #[test]
    fn test_rust_detector_both() {
        let db = get_virtual_layout();
        let detector = RustBuildDirDetector::default();

        assert_eq!(
            detector.detect(&db, &PathBuf::from("projects/rust/target")),
            true
        );
    }

    #[test]
    fn test_node_detector() {
        let db = get_virtual_layout();
        let detector = JsNpmDetector::default();

        assert_eq!(
            detector.detect(&db, &PathBuf::from("projects/node/node_modules")),
            true
        );
        assert_eq!(
            detector.detect(
                &db,
                &PathBuf::from("projects/node/node_modules/.bin/node_modules")
            ),
            false
        );
    }

    #[test]
    fn test_node_detector_on_nested() {
        let db = get_virtual_layout();
        let detector = JsNpmDetector::default();

        assert_eq!(
            detector.detect(
                &db,
                &PathBuf::from("projects/node/node_modules/.bin/node_modules")
            ),
            false
        );
    }
}
