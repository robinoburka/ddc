use std::path::PathBuf;
use std::sync::mpsc::channel;

use jwalk::rayon::prelude::*;
use jwalk::{Parallelism, WalkDir};

use crate::file_info::FileInfo;
use crate::files_db::FilesDB;

// The value was carefully tested and smaller numbers work better than higher.
const THREADS: usize = 4;

#[derive(thiserror::Error, Debug)]
pub enum LoaderError {
    #[error("Unable to access metadata: {inner}")]
    FailedToAccessMetadata {
        #[from]
        inner: std::io::Error,
    },
}

// pub fn load_files_vec(scan_path: &PathBuf) -> anyhow::Result<FilesDB> {
//     let paths = load_paths(scan_path)?;
//
//     let (sender, receiver) = channel();
//     paths.into_par_iter().for_each_with(sender, |sender, path| {
//         if let Ok(fi) = FileInfo::try_from(&path) {
//             sender.send((path, fi)).unwrap();
//         }
//     });
//
//     let mut db = FilesDB::new();
//     receiver.iter().for_each(|(path, info)| {
//         db.add(path, info);
//     });
//
//     Ok(db)
// }

fn load_paths(directory: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let paths = WalkDir::new(directory)
        .parallelism(Parallelism::RayonNewPool(THREADS))
        .skip_hidden(false)
        .into_iter()
        .filter_map(|res| res.map(|de| de.path()).ok())
        .collect::<Vec<_>>();

    Ok(paths)
}

pub fn load_multiple_paths(scan_paths: &[PathBuf]) -> FilesDB {
    let (sender, receiver) = channel();

    scan_paths
        .into_par_iter()
        .for_each_with(sender, |sender, path| {
            // if let Ok(infos) = load_files(&path) {
            let infos = load_files(&path);
            infos
                .into_iter()
                .for_each(|fi| sender.send((fi.path.clone(), fi)).unwrap());
            // } else {
            //     eprintln!("Unable to load path '{}'", path.display());
            // }
        });

    let mut db = FilesDB::new();
    receiver.iter().for_each(|(path, info)| {
        db.add(path, info);
    });

    db
}

fn load_files(directory: &PathBuf) -> Vec<FileInfo> {
    let paths = WalkDir::new(directory)
        .parallelism(Parallelism::RayonNewPool(THREADS))
        .skip_hidden(false)
        .into_iter()
        .filter_map(|res| res.map(|de| de.path()).ok())
        .filter_map(|path| FileInfo::try_from(&path).ok())
        .collect::<Vec<_>>();

    paths
}
