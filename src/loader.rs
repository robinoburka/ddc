use std::path::PathBuf;
use std::sync::mpsc::channel;

use crossbeam::channel;
use jwalk::rayon::prelude::*;
use jwalk::{Parallelism, WalkDir};
use tracing::{debug, debug_span};

use crate::discovery::PathLoader;
use crate::file_info::{FileMeta, get_file_meta};
use crate::files_db::FilesDB;

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Default)]
pub struct BaseLoader;

impl PathLoader for BaseLoader {
    fn load_multiple_paths(&self, scan_paths: &[PathBuf]) -> FilesDB {
        let (sender, receiver) = channel();

        scan_paths
            .into_par_iter()
            .for_each_with(sender, |sender, path| {
                let paths = walk_dir_paths(path);
                paths
                    .into_iter()
                    .filter_map(|path| match get_file_meta(&path) {
                        Ok(meta) => Some((path, meta)),
                        Err(_e) => {
                            debug!("Failed to load info for {}", path.display());
                            None
                        }
                    })
                    .for_each(|(path, meta)| sender.send((path, meta)).unwrap());
            });

        let mut db = FilesDB::new();
        receiver.iter().for_each(|(path, meta)| {
            db.add(path, meta);
        });

        db
    }
}

#[derive(Default)]
pub struct FullyParallelLoader;

impl FullyParallelLoader {
    const NUM_LOADER_THREADS: usize = 4;
    const NUM_WORKER_THREADS: usize = 6;
    const BULK_SIZE: usize = 10000;
}

impl PathLoader for FullyParallelLoader {
    fn load_multiple_paths(&self, scan_paths: &[PathBuf]) -> FilesDB {
        let (sources_sender, sources_receiver) = channel::unbounded();
        let (paths_sender, paths_receiver) = channel::unbounded();
        let (infos_sender, infos_receiver) = channel::unbounded();

        scan_paths.iter().for_each(|path| {
            sources_sender.send(path.clone()).unwrap();
        });
        drop(sources_sender);

        for _ in 0..Self::NUM_LOADER_THREADS {
            let my_paths_sender = paths_sender.clone();
            let my_sources_receiver = sources_receiver.clone();
            rayon::spawn(move || {
                let mut buffer: Vec<PathBuf> = Vec::with_capacity(Self::BULK_SIZE);
                for path in my_sources_receiver.iter() {
                    let _guard = debug_span!("walk_dir", path = ?path).entered();
                    let loaded_paths = walk_dir_paths(&path);
                    for path in loaded_paths {
                        buffer.push(path);
                        if buffer.len() >= Self::BULK_SIZE {
                            my_paths_sender.send(buffer).unwrap();
                            buffer = Vec::with_capacity(Self::BULK_SIZE);
                        }
                    }
                }
                if !buffer.is_empty() {
                    my_paths_sender.send(buffer).unwrap();
                }
            });
        }
        drop(sources_receiver);
        drop(paths_sender);

        for _ in 0..Self::NUM_WORKER_THREADS {
            let my_paths_receiver = paths_receiver.clone();
            let my_infos_sender = infos_sender.clone();
            rayon::spawn(move || {
                let mut out_buffer: Vec<(PathBuf, FileMeta)> = Vec::with_capacity(Self::BULK_SIZE);
                for in_buffer in my_paths_receiver.iter() {
                    for path in in_buffer {
                        if let Ok(meta) = get_file_meta(&path) {
                            out_buffer.push((path, meta));
                            if out_buffer.len() >= Self::BULK_SIZE {
                                my_infos_sender.send(out_buffer).unwrap();
                                out_buffer = Vec::with_capacity(Self::BULK_SIZE);
                            }
                        } else {
                            debug!("Failed to load info for {}", path.display());
                        }
                    }
                }
                if !out_buffer.is_empty() {
                    my_infos_sender.send(out_buffer).unwrap();
                }
            });
        }
        drop(paths_receiver);
        drop(infos_sender);

        let mut db = FilesDB::new();
        infos_receiver.iter().for_each(|in_buffer| {
            for (path, meta) in in_buffer {
                db.add(path, meta);
            }
        });
        drop(infos_receiver);

        debug!("Loaded {} files into files DB", db.len());

        db
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_base_loader() {
        perform_test(BaseLoader::default());
    }

    #[test]
    fn test_fully_parallel_loader() {
        perform_test(FullyParallelLoader::default());
    }

    fn perform_test<L: PathLoader>(loader: L) {
        let tmp = tempdir().unwrap();
        let root_path = tmp.path();
        let dir_path = root_path.join("foo/bar");
        let file_path = dir_path.join("baz.txt");
        fs::create_dir_all(&dir_path).unwrap();
        fs::write(&file_path, "Hello, World!").unwrap();

        let db = loader.load_multiple_paths(&[root_path.to_path_buf()]);

        assert!(db.exists(&root_path.join("foo")));
        assert!(db.is_dir(&root_path.join("foo")));
        assert!(db.exists(&root_path.join("foo/bar")));
        assert!(db.is_dir(&root_path.join("foo/bar")));
        assert!(db.exists(&root_path.join("foo/bar/baz.txt")));
        assert!(!db.is_dir(&root_path.join("foo/bar/baz.txt")));
    }
}
