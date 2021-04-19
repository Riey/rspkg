#![no_std]

rspkg::nostd_template!();

use rspkg::{Artifact, CrateType, Dependency, DependencyType, Edition};

#[no_mangle]
pub extern "C" fn build() -> Artifact {
    let hello = Dependency::new("hello", DependencyType::Normal);

    hello.build("./hello.rs", CrateType::Bin, Edition::Edition2018)
}
