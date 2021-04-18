use crate::{BuildEnvironment, BuildInfo, DependencyStore, Result};
use rspkg_shared::{CrateType, DependencyType, Edition};
use std::{path::PathBuf, result::Result as StdResult, sync::Arc};
use wasmer::{imports, Array, Function, Store, WasmPtr};
use wasmer::{HostEnvInitError, Instance, LazyInit, Memory, Module, WasmerEnv};

pub struct Project {
    project_name: String,
    store: Store,
    manifest_module: Module,
    manifest_env: ManifestWasmEnv,
}

impl Project {
    pub fn new(
        project_name: impl Into<String>,
        manifest_env: ManifestWasmEnv,
        manifest: PathBuf,
    ) -> Result<Self> {
        let project_name = project_name.into();
        let manifest_name = format!("{}-build-manifest", project_name);
        let store = Store::default();
        let manifest_build = BuildInfo::new(manifest, manifest_name)
            .build_crate_type(CrateType::Cdylib)
            .build_flag(format!(
                "--extern=rspkg={}",
                manifest_env.rspkg_lib_path.display()
            ))
            .build(&manifest_env.env, DependencyType::Manifest)?;

        let wasm = std::fs::read(manifest_build.out).unwrap();
        let manifest_module = wasmer::Module::new(&store, &wasm).unwrap();

        Ok(Self {
            project_name,
            store,
            manifest_module,
            manifest_env,
        })
    }

    pub fn name(&self) -> &str {
        &self.project_name
    }

    // TODO: pass target to manifest's build function
    pub fn build(&self, _target: DependencyType) -> Result<u32> {
        let instance = wasmer::Instance::new(&self.manifest_module, &imports! {
            "env" => {
                "build_file" => Function::new_native_with_env(&self.store, self.manifest_env.clone(), ManifestWasmEnv::build_file),
            }
        }).unwrap();
        let build_func = instance
            .exports
            .get_native_function::<(), u32>("build")
            .unwrap();

        Ok(build_func.call().unwrap())
    }

    pub fn dependencies(&self) -> Result<()> {
        let instance = wasmer::Instance::new(&self.manifest_module, &imports! {
            "env" => {
                "build_file" => Function::new_native_with_env(&self.store, self.manifest_env.clone(), ManifestWasmEnv::build_file),
            }
        }).unwrap();
        let deps_func = instance.exports.get_function("dependencies").unwrap();
        deps_func.call(&[]).unwrap();
        Ok(())
    }
}

#[derive(Clone)]
pub struct ManifestWasmEnv {
    memory: LazyInit<Memory>,
    rspkg_lib_path: Arc<PathBuf>,
    env: Arc<BuildEnvironment>,
    deps: Arc<DependencyStore>,
}

impl ManifestWasmEnv {
    pub fn new(
        env: Arc<BuildEnvironment>,
        deps: Arc<DependencyStore>,
        rspkg_lib_path: Arc<PathBuf>,
    ) -> Self {
        Self {
            memory: LazyInit::default(),
            env,
            deps,
            rspkg_lib_path,
        }
    }

    fn build_file(
        &self,
        name: WasmPtr<u8, Array>,
        name_len: u32,
        path: WasmPtr<u8, Array>,
        path_len: u32,
        crate_ty: u32,
        edition: u32,
    ) -> u32 {
        let name = unsafe {
            name.get_utf8_str(self.memory.get_unchecked(), name_len)
                .unwrap()
        };

        let path = unsafe {
            path.get_utf8_str(self.memory.get_unchecked(), path_len)
                .unwrap()
        };

        let out = BuildInfo::new(path, name)
            .build_edition(Edition::from_u32(edition).unwrap_or_default())
            .build_crate_type(CrateType::from_u32(crate_ty).unwrap_or_default())
            .build(&self.env, DependencyType::Normal)
            .unwrap();

        eprintln!("Build {}", out.out.display());

        self.deps.add_artifact(out)
    }
}

impl WasmerEnv for ManifestWasmEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> StdResult<(), HostEnvInitError> {
        self.memory
            .initialize(instance.exports.get_memory("memory").unwrap().clone());
        Ok(())
    }
}
