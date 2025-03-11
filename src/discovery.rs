use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tracing::instrument;

use crate::config::{Config, CustomPathDefinition};
use crate::files_db::FilesDB;
use crate::loader::FullyParallelLoader;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Language {
    Python,
    Rust,
    Projects,
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Python => write!(f, "🐍"),
            Language::Rust => write!(f, "🦀"),
            Language::Projects => write!(f, ""),
        }
    }
}

#[derive(Debug)]
pub struct DetectedResult {
    pub lang: Language,
    pub path: PathBuf,
    pub size: u64,
    pub last_update: Option<SystemTime>,
}

#[derive(Debug)]
pub struct DiscoveryDefinition {
    pub lang: Language,
    pub discovery: bool,
    pub description: String,
    pub path: PathBuf,
    pub results: Vec<DetectedResult>,
}

pub trait PathLoader: Default {
    // There should be better encapsulation than this
    fn load_multiple_paths(&self, scan_paths: &[PathBuf]) -> FilesDB;
}

fn default_discovery_definitions() -> Vec<DiscoveryDefinition> {
    vec![
        DiscoveryDefinition {
            lang: Language::Rust,
            discovery: false,
            description: "Cargo registry".into(),
            path: ".cargo/registry".into(),
            results: vec![],
        },
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: false,
            description: "Poetry cache".into(),
            path: "Library/Caches/pypoetry".into(),
            results: vec![],
        },
        DiscoveryDefinition {
            lang: Language::Python,
            discovery: false,
            description: "uv cache".into(),
            path: ".cache/uv".into(),
            results: vec![],
        },
    ]
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
        if let Some(r) = define_from_section(&self.home, &config.python, Language::Python) {
            self.definitions.extend(r)
        }
        if let Some(r) = define_from_section(&self.home, &config.rust, Language::Rust) {
            self.definitions.extend(r)
        }

        self.definitions.extend(
            config
                .projects
                .iter()
                .map(|pd| DiscoveryDefinition {
                    lang: Language::Projects,
                    discovery: true,
                    description: pd.name.clone().unwrap_or("Projects".into()),
                    path: pd.path.clone(),
                    results: vec![],
                })
                .collect::<Vec<_>>(),
        );
        self
    }

    pub fn collect(mut self) -> Vec<DiscoveryDefinition> {
        self.resolve_relative_paths();
        self.load_paths();
        self.discover();

        self.definitions
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
    fn discover(&mut self) {
        for pd in self.definitions.iter_mut() {
            match (pd.lang, pd.discovery) {
                (_, false) => {
                    let size = self.db.iter_dir(&pd.path).filter_map(|fi| fi.size).sum();
                    let last_update = self.db.iter_dir(&pd.path).filter_map(|fi| fi.touched).max();
                    pd.results.push(DetectedResult {
                        lang: pd.lang,
                        path: pd.path.clone(),
                        last_update,
                        size,
                    });
                }
                (Language::Rust, true) => {
                    dynamic_discovery(&self.db, pd, rust_detector, Language::Rust);
                }
                (Language::Python, true) => {
                    dynamic_discovery(&self.db, pd, python_detector, Language::Python);
                }
                (Language::Projects, true) => {
                    dynamic_discovery(&self.db, pd, rust_detector, Language::Rust);
                    dynamic_discovery(&self.db, pd, python_detector, Language::Python);
                }
            }
        }
    }
}

fn dynamic_discovery<D>(db: &FilesDB, pd: &mut DiscoveryDefinition, detector: D, lang: Language)
where
    D: Fn(&FilesDB, &Path) -> bool,
{
    let detected_paths: Vec<&PathBuf> = db
        .iter_directories(&pd.path)
        .filter(|fi| detector(db, &fi.path))
        .map(|fi| &fi.path)
        .collect();
    detected_paths.iter().for_each(|p| {
        let size = db.iter_dir(p).filter_map(|fi| fi.size).sum();
        let last_update = db.iter_dir(p).filter_map(|fi| fi.touched).max();
        pd.results.push(DetectedResult {
            lang,
            path: (*p).clone(),
            last_update,
            size,
        });
    });
}

fn rust_detector(db: &FilesDB, path: &Path) -> bool {
    path.ends_with("target")
        && (db.is_dir(&path.join("debug/build")) || db.is_dir(&path.join("release/build")))
}

fn python_detector(db: &FilesDB, path: &Path) -> bool {
    db.exists(&path.join("bin/python"))
}

fn define_from_section(
    home: &Path,
    section: &Option<Vec<CustomPathDefinition>>,
    language: Language,
) -> Option<Vec<DiscoveryDefinition>> {
    section.as_ref().map(|v| {
        v.iter()
            .map(|pd| DiscoveryDefinition {
                lang: language,
                discovery: pd.discovery,
                description: pd.name.clone(),
                path: home.join(&pd.path),
                results: vec![],
            })
            .collect::<Vec<_>>()
    })
}
