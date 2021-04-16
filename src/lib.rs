mod build;
mod error;
#[cfg(feature = "fetch")]
mod fetch;
mod project;

pub use crate::build::{BuildArtifacts, BuildEnvironment};
pub use crate::error::{Error, Result};
#[cfg(feature = "fetch")]
pub use crate::fetch::CratesIoRegistry;
pub use crate::project::{LocalProject, LocalProjectBuilder, Project, RspkgProject};
pub use serde;
pub use serde_json;

use serde::{Deserialize, Serialize};
use std::process::{Command, ExitStatus};

pub trait CheckResult<T> {
    fn check(&self, arg: T) -> Result<()>;
}

impl CheckResult<&str> for ExitStatus {
    fn check(&self, arg: &str) -> Result<()> {
        if self.success() {
            Ok(())
        } else {
            Err(Error::CommandError(arg.into(), *self))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CrateType {
    Bin,
    Lib,
    ProcMacro,
}

impl Default for CrateType {
    fn default() -> Self {
        Self::Lib
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Profile {
    Dev,
    Release,
}

impl Default for Profile {
    fn default() -> Self {
        Self::Dev
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Edition {
    Edition2015,
    Edition2018,
}

impl Default for Edition {
    fn default() -> Self {
        Self::Edition2018
    }
}

pub trait RustcFlags<T> {
    fn rustc_flags(&mut self, arg: T) -> &mut Self;
}

impl RustcFlags<Profile> for Command {
    fn rustc_flags(&mut self, arg: Profile) -> &mut Self {
        match arg {
            Profile::Dev => self.arg("-Cdebuginfo=2"),
            Profile::Release => self.arg("-Copt-level=3"),
        }
    }
}

impl RustcFlags<CrateType> for Command {
    fn rustc_flags(&mut self, arg: CrateType) -> &mut Self {
        match arg {
            CrateType::Bin => self.arg("--crate-type=bin"),
            CrateType::Lib => self.arg("--crate-type=lib"),
            CrateType::ProcMacro => self.arg("--crate-type=proc-macro"),
        }
    }
}

impl RustcFlags<Edition> for Command {
    fn rustc_flags(&mut self, arg: Edition) -> &mut Self {
        match arg {
            Edition::Edition2015 => self.arg("--edition=2015"),
            Edition::Edition2018 => self.arg("--edition=2018"),
        }
    }
}
