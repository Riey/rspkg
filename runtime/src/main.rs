use rspkg_runtime::{build_manifest_bin, build_manifest_lib, Interner, Manifest, Result};
use std::path::{Path, PathBuf};
use std::{collections::HashMap, env, sync::Arc};

fn main() -> Result<()> {
    let arg = env::args().nth(1).expect("No argument");
    let tmp_dir = PathBuf::from("rspkg-result");
    let interner = Arc::new(Interner::new());
    let plugins = HashMap::new();

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
    let manifest_bin =
        build_manifest_bin("root", Path::new(&arg), &["rspkg_plugin_rustc"], &tmp_dir)?;
    let manifest = Manifest::new(&manifest_bin, &interner, &plugins)?;
    manifest.build();

    Ok(())
}
