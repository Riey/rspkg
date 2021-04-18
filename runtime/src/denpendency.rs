use crate::{Error, Project, Result};
use dashmap::{mapref::one::Ref, DashMap};

#[derive(Default)]
pub struct DependencyStore {
    projects: DashMap<String, Project>,
}

impl DependencyStore {
    pub fn has_project(&self, project_name: &str) -> bool {
        self.projects.contains_key(project_name)
    }

    pub fn add_project(&self, project: impl Into<Project>) {
        let project = project.into();
        self.projects.insert(project.id().into(), project);
    }

    pub fn get_project(&self, project_name: &str) -> Result<Ref<String, Project>> {
        self.projects
            .get(project_name)
            .ok_or_else(|| Error::CrateNotFound(project_name.into()))
    }
}
