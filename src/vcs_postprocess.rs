use std::path::PathBuf;
use std::time::SystemTime;

use tracing::debug_span;

use crate::discovery::{ProjectResult, VcsResult};

#[derive(Debug)]
pub struct EnrichedVcsResult {
    pub path: PathBuf,
    pub size: u64,
    pub last_update: Option<SystemTime>,
    pub vcs_size: u64,
    pub matched_projects: Vec<ProjectResult>,
}

impl From<VcsResult> for EnrichedVcsResult {
    fn from(vcs: VcsResult) -> Self {
        EnrichedVcsResult {
            path: vcs.path,
            size: vcs.size,
            last_update: vcs.last_update,
            vcs_size: vcs.vcs_size,
            matched_projects: vec![],
        }
    }
}

pub fn vcs_postprocess(
    projects: &[ProjectResult],
    vcs_results: Vec<VcsResult>,
) -> Vec<EnrichedVcsResult> {
    let _guard = debug_span!("vcs_postprocess", projects = ?projects.len(), vcs_results = ?vcs_results.len()).entered();

    let mut results = vcs_results
        .into_iter()
        .map(EnrichedVcsResult::from)
        .collect::<Vec<_>>();

    for vcs_dir in results.iter_mut() {
        for project in projects {
            if project.path.starts_with(&vcs_dir.path) {
                vcs_dir.matched_projects.push(project.clone());
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::vcs_postprocess;
    use crate::discovery::{Language, ProjectResult, VcsResult};

    #[test]
    fn test_vcs_postprocess() {
        let projects = vec![
            ProjectResult {
                path: PathBuf::from("/home/user/projects/rust/target"),
                lang: Language::Rust,
                size: 100,
                last_update: None,
                parent: None,
            },
            ProjectResult {
                path: PathBuf::from("/home/user/projects/python/.venv"),
                lang: Language::Python,
                size: 200,
                last_update: None,
                parent: None,
            },
        ];
        let vcs_results = vec![
            VcsResult {
                path: PathBuf::from("/home/user/projects/rust"),
                size: 1000,
                last_update: None,
                vcs_size: 10,
            },
            VcsResult {
                path: PathBuf::from("/home/user/projects/experiments"),
                size: 1000,
                last_update: None,
                vcs_size: 20,
            },
        ];

        let enriched = vcs_postprocess(&projects, vcs_results);

        assert_eq!(enriched.len(), 2);
        assert_eq!(enriched[0].matched_projects.len(), 1);
        assert_eq!(
            enriched[0].matched_projects[0].path,
            PathBuf::from("/home/user/projects/rust/target")
        );
        assert_eq!(enriched[1].matched_projects.len(), 0);
    }
}
