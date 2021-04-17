use crate::{
    BuildArtifacts, BuildEnvironment, CheckResult, CrateType, Edition, Result, RustcFlags,
};
use std::{collections::HashMap, process::Command};
use std::{fmt::Display, path::PathBuf};
use wasmer::{imports, Function, Singlepass, Store, Target, JIT};

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

    pub fn dependencies(&self, env: &BuildEnvironment, deps: &mut HashMap<String, Project>) -> Result<()> {
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
            .build_target("wasm32-wasi")
            .build_edition(Edition::Edition2018)
            .build_dependency(Dependency::new("rspkg"));

        manifest.build(env)
    }

    pub fn build(&self, env: &BuildEnvironment) -> Result<BuildArtifacts> {
        todo!()
    }

    pub fn dependencies(&self, env: &mut BuildEnvironment) -> Result<()> {
        let wasm = std::fs::read(self.build_manifest(env)?.out)?;
        let engine = JIT::new(Singlepass::new());
        let store = Store::new(&engine.engine());
        let module = wasmer::Module::new(&store, &wasm).unwrap();
        let add_dependency = || {
            env.out_dir();
        };
        let import_object = imports! {
            "env" => {
                "add_dependency" => Function::new_native(&store, add_dependency),
            },
        };
        let instance = wasmer::Instance::new(&module, &import_object).unwrap();

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

impl LocalProject {
    pub fn build_root_file(mut self, root_file: impl Into<PathBuf>) -> Self {
        self.root_file = root_file.into();
        self
    }

    pub fn build_project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = name.into();
        self
    }

    pub fn build_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
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

/// Local project
#[derive(Clone, Default)]
pub struct LocalProject {
    /// path for `lib.rs` or `main.rs`
    root_file: PathBuf,
    project_name: String,
    crate_type: CrateType,
    edition: Edition,
    target: Option<String>,
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

    pub fn dependencies(&self, env: &BuildEnvironment) -> Result<Vec<Project>> {
        let mut out = Vec::new();
        for dep in self.dependencies.iter() {
            let dep = env.get_project(&dep.name)?;
            out.extend(dep.dependencies(env)?);
        }
        Ok(out)
    }

    pub fn out_file_name(&self) -> String {
        if self.target.map_or(false, |t| t.contains("wasm")) {
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

            if let Some(target) = self.target.as_ref() {
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
}
