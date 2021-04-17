use crate::{CrateType, Edition, Profile};
use std::process::Command;

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
            CrateType::Cdylib => self.arg("--crate-type=cdylib"),
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
