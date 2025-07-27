#![allow(dead_code)]
use std::collections::BTreeMap;
use std::collections::btree_map::Cursor;
use std::ops::Bound;
use std::path::PathBuf;

use crate::file_info::{FileInfo, FileMeta};

pub struct FilesDB {
    files: BTreeMap<PathBuf, FileMeta>,
}

impl FilesDB {
    pub fn new() -> Self {
        Self {
            files: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, path: PathBuf, meta: FileMeta) {
        self.files.insert(path, meta);
    }

    /// Iterate over complete content of `lookup_path`
    ///
    /// This is especially useful for getting sum of sizes of any path
    ///
    /// # Examples
    /// ```
    /// let lookup_path = "/foo/bar";
    /// db.iter_dir.filter_map(|fi| fi.size).sum();
    /// ```
    pub fn iter_dir<'a, 'b>(&'a self, lookup_path: &'b PathBuf) -> DirectoryIter<'a, 'b> {
        DirectoryIter {
            cursor: self.files.lower_bound(Bound::Included(lookup_path)),
            lookup_path,
        }
    }

    /// Iterate over items on current level
    ///
    /// This is basically `ls PATH` operation on FilesDB.
    pub fn iter_level<'a, 'b>(&'a self, lookup_path: &'b PathBuf) -> LevelIter<'a, 'b> {
        LevelIter {
            cursor: self.files.lower_bound(Bound::Included(lookup_path)),
            lookup_path,
        }
    }

    /// Iterate over all directories inside `lookup_path` tree
    ///
    /// This enables you to get all directories, so you can perform specialised
    /// checks for different files/directories on some prefix.
    pub fn iter_directories<'a, 'b>(&'a self, lookup_path: &'b PathBuf) -> AllDirsIter<'a, 'b> {
        AllDirsIter {
            cursor: self.files.lower_bound(Bound::Included(lookup_path)),
            lookup_path,
        }
    }

    pub fn is_dir(&self, path: &PathBuf) -> bool {
        match self.files.get(path) {
            None => false,
            Some(f) => f.is_dir,
        }
    }

    pub fn exists(&self, path: &PathBuf) -> bool {
        self.files.contains_key(path)
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }
}

pub struct DirectoryIter<'a, 'b> {
    cursor: Cursor<'a, PathBuf, FileMeta>,
    lookup_path: &'b PathBuf,
}

impl<'a> Iterator for DirectoryIter<'a, '_> {
    type Item = FileInfo<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let (path, meta) = self.cursor.next()?;
        if !path.starts_with(self.lookup_path) {
            return None;
        }

        Some(FileInfo {
            path,
            is_dir: meta.is_dir,
            size: meta.size,
            touched: meta.touched,
        })
    }
}

pub struct LevelIter<'a, 'b> {
    cursor: Cursor<'a, PathBuf, FileMeta>,
    lookup_path: &'b PathBuf,
}

impl<'a> Iterator for LevelIter<'a, '_> {
    type Item = FileInfo<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((path, meta)) = self.cursor.next() {
            if path.parent()? == self.lookup_path {
                return Some(FileInfo {
                    path,
                    is_dir: meta.is_dir,
                    size: meta.size,
                    touched: meta.touched,
                });
            }
        }

        None
    }
}

pub struct AllDirsIter<'a, 'b> {
    cursor: Cursor<'a, PathBuf, FileMeta>,
    lookup_path: &'b PathBuf,
}

impl<'a> Iterator for AllDirsIter<'a, '_> {
    type Item = FileInfo<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((path, meta)) = self.cursor.next() {
            if !path.starts_with(self.lookup_path) {
                return None;
            }
            if meta.is_dir {
                return Some(FileInfo {
                    path,
                    is_dir: meta.is_dir,
                    size: meta.size,
                    touched: meta.touched,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_info::FileMeta;

    fn build_test_structure() -> FilesDB {
        let mut db = FilesDB::new();
        db.add(
            PathBuf::from("/foo"),
            FileMeta {
                is_dir: true,
                size: None,
                touched: None,
            },
        );
        db.add(
            PathBuf::from("/foo/a.txt"),
            FileMeta {
                is_dir: false,
                size: Some(10),
                touched: None,
            },
        );
        db.add(
            PathBuf::from("/foo/bar"),
            FileMeta {
                is_dir: true,
                size: None,
                touched: None,
            },
        );
        db.add(
            PathBuf::from("/foo/bar/empty"),
            FileMeta {
                is_dir: true,
                size: None,
                touched: None,
            },
        );
        db.add(
            PathBuf::from("/foo/baz"),
            FileMeta {
                is_dir: true,
                size: None,
                touched: None,
            },
        );
        db.add(
            PathBuf::from("/foo/baz/b.txt"),
            FileMeta {
                is_dir: false,
                size: Some(20),
                touched: None,
            },
        );

        return db;
    }
    #[test]
    fn exists() {
        let db = build_test_structure();

        assert_eq!(db.exists(&PathBuf::from("/foo/baz/b.txt")), true);
        assert_eq!(db.exists(&PathBuf::from("/foo/baz/c.txt")), false);
        assert_eq!(db.exists(&PathBuf::from("/foo/baz/")), true);
    }

    #[test]
    fn is_dir() {
        let db = build_test_structure();

        assert_eq!(db.is_dir(&PathBuf::from("/foo/baz/b.txt")), false);
        assert_eq!(db.is_dir(&PathBuf::from("/foo/baz/")), true);
        assert_eq!(db.is_dir(&PathBuf::from("/foo/bazz/")), false);
    }

    #[test]
    fn iter_dir() {
        let db = build_test_structure();
        let q = PathBuf::from("/foo/baz");
        let mut it = db.iter_dir(&q);

        assert_eq!(it.next().unwrap().path, &PathBuf::from("/foo/baz"));
        assert_eq!(it.next().unwrap().path, &PathBuf::from("/foo/baz/b.txt"));
        assert_eq!(it.next().is_none(), true);
    }

    #[test]
    fn iter_level() {
        let db = build_test_structure();
        let q = PathBuf::from("/foo");
        let mut it = db.iter_level(&q);

        assert_eq!(it.next().unwrap().path, &PathBuf::from("/foo/a.txt"));
        assert_eq!(it.next().unwrap().path, &PathBuf::from("/foo/bar"));
        assert_eq!(it.next().unwrap().path, &PathBuf::from("/foo/baz"));
        assert_eq!(it.next().is_none(), true);
    }

    #[test]
    fn iter_directories() {
        let db = build_test_structure();
        let q = PathBuf::from("/foo");
        let mut it = db.iter_directories(&q);

        assert_eq!(it.next().unwrap().path, &PathBuf::from("/foo"));
        assert_eq!(it.next().unwrap().path, &PathBuf::from("/foo/bar"));
        assert_eq!(it.next().unwrap().path, &PathBuf::from("/foo/bar/empty"));
        assert_eq!(it.next().unwrap().path, &PathBuf::from("/foo/baz"));
        assert_eq!(it.next().is_none(), true);

        let q = PathBuf::from("/foo/bar");
        let mut it = db.iter_directories(&q);

        assert_eq!(it.next().unwrap().path, &PathBuf::from("/foo/bar"));
        assert_eq!(it.next().unwrap().path, &PathBuf::from("/foo/bar/empty"));
        assert_eq!(it.next().is_none(), true);
    }
}
