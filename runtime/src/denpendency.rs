use crate::{BuildArtifacts, Error, Project, Result};
use dashmap::{mapref::one::Ref, DashMap};
use std::sync::atomic::{AtomicU32, Ordering::SeqCst};

#[derive(Default)]
pub struct DependencyStore {
    next: AtomicU32,
    projects: DashMap<String, Project>,
    artifacts: DashMap<u32, BuildArtifacts>,
}

impl DependencyStore {
    pub fn get_artifact(&self, index: u32) -> Result<Ref<u32, BuildArtifacts>> {
        self.artifacts
            .get(&index)
            .ok_or_else(|| Error::ArtifactNotFound(index))
    }

    pub fn add_artifact(&self, artifact: BuildArtifacts) -> u32 {
        let next = self.next.fetch_add(1, SeqCst);

        assert!(self.artifacts.insert(next, artifact).is_none());

        next
    }

    pub fn has_project(&self, project_name: &str) -> bool {
        self.projects.contains_key(project_name)
    }

    pub fn add_project(&self, project: Project) {
        self.projects.insert(project.name().into(), project);
    }

    pub fn get_project(&self, project_name: &str) -> Result<Ref<String, Project>> {
        self.projects
            .get(project_name)
            .ok_or_else(|| Error::CrateNotFound(project_name.into()))
    }
}
