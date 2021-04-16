mod build;
mod error;
mod project;

pub use crate::build::{BuildArtifacts, BuildEnvironment};
pub use crate::error::{Error, Result};
pub use crate::project::Project;

use std::process::{Command, ExitStatus};

pub trait CheckResult<T> {
    fn check(&self, arg: T) -> Result<()>;
}

impl CheckResult<&'static str> for ExitStatus {
    fn check(&self, arg: &'static str) -> Result<()> {
        if self.success() {
            Ok(())
        } else {
            Err(Error::CommandError(arg, *self))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CrateType {
    Bin,
    Lib,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Profile {
    Dev,
    Release,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Edition {
    Edition2015,
    Edition2018,
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
