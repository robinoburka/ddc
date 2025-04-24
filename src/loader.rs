use std::ops::Deref;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam::channel;
use indicatif::style::ProgressTracker;
use jwalk::rayon::prelude::*;
use jwalk::{Parallelism, WalkDir};
use tracing::{debug, debug_span};

use crate::discovery::PathLoader;
use crate::file_info::FileInfo;
use crate::files_db::FilesDB;
use crate::progress_report::ProgressReport;
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

fn walk_dir_paths(directory: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(directory)
        .parallelism(Parallelism::Serial)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|res| res.map(|de| de.path()).ok())
        .collect::<Vec<_>>()
}

fn walk_dir_file_infos(directory: &PathBuf) -> Vec<FileInfo> {
    WalkDir::new(directory)
        .parallelism(Parallelism::Serial)
        .skip_hidden(false)
        .into_iter()
        .filter_map(|res| res.map(|de| de.path()).ok())
        .filter_map(|path| FileInfo::try_from(&path).ok())
        .collect::<Vec<_>>()
}

#[derive(Default)]
pub struct BaseLoader;

impl PathLoader for BaseLoader {
    fn load_multiple_paths(&self, scan_paths: &[PathBuf]) -> FilesDB {
        let (sender, receiver) = channel();

        scan_paths
            .into_par_iter()
            .for_each_with(sender, |sender, path| {
                let infos = walk_dir_file_infos(path);
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

#[derive(Default)]
pub struct FullyParallelLoader{
    progress_report: Option<Arc<ProgressReport>>,
}

impl FullyParallelLoader {
    const NUM_THREADS: usize = 4;

    pub fn with_progress(mut self, progress_report: Arc<ProgressReport>) -> Self {
        self.progress_report = Some(progress_report);
        self
    }
}

impl PathLoader for FullyParallelLoader {
    fn load_multiple_paths(&self, scan_paths: &[PathBuf]) -> FilesDB {
        let (paths_sender, paths_receiver) = channel::unbounded();
        let (infos_sender, infos_receiver) = channel::unbounded();

        if let Some(p) = &self.progress_report {
            p.start_scan(scan_paths.len());
        }
        
        scan_paths.iter().for_each(|path| {
            let my_paths_sender = paths_sender.clone();
            let path = path.clone();
            let progress = match &self.progress_report {
                None => None,
                Some(p) => Some(Arc::clone(p)),
            };
            thread::spawn(move || {
                let _guard = debug_span!("walk_dir", path = ?path).entered();
                let paths = walk_dir_paths(&path);
                paths.into_iter().for_each(|path| {
                    my_paths_sender.send(path).unwrap();
                });
                if let Some(p) = progress {
                    p.tick();
                }
            });
        });
        drop(paths_sender);

        for _ in 0..Self::NUM_THREADS {
            let my_paths_receiver = paths_receiver.clone();
            let my_infos_sender = infos_sender.clone();
            thread::spawn(move || {
                my_paths_receiver.iter().for_each(|path| {
                    if let Ok(info) = FileInfo::try_from(&path) {
                        my_infos_sender.send((path, info)).unwrap();
                    } else {
                        debug!("Failed to load info for {}", path.display());
                    }
                });
            });
        }
        drop(paths_receiver);
        drop(infos_sender);

        let mut db = FilesDB::new();
        infos_receiver.iter().for_each(|(path, info)| {
            db.add(path, info);
        });
        drop(infos_receiver);

        db
    }
}
