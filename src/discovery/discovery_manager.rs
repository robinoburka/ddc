use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crossbeam::channel;
use crossbeam::channel::{Receiver, Sender};
use tracing::{debug_span, instrument, warn};

use crate::config::Config;
use crate::discovery::DiscoveryDefinition;
use crate::discovery::default_definitions::default_discovery_definitions;
use crate::discovery::detectors::{JsNpmDetector, PythonVenvDetector, RustBuildDirDetector};
use crate::discovery::discovery_definitions::{
    DiscoveryResults, ParentInfo, ProjectResult, ToolingResult,
};
use crate::discovery::progress::{ProgressEvent, ProgressReporter};
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

pub struct DiscoveryManager<L: PathLoader = FullyParallelLoader> {
    home: PathBuf,
    loader: L,
    db: Arc<FilesDB>,
    definitions: Arc<Vec<DiscoveryDefinition>>,
    progress_tx: Sender<ProgressEvent>,
    progress_rx: Receiver<ProgressEvent>,
}

#[derive(Debug)]
enum DiscoveryResult {
    Project(ProjectResult),
    Tool(ToolingResult),
}

impl DiscoveryManager {
    pub fn new(home: &Path) -> Self {
        Self::with_loader(Default::default(), home)
    }
}

impl<L: PathLoader> DiscoveryManager<L> {
    pub fn with_loader(loader: L, home: &Path) -> Self {
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

    pub fn collect(mut self) -> DiscoveryResults {
        self.load_paths();
        let (projects, tools) = self.discover();
        drop(self.progress_tx);

        DiscoveryResults {
            projects,
            tools,
            db: Arc::into_inner(self.db),
        }
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
    fn discover(&mut self) -> (Vec<ProjectResult>, Vec<ToolingResult>) {
        let reporter = self.create_reporter();
        reporter.report(ProgressEvent::DiscoveryStart { count: 4 });

        let mut project_results = vec![];
        let mut tooling_results = vec![];
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
                    let parent = pd
                        .path
                        .parent()
                        .map(|p| p.to_path_buf())
                        .filter(|p| db.exists(p));
                    let size = db.iter_dir(&pd.path).filter_map(|fi| fi.size).sum();
                    let last_update = db.iter_dir(&pd.path).filter_map(|fi| fi.touched).max();
                    let r = DiscoveryResult::Tool(ToolingResult {
                        description: pd.description.clone(),
                        lang: pd.lang,
                        path: pd.path.clone(),
                        last_update,
                        size,
                        parent: parent.map(|parent_path| ParentInfo {
                            size: db.iter_dir(&parent_path).filter_map(|fi| fi.size).sum(),
                            path: parent_path,
                        }),
                    });
                    tx.send(r).unwrap();
                }
            }
            thread_reporter.report(ProgressEvent::DiscoveryAdvance)
        });

        for res in rx.iter() {
            match res {
                DiscoveryResult::Project(r) => project_results.push(r),
                DiscoveryResult::Tool(r) => {
                    if r.size > 0 {
                        tooling_results.push(r)
                    }
                }
            }
        }

        reporter.report(ProgressEvent::DiscoveryFinished);

        (project_results, tooling_results)
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
                let parent = p.parent().map(|p| p.to_path_buf()).filter(|p| db.exists(p));
                let r = DiscoveryResult::Project(ProjectResult {
                    lang: D::LANG,
                    path: (*p).clone(),
                    last_update,
                    size,
                    parent: parent.map(|parent_path| ParentInfo {
                        size: db.iter_dir(&parent_path).filter_map(|fi| fi.size).sum(),
                        path: parent_path,
                    }),
                });
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

        let discovery_manager = DiscoveryManager::new(root_path).add_from_config(&config);
        let progress = discovery_manager.subscribe();
        let mut discovery_results = discovery_manager.collect();

        discovery_results.projects.sort_by_key(|r| r.path.clone());
        discovery_results.tools.sort_by_key(|r| r.path.clone());

        assert_eq!(discovery_results.projects.len(), 2);
        assert_eq!(discovery_results.tools.len(), 2);

        // Coming from config - discovery
        assert_eq!(
            discovery_results.projects[0].path,
            root_path.join("projects/python/venv")
        );
        assert_eq!(discovery_results.projects[0].lang, Language::Python);
        let dirs_size: u64 = vec![
            &root_path.join("projects/python"),
            &root_path.join("projects/python/venv"),
        ]
        .iter()
        .map(|p| fs::metadata(p).unwrap().len())
        .sum();
        assert_eq!(discovery_results.projects[0].size, dirs_size + 22);

        assert_eq!(
            discovery_results.projects[1].path,
            root_path.join("projects/rust/target")
        );
        assert_eq!(discovery_results.projects[1].lang, Language::Rust);
        let dirs_size: u64 = vec![
            &root_path.join("projects/rust"),
            &root_path.join("projects/rust/target"),
            &root_path.join("projects/rust/target/release"),
        ]
        .iter()
        .map(|p| fs::metadata(p).unwrap().len())
        .sum();
        assert_eq!(discovery_results.projects[1].size, dirs_size + 15);

        // Coming from Default definitions
        assert_eq!(
            discovery_results.tools[0].description,
            String::from("uv cache")
        );
        assert_eq!(discovery_results.tools[0].path, root_path.join(".cache/uv"));
        assert_eq!(discovery_results.tools[0].lang, Some(Language::Python));
        let dirs_size = fs::metadata(&root_path.join(".cache/uv")).unwrap().len();
        assert_eq!(discovery_results.tools[0].size, dirs_size + 43);

        // Coming from config - non-discoverable
        assert_eq!(
            discovery_results.tools[1].description,
            String::from("Just to check")
        );
        assert_eq!(discovery_results.tools[1].path, root_path.join("to_sum"));
        assert_eq!(discovery_results.tools[1].lang, Some(Language::Rust));
        let dirs_size = fs::metadata(&root_path.join("to_sum")).unwrap().len();
        assert_eq!(discovery_results.tools[1].size, dirs_size + 13);

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
