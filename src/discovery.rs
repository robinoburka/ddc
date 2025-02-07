use std::path::{Path, PathBuf};

use crate::config::{Config, CustomPathDefinition};
use crate::files_db::FilesDB;
use crate::loader::BaseLoader;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Language {
    PYTHON,
    RUST,
    PROJECTS,
}

#[derive(Debug)]
pub struct DetectedResult {
    pub lang: Language,
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Debug)]
pub struct DiscoveryDefinition {
    pub lang: Language,
    pub discovery: bool,
    pub description: String,
    pub path: PathBuf,
    pub results: Vec<DetectedResult>,
}

pub trait PathLoader {
    // There should be better encapsulation than this
    fn load_multiple_paths(&self, scan_paths: &[PathBuf]) -> FilesDB;
}

// pub fn discovery_definitions_from_config(home: &PathBuf, config: &Config) -> Vec<PathDefinition> {
//     let mut definitions = default_discovery_definitions(home);
//
//     define_from_section(home, &config.python, Language::PYTHON).map(|r| definitions.extend(r));
//     define_from_section(home, &config.rust, Language::RUST).map(|r| definitions.extend(r));
//
//     definitions.extend(
//         config
//             .projects
//             .iter()
//             .map(|pd| PathDefinition {
//                 lang: Language::PROJECTS,
//                 discovery: true,
//                 description: pd.name.clone().unwrap_or("Projects".into()),
//                 path: pd.path.clone(),
//             })
//             .collect::<Vec<_>>(),
//     );
//
//     definitions
// }

// fn default_discovery_definitions(home: &PathBuf) -> Vec<PathDefinition> {
//     vec![
//         PathDefinition {
//             lang: Language::RUST,
//             discovery: false,
//             description: "Cargo registry".into(),
//             path: home.join(".cargo/registry".into()),
//         },
//         PathDefinition {
//             lang: Language::PYTHON,
//             discovery: false,
//             description: "Poetry cache".into(),
//             path: home.join("Library/Caches/pypoetry".into()),
//         },
//         PathDefinition {
//             lang: Language::PYTHON,
//             discovery: false,
//             description: "uv cache".into(),
//             path: home.join(".cache/uv".into()),
//         },
//     ]
// }

fn default_discovery_definitions() -> Vec<DiscoveryDefinition> {
    vec![
        DiscoveryDefinition {
            lang: Language::RUST,
            discovery: false,
            description: "Cargo registry".into(),
            path: ".cargo/registry".into(),
            results: vec![],
        },
        DiscoveryDefinition {
            lang: Language::PYTHON,
            discovery: false,
            description: "Poetry cache".into(),
            path: "Library/Caches/pypoetry".into(),
            results: vec![],
        },
        DiscoveryDefinition {
            lang: Language::PYTHON,
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

impl DiscoveryManager<BaseLoader> {
    pub fn with_default_loader(home: &PathBuf) -> Self {
        Self {
            home: home.clone(),
            loader: BaseLoader::new(),
            db: FilesDB::new(),
            definitions: default_discovery_definitions(),
        }
    }
}

impl<L: PathLoader> DiscoveryManager<L> {
    #[allow(dead_code)]
    pub fn new(loader: L, home: &PathBuf) -> Self {
        Self {
            home: home.clone(),
            loader,
            db: FilesDB::new(),
            definitions: default_discovery_definitions(),
        }
    }

    pub fn add_from_config(mut self, config: &Config) -> Self {
        define_from_section(&self.home, &config.python, Language::PYTHON)
            .map(|r| self.definitions.extend(r));
        define_from_section(&self.home, &config.rust, Language::RUST)
            .map(|r| self.definitions.extend(r));

        self.definitions.extend(
            config
                .projects
                .iter()
                .map(|pd| DiscoveryDefinition {
                    lang: Language::PROJECTS,
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

    fn resolve_relative_paths(&mut self) {
        self.definitions.iter_mut().for_each(|def| {
            def.path = self.home.join(&def.path);
        });
    }

    fn load_paths(&mut self) {
        let paths = self
            .definitions
            .iter()
            .map(|def| def.path.clone())
            .collect::<Vec<_>>();
        self.db = self.loader.load_multiple_paths(&paths);
    }

    fn discover(&mut self) {
        for pd in self.definitions.iter_mut() {
            match (pd.lang, pd.discovery) {
                (_, false) => {
                    let size = self.db.iter_dir(&pd.path).filter_map(|fi| fi.size).sum();
                    pd.results.push(DetectedResult {
                        lang: pd.lang,
                        path: pd.path.clone(),
                        size,
                    });
                }
                (Language::RUST, true) => {
                    let detected_paths: Vec<&PathBuf> = self
                        .db
                        .iter_directories(&pd.path)
                        .filter(|file| rust_detector(&self.db, &file.path))
                        .map(|file| &file.path)
                        .collect();
                    detected_paths.iter().for_each(|p| {
                        let size = self.db.iter_dir(&p).filter_map(|fi| fi.size).sum();
                        pd.results.push(DetectedResult {
                            lang: pd.lang,
                            path: pd.path.clone(),
                            size,
                        });
                    })
                }
                (Language::PYTHON, true) => {
                    let detected_paths: Vec<&PathBuf> = self
                        .db
                        .iter_directories(&pd.path)
                        .filter(|file| python_detector(&self.db, &file.path))
                        .map(|file| &file.path)
                        .collect();
                    detected_paths.iter().for_each(|p| {
                        let size = self.db.iter_dir(&p).filter_map(|fi| fi.size).sum();
                        pd.results.push(DetectedResult {
                            lang: pd.lang,
                            path: pd.path.clone(),
                            size,
                        });
                    })
                }
                (Language::PROJECTS, true) => {
                    let detected_paths: Vec<&PathBuf> = self
                        .db
                        .iter_directories(&pd.path)
                        .filter(|file| python_detector(&self.db, &file.path))
                        .map(|file| &file.path)
                        .collect();
                    detected_paths.iter().for_each(|p| {
                        let size = self.db.iter_dir(&p).filter_map(|fi| fi.size).sum();
                        pd.results.push(DetectedResult {
                            lang: Language::PYTHON,
                            path: (*p).clone(),
                            size,
                        });
                    });
                    let detected_paths: Vec<&PathBuf> = self
                        .db
                        .iter_directories(&pd.path)
                        .filter(|file| rust_detector(&self.db, &file.path))
                        .map(|file| &file.path)
                        .collect();
                    detected_paths.into_iter().for_each(|p| {
                        let size = self.db.iter_dir(&p).filter_map(|fi| fi.size).sum();
                        pd.results.push(DetectedResult {
                            lang: Language::RUST,
                            path: (*p).clone(),
                            size,
                        });
                    })
                }
            }
        }
    }
}

fn rust_detector(db: &FilesDB, path: &Path) -> bool {
    path.ends_with("target")
        && (db.is_dir(&path.join("debug/build")) || db.is_dir(&path.join("release/build")))
}

fn python_detector(db: &FilesDB, path: &Path) -> bool {
    db.exists(&path.join("bin/python"))
}

fn define_from_section(
    home: &PathBuf,
    section: &Option<Vec<CustomPathDefinition>>,
    language: Language,
) -> Option<Vec<DiscoveryDefinition>> {
    match &section {
        None => None,
        Some(v) => Some(
            v.iter()
                .map(|pd| DiscoveryDefinition {
                    lang: language,
                    discovery: pd.discovery,
                    description: pd.name.clone(),
                    path: home.join(&pd.path),
                    results: vec![],
                })
                .collect::<Vec<_>>(),
        ),
    }
}
