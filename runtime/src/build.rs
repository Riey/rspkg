use std::path::Path;
use std::path::PathBuf;

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

    pub fn out_dir(&self) -> &Path {
        &self.out_dir
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
