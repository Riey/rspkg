use rspkg_plugin::Interner;
use rspkg_plugin_rustc::RustcPlugin;
use rspkg_runtime::{build_manifest_bin, build_manifest_lib, Manifest, Result};
use std::path::{Path, PathBuf};
use std::{env, sync::Arc};

fn main() -> Result<()> {
    let arg = env::args().nth(1).expect("No argument");
    let tmp_dir = PathBuf::from("rspkg-result");
    let interner = Arc::new(Interner::new());
    let plugins = vec![Box::new(RustcPlugin {
        out_dir: tmp_dir.clone(),
    }) as Box<_>];

    build_manifest_lib(
        "rspkg_plugin_rustc_ffi",
        Path::new("plugins/rustc-ffi/src/lib.rs"),
        &[],
        &tmp_dir,
    )?;
    let manifest_bin = build_manifest_bin(
        "root",
        Path::new(&arg),
        &["rspkg_plugin_rustc_ffi"],
        &tmp_dir,
    )?;
    let manifest = Manifest::new(&manifest_bin, &interner, &plugins)?;
    manifest.build();

    Ok(())
}
