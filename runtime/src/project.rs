use crate::{
    BuildArtifacts, BuildEnvironment, CheckResult, CrateType, Edition, Result, RustcFlags,
};
use std::{collections::HashMap, fmt::Debug, process::Command};
use std::{
    fmt::Display,
    path::PathBuf,
    result::Result as StdResult,
    sync::{Arc, Mutex},
};
use wasmer::{imports, Array, Function, WasmPtr};
use wasmer::{HostEnvInitError, Instance, LazyInit, Memory, WasmerEnv};

#[derive(Clone)]
struct ManifestWasmEnv {
    memory: LazyInit<Memory>,
    deps: Arc<Mutex<HashMap<String, Project>>>,
}

impl ManifestWasmEnv {
    fn add_dep(&self, p: Project) {
        let mut deps = self.deps.lock().unwrap();
        // TODO: merge features
        deps.insert(p.id().into(), p);
    }

    fn add_local_dependency(
        &self,
        name: WasmPtr<u8, Array>,
        name_len: u32,
        path: WasmPtr<u8, Array>,
        path_len: u32,
        crate_type: u32,
        edition: u32,
    ) {
        let name = unsafe {
            name.get_utf8_str(self.memory.get_unchecked(), name_len)
                .unwrap()
        };

        let path = unsafe {
            path.get_utf8_str(self.memory.get_unchecked(), path_len)
                .unwrap()
        };

        self.add_dep(
            LocalProject::new(path.into())
                .build_project_name(name)
                .build_crate_type(CrateType::from_u32(crate_type).unwrap_or_default())
                .build_edition(Edition::from_u32(edition).unwrap_or_default())
                .into(),
        );
    }

    fn add_rspkg_dependency(
        &self,
        name: WasmPtr<u8, Array>,
        name_len: u32,
        path: WasmPtr<u8, Array>,
        path_len: u32,
    ) {
        let name = unsafe {
            name.get_utf8_str(self.memory.get_unchecked(), name_len)
                .unwrap()
        };

        let path = unsafe {
            path.get_utf8_str(self.memory.get_unchecked(), path_len)
                .unwrap()
        };

        self.add_dep(RspkgProject::new(name, path.into()).into());
    }
}

impl WasmerEnv for ManifestWasmEnv {
    fn init_with_instance(&mut self, instance: &Instance) -> StdResult<(), HostEnvInitError> {
        self.memory
            .initialize(instance.exports.get_memory("memory").unwrap().clone());
        Ok(())
    }
}

#[derive(Clone)]
pub enum Project {
    Rspkg(RspkgProject),
    Local(LocalProject),
}

impl From<RspkgProject> for Project {
    fn from(p: RspkgProject) -> Self {
        Self::Rspkg(p)
    }
}
impl From<LocalProject> for Project {
    fn from(p: LocalProject) -> Self {
        Self::Local(p)
    }
}

impl Project {
    pub fn id(&self) -> &str {
        match self {
            Project::Rspkg(p) => &p.project_name,
            Project::Local(p) => &p.project_name,
        }
    }

    pub fn dependencies(
        &self,
        env: &BuildEnvironment,
        deps: &Arc<Mutex<HashMap<String, Project>>>,
    ) -> Result<()> {
        match self {
            Project::Rspkg(p) => p.dependencies(env, deps),
            Project::Local(p) => p.dependencies(env, deps),
        }
    }

    pub fn build(&self, env: &BuildEnvironment) -> Result<BuildArtifacts> {
        match self {
            Project::Rspkg(p) => p.build(env),
            Project::Local(p) => p.build(env),
        }
    }
}

#[derive(Clone)]
pub struct RspkgProject {
    project_name: String,
    manifest: PathBuf,
}

impl RspkgProject {
    pub fn new(project_name: &str, manifest: PathBuf) -> Self {
        Self {
            project_name: format!("{}-manifest", project_name),
            manifest,
        }
    }

    fn build_manifest(&self, env: &BuildEnvironment) -> Result<BuildArtifacts> {
        let manifest = LocalProject::new(self.manifest.clone())
            .build_project_name(self.project_name.clone())
            .build_crate_type(CrateType::Cdylib)
            .build_target("wasm32-unknown-unknown")
            .build_edition(Edition::Edition2018)
            .build_dependency(Dependency::new("rspkg"));

        manifest.build(env)
    }

    pub fn build(&self, env: &BuildEnvironment) -> Result<BuildArtifacts> {
        todo!()
    }

    pub fn dependencies(
        &self,
        env: &BuildEnvironment,
        deps: &Arc<Mutex<HashMap<String, Project>>>,
    ) -> Result<()> {
        let wasm = std::fs::read(self.build_manifest(env)?.out)?;
        let manifest_env = ManifestWasmEnv {
            memory: LazyInit::default(),
            deps: deps.clone(),
        };
        let import_object = imports! {
            "env" => {
                "add_local_dependency" => Function::new_native_with_env(env.store(), manifest_env.clone(), ManifestWasmEnv::add_local_dependency),
                "add_rspkg_dependency" => Function::new_native_with_env(env.store(), manifest_env, ManifestWasmEnv::add_rspkg_dependency),
            },
        };
        let module = wasmer::Module::new(env.store(), &wasm).unwrap();
        let instance = wasmer::Instance::new(&module, &import_object).unwrap();
        instance.exports.get_memory("").unwrap();

        let deps_func = instance.exports.get_function("dependencies").unwrap();
        deps_func.call(&[]).unwrap();
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Dependency {
    pub name: String,
    pub no_default_features: bool,
    pub cfgs: Vec<String>,
}

impl Dependency {
    pub fn new(name: impl Into<String>) -> Dependency {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn build_cfg(mut self, cfg: impl Into<String>) -> Self {
        self.cfgs.push(cfg.into());
        self
    }

    pub fn build_feature(mut self, feature: impl Display) -> Self {
        self.cfgs.push(format!("feature=\"{}\"", feature));
        self
    }

    pub fn build_features(mut self, features: &[&str]) -> Self {
        for feature in features {
            self = self.build_feature(feature);
        }
        self
    }

    pub fn build_no_default_features(mut self, no_default_features: bool) -> Self {
        self.no_default_features = no_default_features;
        self
    }
}

/// Local project
#[derive(Clone, Default)]
pub struct LocalProject {
    /// path for `lib.rs` or `main.rs`
    root_file: PathBuf,
    project_name: String,
    crate_type: CrateType,
    edition: Edition,
    dependencies: Vec<Dependency>,
    cfgs: Vec<String>,
}

impl LocalProject {
    pub fn new(root_file: PathBuf) -> Self {
        Self {
            root_file,
            ..Default::default()
        }
    }

    pub fn root_file(&self) -> &std::path::Path {
        &self.root_file
    }

    pub fn edition(&self) -> Edition {
        self.edition
    }

    pub fn crate_name(&self) -> &str {
        &self.project_name
    }

    pub fn crate_type(&self) -> CrateType {
        self.crate_type
    }

    pub fn dependencies(
        &self,
        env: &BuildEnvironment,
        deps: &Arc<Mutex<HashMap<String, Project>>>,
    ) -> Result<()> {
        for dep in self.dependencies.iter() {
            let dep = env.get_project(&dep.name)?;
            dep.dependencies(env, deps)?;
        }

        Ok(())
    }

    pub fn out_file_name(&self) -> String {
        if self.target.as_ref().map_or(false, |t| t.contains("wasm")) {
            format!("{}.wasm", self.crate_name())
        } else {
            match self.crate_type() {
                CrateType::Bin => self.crate_name().replace("-", "_"),
                CrateType::Lib => format!("lib{}.rlib", self.crate_name().replace("-", "_")),
                CrateType::Cdylib | CrateType::ProcMacro => {
                    format!("lib{}.so", self.crate_name().replace("-", "_"))
                }
            }
        }
    }

    pub fn build(&self, env: &BuildEnvironment) -> Result<BuildArtifacts> {
        let out = env.out_dir().join(self.out_file_name());

        if !out.exists() {
            let mut cmd = Command::new("rustc");
            cmd.arg(self.root_file())
                .arg("--crate-name")
                .arg(self.crate_name().replace("-", "_"))
                .arg("-L")
                .arg(env.out_dir())
                .arg("--out-dir")
                .arg(env.out_dir())
                .rustc_flags(self.edition())
                .rustc_flags(self.crate_type())
                .rustc_flags(env.profile());

            if let Some(target) = env.target() {
                cmd.arg("--target").arg(target);
            }

            for cfg in self.cfgs.iter() {
                cmd.arg("--cfg").arg(cfg);
            }

            for dep in self.dependencies.iter() {
                let dep_project = env.get_project(&dep.name)?;
                let dep_out = dep_project.build(env)?;
                cmd.arg("--extern").arg(format!(
                    "{}={}",
                    dep.name.replace("-", "_"),
                    dep_out.out.display()
                ));
            }

            eprintln!("RUN: {:?}", cmd);

            cmd.spawn()?.wait()?.check("rustc")?;
        }

        Ok(BuildArtifacts { out })
    }

    pub fn build_root_file(mut self, root_file: impl Into<PathBuf>) -> Self {
        self.root_file = root_file.into();
        self
    }

    pub fn set_build_deps(mut self) -> Self {
        self.is_build_dep = true;
        self
    }

    pub fn build_project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = name.into();
        self
    }

    pub fn build_crate_type(mut self, ty: CrateType) -> Self {
        self.crate_type = ty;
        self
    }

    pub fn build_edition(mut self, edition: Edition) -> Self {
        self.edition = edition;
        self
    }

    pub fn build_dependency(mut self, dep: Dependency) -> Self {
        self.dependencies.push(dep);
        self
    }

    pub fn build_cfg(mut self, cfg: impl Into<String>) -> Self {
        self.cfgs.push(cfg.into());
        self
    }

    pub fn build_feature(mut self, feature: impl Display) -> Self {
        self.cfgs.push(format!("feature=\"{}\"", feature));
        self
    }

    pub fn build_features(mut self, features: &[&str]) -> Self {
        for feature in features {
            self = self.build_feature(feature);
        }
        self
    }
}
