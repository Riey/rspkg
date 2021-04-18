use std::path::{Path, PathBuf};

use rspkg_shared::DependencyType;

#[derive(Clone)]
pub struct BuildEnvironment {
    profile: Profile,
    out_dir: PathBuf,
    build_target: Option<String>,
    host_target: Option<String>,
}

impl BuildEnvironment {
    pub fn new(
        profile: Profile,
        out_dir: PathBuf,
        build_target: Option<String>,
        host_target: Option<String>,
    ) -> Self {
        Self {
            profile,
            out_dir,
            build_target,
            host_target,
        }
    }

    pub fn build_target(&self) -> Option<&str> {
        self.build_target.as_deref()
    }

    pub fn host_target(&self) -> Option<&str> {
        self.host_target.as_deref()
    }

    pub fn get_target(&self, dep_ty: DependencyType) -> Option<&str> {
        match dep_ty {
            DependencyType::Normal => self.host_target(),
            DependencyType::Build | DependencyType::Dev => self.build_target(),
            DependencyType::Manifest => Some("wasm32-unknown-unknown"),
        }
    }

    pub fn out_dir(&self) -> &Path {
        &self.out_dir
    }

    pub fn target_out_dir(&self, dep_ty: DependencyType) -> PathBuf {
        let dir = if let Some(t) = self.get_target(dep_ty) {
            self.out_dir.join(t)
        } else {
            self.out_dir.clone()
        };

        if !dir.exists() {
            std::fs::create_dir(&dir).ok();
        }

        dir
    }

    pub fn profile(&self) -> Profile {
        self.profile
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
