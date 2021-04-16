use crate::{Error, Profile, Project, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

pub struct BuildEnvironment {
    profile: Profile,
    projects: HashMap<String, Project>,
    out_dir: PathBuf,
}

impl BuildEnvironment {
    pub fn new(profile: Profile, out_dir: PathBuf) -> Self {
        Self {
            profile,
            out_dir,
            projects: HashMap::new(),
        }
    }

    pub fn out_dir(&self) -> &Path {
        &self.out_dir
    }

    pub fn profile(&self) -> Profile {
        self.profile
    }

    pub fn has_project(&self, project_name: &str) -> bool {
        self.projects.contains_key(project_name)
    }

    pub fn add_project(&mut self, project: Project) {
        self.projects.insert(project.id().into(), project);
    }

    pub fn get_project(&self, project_name: &str) -> Result<&Project> {
        self.projects
            .get(project_name)
            .ok_or_else(|| Error::CrateNotFound(project_name.into()))
    }

    /// Load all dependencies into env
    pub fn prepare_deps(&mut self) -> Result<()> {
        let mut new_projects = self.projects.clone();
        for project in self.projects.values() {
            let deps = project.dependencies(self)?;
            for dep in deps {
                new_projects.insert(dep.id().into(), dep);
            }
        }
        self.projects = new_projects;

        Ok(())
    }

    pub fn build(&self, project_name: &str) -> Result<BuildArtifacts> {
        self.get_project(project_name)?.build(self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct BuildArtifacts {
    pub out: PathBuf,
}
