use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{CheckResult, Result};
use wasm_env::ManifestWasmEnv;
use wasmer::{
    imports, Array, ChainableNamedResolver, Function, ImportObject, Instance, LazyInit, Memory,
    Module, Store, WasmPtr, WasmerEnv,
};
use wasmer_wasi::WasiState;

fn build_manifest_lib(crate_name: &str)

pub struct Manifest {}

impl Manifest {
    pub fn new(manifest_file: &Path, out_dir: PathBuf) -> Result<Self> {
        let wasi = WasiState::new("manifest").finalize().unwrap();
        let store = Store::default();
        Command::new("rustc")
            .arg("--out-dir")
            .arg(&out_dir)
            .arg("--crate-name=manifest")
            .arg("-Clto")
            .arg("-Copt-level=3")
            .arg("--target=wasm32-wasi")
            .spawn()?
            .wait()?
            .check("rustc")?;

        let module =
            Module::from_file(&store, out_dir.join("manifest.wasm")).expect("Read wasm module");
        let manifest_env = ManifestWasmEnv::default();
        let import_objects = imports! {
            "env" => {
                "alloc_host_string" => Function::new_native_with_env(&store, manifest_env.clone(), ManifestWasmEnv::alloc_host_string),
                "spawn_command" => Function::new_native_with_env(&store, manifest_env, ManifestWasmEnv::spawn_command),
            }
        };

        Ok(Self {})
    }
}

mod wasm_env {
    use std::sync::atomic::{AtomicU32, Ordering::SeqCst};
    use std::sync::Arc;

    use dashmap::DashMap;
    use wasmer::{Array, LazyInit, Memory, WasmPtr, WasmerEnv};

    use crate::CheckResult;

    #[derive(Clone, Default, WasmerEnv)]
    pub struct ManifestWasmEnv {
        #[wasmer(export)]
        memory: LazyInit<Memory>,
        next: Arc<AtomicU32>,
        string_store: Arc<DashMap<u32, String>>,
    }

    impl ManifestWasmEnv {
        pub fn alloc_host_string(&self, text: WasmPtr<u8, Array>, text_len: u32) -> u32 {
            let text = unsafe {
                text.get_utf8_str(self.memory.get_unchecked(), text_len)
                    .unwrap()
            };

            let next = self.next.fetch_add(1, SeqCst);

            self.string_store.insert(next, text.into());
            next
        }

        fn spawn_command_impl(&self, name: u32, args: &[std::cell::Cell<u32>]) -> Option<()> {
            let mut command = std::process::Command::new(self.string_store.get(&name)?.as_str());

            for arg in args {
                command.arg(self.string_store.get(&arg.get())?.as_str());
            }

            command.spawn().ok()?.wait().ok()?.check("").ok()?;

            Some(())
        }

        pub fn spawn_command(&self, name: u32, args: WasmPtr<u32, Array>, args_len: u32) -> u32 {
            let args = args.deref(self.memory_ref().unwrap(), 0, args_len).unwrap();
            self.spawn_command_impl(name, args).is_some() as u32
        }
    }
}
