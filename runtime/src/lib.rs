mod error;
mod manifest;

pub use crate::error::{Error, Result};
pub use crate::manifest::{
    Manifest,
    build_manifest_bin,
    build_manifest_lib,
};

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
