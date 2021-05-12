use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use crate::{CheckResult, Result};
use rspkg_plugin::{Interner, Plugin};
use wasmer::{ChainableNamedResolver, ImportObject, Instance, Module, NamedResolverChain, Store};
use wasmer_wasi::WasiState;

pub fn build_manifest_lib(
    crate_name: &str,
    root_file: &Path,
    externs: &[&str],
    out_dir: &Path,
) -> Result<PathBuf> {
    let out_path = out_dir.join(format!("lib{}.rlib", crate_name));

    let mut cmd = Command::new("rustc");

    cmd.arg("--out-dir")
        .arg(&out_dir)
        .arg("--crate-name")
        .arg(crate_name)
        .arg("--crate-type=lib")
        .arg("--edition=2018")
        .arg("-Clto")
        .arg("-Copt-level=3")
        .arg("--target=wasm32-wasi")
        .arg("-L")
        .arg(out_dir);

    for name in externs.iter() {
        cmd.arg("--extern").arg(name);
    }

    dbg!(&cmd);

    cmd.arg(root_file).spawn()?.wait()?.check("rustc")?;

    Ok(out_path)
}

pub fn build_manifest_bin(
    crate_name: &str,
    root_file: &Path,
    externs: &[&str],
    out_dir: &Path,
) -> Result<PathBuf> {
    let out_path = out_dir.join(format!("{}.wasm", crate_name));

    let mut cmd = Command::new("rustc");

    cmd.arg("--out-dir")
        .arg(&out_dir)
        .arg("--edition=2018")
        .arg("--crate-name")
        .arg(crate_name)
        .arg("--crate-type=cdylib")
        .arg("-Clto")
        .arg("-Copt-level=3")
        .arg("--target=wasm32-wasi")
        .arg("-L")
        .arg(out_dir);

    for name in externs.iter() {
        cmd.arg("--extern").arg(name);
    }

    dbg!(&cmd);

    cmd.arg(root_file).spawn()?.wait()?.check("rustc")?;

    Ok(out_path)
}

pub struct Manifest {
    module: Module,
    import_objects: NamedResolverChain<ImportObject, ImportObject>,
}

impl Manifest {
    pub fn new(
        manifest_bin: &Path,
        interner: &Arc<Interner>,
        plugins: &Vec<Box<dyn Plugin>>,
    ) -> Result<Self> {
        let mut wasi = WasiState::new("manifest")
            .preopen(|p| p.directory(".").read(true))?
            .preopen(|p| {
                p.directory("rspkg-result")
                    .read(true)
                    .write(true)
                    .create(true)
            })?
            .finalize()
            .unwrap();
        let store = Store::default();

        let module = Module::from_file(&store, manifest_bin).expect("Read wasm module");
        let mut import_objects = ImportObject::new();

        for plugin in plugins.iter() {
            import_objects
                .register(plugin.name(), plugin.exports(&store, interner))
                .expect("Plugin name is duplicated");
        }

        Ok(Self {
            import_objects: import_objects.chain_front(
                wasi.import_object(&module)
                    .expect("Create wasi import object"),
            ),
            module,
        })
    }

    pub fn build(&self) {
        let instance =
            Instance::new(&self.module, &self.import_objects).expect("Make manifest instance");
        let build_func = instance
            .exports
            .get_native_function::<(), ()>("build")
            .expect("Get build function");
        build_func.call().expect("Call build function")
    }
}
