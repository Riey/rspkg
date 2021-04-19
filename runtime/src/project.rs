use crate::{denpendency::DependencyInfo, BuildEnvironment, BuildInfo, DependencyStore, Result};
use rspkg_shared::{CrateType, DependencyType, Edition};
use std::{path::PathBuf, result::Result as StdResult, sync::Arc};
use wasmer::{imports, Array, Function, ImportObject, Store, WasmPtr};
use wasmer::{HostEnvInitError, Instance, LazyInit, Memory, Module, WasmerEnv};

pub struct Project {
    project_name: String,
    import_objects: ImportObject,
    manifest_module: Module,
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
        let import_objects = imports! {
            "env" => {
                "build_file" => Function::new_native_with_env(&store, manifest_env.clone(), ManifestWasmEnv::build_file),
                "dependency_new" => Function::new_native_with_env(&store, manifest_env.clone(), ManifestWasmEnv::dependency_new),
                "dependency_add_feature" => Function::new_native_with_env(&store, manifest_env.clone(), ManifestWasmEnv::dependency_add_feature),
                "dependency_add_cfg" => Function::new_native_with_env(&store, manifest_env.clone(), ManifestWasmEnv::dependency_add_cfg),
            }
        };

        Ok(Self {
            project_name,
            manifest_module,
            import_objects,
        })
    }

    pub fn name(&self) -> &str {
        &self.project_name
    }

    // TODO: pass target to manifest's build function
    pub fn build(&self, _target: DependencyType) -> Result<u32> {
        let instance = wasmer::Instance::new(&self.manifest_module, &self.import_objects).unwrap();
        let build_func = instance
            .exports
            .get_native_function::<(), u32>("build")
            .unwrap();

        Ok(build_func.call().unwrap())
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

    pub fn dependency_new(&self, name: WasmPtr<u8, Array>, name_len: u32, ty: u32) -> u32 {
        let name = unsafe {
            name.get_utf8_str(self.memory.get_unchecked(), name_len)
                .unwrap()
        };

        self.deps.add_dependency(DependencyInfo {
            name: name.into(),
            ty: DependencyType::from_u32(ty).unwrap_or_default(),
            ..Default::default()
        })
    }

    pub fn dependency_add_cfg(&self, index: u32, cfg: WasmPtr<u8, Array>, cfg_len: u32) {
        let cfg = unsafe {
            cfg.get_utf8_str(self.memory.get_unchecked(), cfg_len)
                .unwrap()
        };

        self.deps.add_cfg(index, cfg).unwrap();
    }

    pub fn dependency_add_feature(
        &self,
        index: u32,
        feature: WasmPtr<u8, Array>,
        feature_len: u32,
    ) {
        let feature = unsafe {
            feature
                .get_utf8_str(self.memory.get_unchecked(), feature_len)
                .unwrap()
        };

        self.deps.add_feature(index, feature).unwrap();
    }

    pub fn dependency_build(
        &self,
        index: u32,
        path: WasmPtr<u8, Array>,
        path_len: u32,
        crate_ty: u32,
        edition: u32,
    ) -> u32 {
        let dep = self.deps.get_dependency(index).unwrap();

        let path = unsafe {
            path.get_utf8_str(self.memory.get_unchecked(), path_len)
                .unwrap()
        };

        let out = BuildInfo::new(path, &dep.name)
            .build_flags(dep.cfgs.iter().map(|c| format!("--cfg={}", c)))
            .build_edition(Edition::from_u32(edition).unwrap_or_default())
            .build_crate_type(CrateType::from_u32(crate_ty).unwrap_or_default())
            .build(&self.env, DependencyType::Normal)
            .unwrap();

        self.deps.add_artifact(out)
    }

    pub fn build_file(
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
