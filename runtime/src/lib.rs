mod build;
mod denpendency;
mod error;
#[cfg(feature = "fetch")]
mod fetch;
mod project;
mod rustc_flags;

pub use crate::build::{BuildArtifacts, BuildEnvironment, Profile};
pub use crate::denpendency::DependencyStore;
pub use crate::error::{Error, Result};
#[cfg(feature = "fetch")]
pub use crate::fetch::CratesIoRegistry;
pub use crate::project::{Dependency, LocalProject, Project, RspkgProject};
pub use crate::rustc_flags::RustcFlags;
pub use rspkg_shared::*;

use std::process::ExitStatus;

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
