use wasmer::{Singlepass, Store, JIT};

use crate::{Error, Project, Result};
use std::path::PathBuf;
use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
};

pub struct BuildEnvironment {
    profile: Profile,
    projects: HashMap<String, Project>,
    out_dir: PathBuf,

    store: Store,
}

impl BuildEnvironment {
    pub fn new(profile: Profile, out_dir: PathBuf) -> Self {
        let engine = JIT::new(Singlepass::new());
        let store = Store::new(&engine.engine());

        Self {
            profile,
            out_dir,
            store,
            projects: HashMap::new(),
        }
    }

    pub fn store(&self) -> &Store {
        &self.store
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
        eprintln!("Add project: {}", project.id());
        self.projects.insert(project.id().into(), project);
    }

    pub fn get_project(&self, project_name: &str) -> Result<&Project> {
        self.projects
            .get(project_name)
            .ok_or_else(|| Error::CrateNotFound(project_name.into()))
    }

    /// Load all dependencies into env
    pub fn prepare_deps(&mut self) -> Result<()> {
        let new_projects = Arc::new(Mutex::new(self.projects.clone()));
        for project in self.projects.values() {
            project.dependencies(self, &new_projects)?;
        }
        self.projects = if let Ok(p) = Arc::try_unwrap(new_projects) {
            p.into_inner().unwrap()
        } else {
            unreachable!()
        };

        Ok(())
    }

    pub fn build(&self, project_name: &str) -> Result<BuildArtifacts> {
        self.get_project(project_name)?.build(self)
    }
}

pub struct BuildArtifacts {
    pub out: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Profile {
    Dev,
    Release,
}

impl Default for Profile {
    fn default() -> Self {
        Self::Dev
    }
}
