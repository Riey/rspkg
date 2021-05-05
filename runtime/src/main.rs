use rspkg_runtime::{build_manifest_bin, build_manifest_lib, Manifest, Result};
use std::env;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let arg = env::args().nth(1).expect("No argument");
    let tmp_dir = PathBuf::from("rspkg-result");

    build_manifest_lib(
        "rspkg_runtime_ffi",
        Path::new("runtime-ffi/src/lib.rs"),
        &[],
        &tmp_dir,
    )?;
    build_manifest_lib(
        "rspkg_plugin_rustc",
        Path::new("plugins/rustc/src/lib.rs"),
        &["rspkg_runtime_ffi"],
        &tmp_dir,
    )?;
    let manifest_bin = build_manifest_bin(
        "root",
        Path::new("manifest.rs"),
        &["rspkg_plugin_rustc"],
        &tmp_dir,
    )?;
    let manifest = Manifest::new(&manifest_bin)?;
    manifest.build();

    Ok(())
}
