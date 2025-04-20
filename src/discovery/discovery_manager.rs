use std::path::{Path, PathBuf};

use tracing::{instrument, warn};

use crate::config::Config;
use crate::discovery::default_definitions::default_discovery_definitions;
use crate::discovery::detectors::{python_detector, rust_detector};
use crate::discovery::discovery_definitions::ResultType;
use crate::discovery::{DiscoveryDefinition, DiscoveryResult};
use crate::files_db::FilesDB;
use crate::loader::FullyParallelLoader;
use crate::types::Language;

pub trait PathLoader: Default {
    // There should be better encapsulation than this
    fn load_multiple_paths(&self, scan_paths: &[PathBuf]) -> FilesDB;
}

pub struct DiscoveryManager<L: PathLoader> {
    home: PathBuf,
    loader: L,
    db: FilesDB,
    definitions: Vec<DiscoveryDefinition>,
}

impl DiscoveryManager<FullyParallelLoader> {
    pub fn with_default_loader(home: &Path) -> Self {
        Self {
            home: home.to_path_buf(),
            loader: FullyParallelLoader,
            db: FilesDB::new(),
            definitions: default_discovery_definitions(),
        }
    }
}

impl<L: PathLoader> DiscoveryManager<L> {
    #[allow(dead_code)]
    pub fn new(loader: L, home: &Path) -> Self {
        Self {
            home: home.to_path_buf(),
            loader,
            db: FilesDB::new(),
            definitions: default_discovery_definitions(),
        }
    }

    pub fn add_from_config(mut self, config: &Config) -> Self {
        self.definitions.extend(
            config
                .paths
                .iter()
                .map(|pd| {
                    let lang = match pd.language.as_ref() {
                        None => None,
                        Some(lang) => match Language::try_from(lang) {
                            Ok(l) => Some(l),
                            Err(e) => {
                                warn!("Unknown language definition: {}", e);
                                None
                            }
                        },
                    };
                    DiscoveryDefinition {
                        lang,
                        discovery: pd.discovery,
                        description: pd.name.clone().unwrap_or("Projects".into()),
                        path: pd.path.clone(),
                    }
                })
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn collect(mut self) -> Vec<DiscoveryResult> {
        self.resolve_relative_paths();
        self.load_paths();
        self.discover()
    }

    #[instrument(level = "debug", skip(self))]
    fn resolve_relative_paths(&mut self) {
        self.definitions.iter_mut().for_each(|def| {
            def.path = self.home.join(&def.path);
        });
    }

    #[instrument(level = "debug", skip(self))]
    fn load_paths(&mut self) {
        let paths = self
            .definitions
            .iter()
            .map(|def| def.path.clone())
            .collect::<Vec<_>>();
        self.db = self.loader.load_multiple_paths(&paths);
    }

    #[instrument(level = "debug", skip(self))]
    fn discover(&mut self) -> Vec<DiscoveryResult> {
        let mut results = vec![];
        for pd in self.definitions.iter() {
            if pd.discovery {
                results.extend(dynamic_discovery(
                    &self.db,
                    &pd.path,
                    rust_detector,
                    Language::Rust,
                ));
                results.extend(dynamic_discovery(
                    &self.db,
                    &pd.path,
                    python_detector,
                    Language::Python,
                ));
            } else {
                let size = self.db.iter_dir(&pd.path).filter_map(|fi| fi.size).sum();
                let last_update = self.db.iter_dir(&pd.path).filter_map(|fi| fi.touched).max();
                results.push(DiscoveryResult {
                    result_type: ResultType::Static(pd.description.clone()),
                    lang: pd.lang,
                    path: pd.path.clone(),
                    last_update,
                    size,
                });
            }
        }

        results
    }
}

fn dynamic_discovery<D>(
    db: &FilesDB,
    path: &PathBuf,
    detector: D,
    lang: Language,
) -> Vec<DiscoveryResult>
where
    D: Fn(&FilesDB, &Path) -> bool,
{
    let detected_paths: Vec<&PathBuf> = db
        .iter_directories(path)
        .filter(|fi| detector(db, &fi.path))
        .map(|fi| &fi.path)
        .collect();
    detected_paths
        .iter()
        .map(|p| {
            let size = db.iter_dir(p).filter_map(|fi| fi.size).sum();
            let last_update = db.iter_dir(p).filter_map(|fi| fi.touched).max();
            DiscoveryResult {
                result_type: ResultType::Discovery,
                lang: Some(lang),
                path: (*p).clone(),
                last_update,
                size,
            }
        })
        .collect::<Vec<_>>()
}
