#![no_std]

rspkg::nostd_template!();

use rspkg::{build_file, Artifact, CrateType, Edition};

#[no_mangle]
pub extern "C" fn build() -> Artifact {
    build_file("hello", "./hello.rs", CrateType::Bin, Edition::Edition2018)
}
