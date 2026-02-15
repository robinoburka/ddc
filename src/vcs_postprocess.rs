use std::path::PathBuf;
use std::time::SystemTime;

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
