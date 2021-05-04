mod error;
#[cfg(feature = "fetch")]
mod fetch;
mod manifest;

pub use crate::error::{Error, Result};
#[cfg(feature = "fetch")]
pub use crate::fetch::FetchClient;
pub use crate::manifest::Manifest;

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
