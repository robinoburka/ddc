use std::path::PathBuf;
use std::sync::mpsc::channel;

use jwalk::rayon::prelude::*;
use jwalk::{Parallelism, WalkDir};

use crate::discovery::PathLoader;
use crate::file_info::FileInfo;
use crate::files_db::FilesDB;

// The value was carefully tested and smaller numbers work better than higher.
// const THREADS: usize = 4;

#[derive(thiserror::Error, Debug)]
pub enum LoaderError {
    #[error("Unable to access metadata: {inner}")]
    FailedToAccessMetadata {
        #[from]
        inner: std::io::Error,
    },
}

pub struct BaseLoader;

impl BaseLoader {
    pub fn new() -> Self {
        Self
    }

    fn load_file_infos(&self, directory: &PathBuf) -> Vec<FileInfo> {
        let paths = WalkDir::new(directory)
            .parallelism(Parallelism::Serial)
            .skip_hidden(false)
            .into_iter()
            .filter_map(|res| res.map(|de| de.path()).ok())
            .filter_map(|path| FileInfo::try_from(&path).ok())
            .collect::<Vec<_>>();
        paths
    }
}
impl PathLoader for BaseLoader {
    fn load_multiple_paths(&self, scan_paths: &[PathBuf]) -> FilesDB {
        let (sender, receiver) = channel();

        scan_paths
            .into_par_iter()
            .for_each_with(sender, |sender, path| {
                let infos = self.load_file_infos(&path);
                infos
                    .into_iter()
                    .for_each(|fi| sender.send((fi.path.clone(), fi)).unwrap());
            });

        let mut db = FilesDB::new();
        receiver.iter().for_each(|(path, info)| {
            db.add(path, info);
        });

        db
    }
}
