use std::path::Path;

use crate::files_db::FilesDB;

pub fn rust_detector(db: &FilesDB, path: &Path) -> bool {
    path.ends_with("target")
        && (db.is_dir(&path.join("debug/build")) || db.is_dir(&path.join("release/build")))
}

pub fn python_detector(db: &FilesDB, path: &Path) -> bool {
    db.exists(&path.join("bin/python"))
}
