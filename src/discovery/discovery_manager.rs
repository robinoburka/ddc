use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use tracing::{debug_span, instrument, warn};

use crate::config::Config;
use crate::discovery::default_definitions::default_discovery_definitions;
use crate::discovery::detectors::{JsNpmDetector, PythonVenvDetector, RustBuildDirDetector};
use crate::discovery::discovery_definitions::ResultType;
use crate::discovery::progress::{ProgressEvent, ProgressReporter};
use crate::discovery::{DiscoveryDefinition, DiscoveryResult};
use crate::files_db::FilesDB;
use crate::loader::FullyParallelLoader;
use crate::types::Language;

pub trait PathLoader: Default {
    fn load_multiple_paths<R: ProgressReporter>(
        &self,
        scan_paths: &[PathBuf],
        progress: Option<R>,
    ) -> FilesDB;
}

pub trait DynamicDetector: Default + Send + Sync + 'static {
    const LANG: Language;
    fn detect(&self, db: &FilesDB, path: &Path) -> bool;
}

#[derive(Clone)]
pub struct ChannelProgressReporter {
    tx: Sender<ProgressEvent>,
}

impl ChannelProgressReporter {
    pub fn new(tx: Sender<ProgressEvent>) -> Self {
        Self { tx }
    }
}

impl ProgressReporter for ChannelProgressReporter {
    fn report(&self, event: ProgressEvent) {
        let _ = self.tx.try_send(event);
    }
}

pub struct DiscoveryManager<L: PathLoader> {
    home: PathBuf,
    loader: L,
    db: Arc<FilesDB>,
    definitions: Arc<Vec<DiscoveryDefinition>>,
    progress_tx: Sender<ProgressEvent>,
    progress_rx: Receiver<ProgressEvent>,
}

impl DiscoveryManager<FullyParallelLoader> {
    pub fn with_default_loader(home: &Path) -> Self {
        let (progress_tx, progress_rx) = channel::bounded(100);

        Self {
            home: home.to_path_buf(),
            loader: FullyParallelLoader,
            db: Arc::new(FilesDB::new()),
            definitions: Arc::new(default_discovery_definitions(home)),
            progress_tx,
            progress_rx,
        }
    }
}

impl<L: PathLoader> DiscoveryManager<L> {
    #[allow(dead_code)]
    pub fn new(loader: L, home: &Path) -> Self {
        let (progress_tx, progress_rx) = channel::bounded(100);

        Self {
            home: home.to_path_buf(),
            loader,
            db: Arc::new(FilesDB::new()),
            definitions: Arc::new(default_discovery_definitions(home)),
            progress_tx,
            progress_rx,
        }
    }

    pub fn add_from_config(mut self, config: &Config) -> Self {
        let config_definitions = config
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
                    path: self.home.join(&pd.path),
                }
            })
            .collect::<Vec<_>>();

        self.definitions = Arc::try_unwrap(self.definitions)
            .map(|mut inner| {
                inner.extend(config_definitions);
                Arc::new(inner)
            })
            .expect("Arc is still shared. Programmer error?");
        self
    }

    pub fn subscribe(&self) -> Receiver<ProgressEvent> {
        self.progress_rx.clone()
    }

    fn create_reporter(&self) -> ChannelProgressReporter {
        ChannelProgressReporter::new(self.progress_tx.clone())
    }

    pub fn collect(mut self) -> Vec<DiscoveryResult> {
        self.load_paths();
        let results = self.discover();
        drop(self.progress_tx);
        results
    }

    #[instrument(level = "debug", skip(self))]
    fn load_paths(&mut self) {
        let paths = self
            .definitions
            .iter()
            .map(|def| def.path.clone())
            .collect::<Vec<_>>();
        let reporter = self.create_reporter();
        self.db = Arc::new(self.loader.load_multiple_paths(&paths, Some(reporter)));
    }

    #[instrument(level = "debug", skip(self))]
    fn discover(&mut self) -> Vec<DiscoveryResult> {
        let reporter = self.create_reporter();
        reporter.report(ProgressEvent::DiscoveryStart { count: 4 });

        let mut results = vec![];
        let (tx, rx) = channel::unbounded();
        spawn_discovery_thread(
            self.db.clone(),
            self.definitions.clone(),
            RustBuildDirDetector,
            tx.clone(),
            self.create_reporter(),
        );
        spawn_discovery_thread(
            self.db.clone(),
            self.definitions.clone(),
            PythonVenvDetector,
            tx.clone(),
            self.create_reporter(),
        );
        spawn_discovery_thread(
            self.db.clone(),
            self.definitions.clone(),
            JsNpmDetector,
            tx.clone(),
            self.create_reporter(),
        );

        let db = self.db.clone();
        let definitions = self.definitions.clone();
        let thread_reporter = reporter.clone();
        rayon::spawn(move || {
            let _guard = debug_span!("static_thread").entered();
            for pd in definitions.iter() {
                if !pd.discovery {
                    let size = db.iter_dir(&pd.path).filter_map(|fi| fi.size).sum();
                    let last_update = db.iter_dir(&pd.path).filter_map(|fi| fi.touched).max();
                    let r = DiscoveryResult {
                        result_type: ResultType::Static(pd.description.clone()),
                        lang: pd.lang,
                        path: pd.path.clone(),
                        last_update,
                        size,
                    };
                    tx.send(r).unwrap();
                }
            }
            thread_reporter.report(ProgressEvent::DiscoveryAdvance)
        });

        for res in rx.iter() {
            results.push(res);
        }

        reporter.report(ProgressEvent::DiscoveryFinished);

        results
    }
}

fn spawn_discovery_thread<D, R>(
    db: Arc<FilesDB>,
    definitions: Arc<Vec<DiscoveryDefinition>>,
    detector: D,
    tx: Sender<DiscoveryResult>,
    progress: R,
) where
    D: DynamicDetector,
    R: ProgressReporter,
{
    rayon::spawn(move || {
        let _guard = debug_span!("discovery_thread", lang = ?D::LANG).entered();
        discovery_thread(db, definitions, detector, tx, progress);
    });
}

fn discovery_thread<D, R>(
    db: Arc<FilesDB>,
    discovery_definitions: Arc<Vec<DiscoveryDefinition>>,
    detector: D,
    tx: Sender<DiscoveryResult>,
    progress: R,
) where
    D: DynamicDetector,
    R: ProgressReporter,
{
    for definition in discovery_definitions.iter() {
        if definition.discovery {
            let detected_paths: Vec<&PathBuf> = db
                .iter_directories(&definition.path)
                .filter(|fi| detector.detect(db.deref(), fi.path))
                .map(|fi| fi.path)
                .collect();
            detected_paths.iter().for_each(|p| {
                let size = db.iter_dir(p).filter_map(|fi| fi.size).sum();
                let last_update = db.iter_dir(p).filter_map(|fi| fi.touched).max();
                let r = DiscoveryResult {
                    result_type: ResultType::Discovery,
                    lang: Some(D::LANG),
                    path: (*p).clone(),
                    last_update,
                    size,
                };
                tx.send(r).unwrap();
            });
        }
    }
    progress.report(ProgressEvent::DiscoveryAdvance);
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::string::String;

    use tempfile::tempdir;

    use super::*;
    use crate::config::PathDefinition;
    use crate::discovery::discovery_definitions::ResultType;

    #[test]
    fn test_discovery_manager() {
        let tmp = tempdir().unwrap();
        let root_path = tmp.path();
        fs::create_dir_all(root_path.join("projects/rust/target/release/build")).unwrap();
        fs::write(
            root_path.join("projects/rust/target/release/build/file"),
            "Executable mock",
        )
        .unwrap();
        fs::create_dir_all(root_path.join("projects/python/venv/bin")).unwrap();
        fs::write(
            root_path.join("projects/python/venv/bin/python"),
            "Python executable mock",
        )
        .unwrap();
        fs::create_dir_all(root_path.join(".cache/uv")).unwrap();
        fs::write(
            root_path.join(".cache/uv/CACHEDIR.TAG"),
            "Signature: 8a477f597d28d172789f06886806bc55",
        )
        .unwrap();
        fs::create_dir_all(root_path.join("to_sum")).unwrap();
        fs::write(root_path.join("to_sum/foo.txt"), "Hello, World!").unwrap();

        let config = Config {
            paths: vec![
                PathDefinition {
                    path: root_path.join("projects").to_path_buf(),
                    discovery: true,
                    name: Some(String::from("Projects")),
                    language: None,
                },
                PathDefinition {
                    path: root_path.join("to_sum").to_path_buf(),
                    discovery: false,
                    name: Some(String::from("Just to check")),
                    language: Some(String::from("rust")),
                },
            ],
        };

        let discovery_manager =
            DiscoveryManager::with_default_loader(root_path).add_from_config(&config);
        let progress = discovery_manager.subscribe();
        let discovery_results = discovery_manager.collect();

        let mut found_paths: Vec<_> = discovery_results.iter().filter(|r| r.size > 0).collect();
        found_paths.sort_by_key(|r| r.path.clone());

        assert_eq!(found_paths.len(), 4);

        // Coming from Default definitions
        assert_eq!(
            found_paths[0].result_type,
            ResultType::Static(String::from("uv cache"))
        );
        assert_eq!(found_paths[0].path, root_path.join(".cache/uv"));
        assert_eq!(found_paths[0].lang, Some(Language::Python));
        let dirs_size = fs::metadata(&root_path.join(".cache/uv")).unwrap().len();
        assert_eq!(found_paths[0].size, dirs_size + 43);

        // Coming from config - discovery
        assert_eq!(found_paths[1].result_type, ResultType::Discovery);
        assert_eq!(found_paths[1].path, root_path.join("projects/python/venv"));
        assert_eq!(found_paths[1].lang, Some(Language::Python));
        let dirs_size: u64 = vec![
            &root_path.join("projects/python"),
            &root_path.join("projects/python/venv"),
        ]
        .iter()
        .map(|p| fs::metadata(p).unwrap().len())
        .sum();
        assert_eq!(found_paths[1].size, dirs_size + 22);

        assert_eq!(found_paths[2].result_type, ResultType::Discovery);
        assert_eq!(found_paths[2].path, root_path.join("projects/rust/target"));
        assert_eq!(found_paths[2].lang, Some(Language::Rust));
        let dirs_size: u64 = vec![
            &root_path.join("projects/rust"),
            &root_path.join("projects/rust/target"),
            &root_path.join("projects/rust/target/release"),
        ]
        .iter()
        .map(|p| fs::metadata(p).unwrap().len())
        .sum();
        assert_eq!(found_paths[2].size, dirs_size + 15);

        // Coming from config - non-discoverable
        assert_eq!(
            found_paths[3].result_type,
            ResultType::Static(String::from("Just to check"))
        );
        assert_eq!(found_paths[3].path, root_path.join("to_sum"));
        assert_eq!(found_paths[3].lang, Some(Language::Rust));
        let dirs_size = fs::metadata(&root_path.join("to_sum")).unwrap().len();
        assert_eq!(found_paths[3].size, dirs_size + 13);

        // Quick check of the most important events collected from progress reporter
        let progress_report = progress.iter().collect::<Vec<_>>();
        assert!(progress_report.contains(&ProgressEvent::WalkStart {
            count: config.paths.len() + default_discovery_definitions(root_path).len()
        }));
        // 2 files in projects/ mocked path
        assert!(progress_report.contains(&ProgressEvent::WalkAddPaths { count: 2 }));
        assert!(progress_report.contains(&ProgressEvent::WalkFinished));
        // Current count of expected detectors 3 + 1 for non-discovery definitions
        assert!(progress_report.contains(&ProgressEvent::DiscoveryStart { count: 4 }));
        assert!(progress_report.contains(&ProgressEvent::DiscoveryAdvance));
        assert!(progress_report.contains(&ProgressEvent::DiscoveryFinished));
    }
}
