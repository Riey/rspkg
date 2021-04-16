use crate::{
    BuildArtifacts, BuildEnvironment, CheckResult, CrateType, Edition, Result, RustcFlags,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Serialize, Deserialize, Clone)]
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

    pub fn dependencies(&self, env: &BuildEnvironment) -> Result<Vec<Project>> {
        match self {
            Project::Rspkg(p) => p.dependencies(env),
            Project::Local(p) => p.dependencies(env),
        }
    }

    pub fn build(&self, env: &BuildEnvironment) -> Result<BuildArtifacts> {
        match self {
            Project::Rspkg(p) => p.build(env),
            Project::Local(p) => p.build(env),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
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
        let manifest = LocalProjectBuilder::default()
            .root_file(self.manifest.clone())
            .project_name(self.project_name.clone())
            .crate_type(CrateType::Bin)
            .edition(Edition::Edition2018)
            .dependency("rspkg")
            .build()
            .unwrap();

        manifest.build(env)
    }

    fn run_manifest<T: DeserializeOwned>(
        &self,
        command: &str,
        env: &BuildEnvironment,
    ) -> Result<T> {
        let output = Command::new(self.build_manifest(env)?.out)
            .arg(command)
            .stdout(Stdio::piped())
            .spawn()?
            .wait_with_output()?;

        output.status.check(&self.project_name)?;

        Ok(serde_json::from_slice(&output.stdout)?)
    }

    pub fn build(&self, env: &BuildEnvironment) -> Result<BuildArtifacts> {
        self.run_manifest("build", env)
    }

    pub fn dependencies(&self, env: &BuildEnvironment) -> Result<Vec<Project>> {
        let mut out = Vec::new();
        let deps: Vec<Project> = self.run_manifest("dependencies", env)?;
        for dep in deps {
            let deps = dep.dependencies(env)?;
            out.extend(deps);
            out.push(dep);
        }
        out.dedup_by(|l, r| l.id() == r.id());
        Ok(out)
    }
}

#[derive(Default)]
pub struct LocalProjectBuilder {
    root_file: Option<PathBuf>,
    project_name: String,
    crate_type: CrateType,
    edition: Edition,
    dependencies: Vec<String>,
    features: Vec<String>,
}

impl LocalProjectBuilder {
    pub fn root_file(mut self, root_file: impl Into<PathBuf>) -> Self {
        self.root_file = Some(root_file.into());
        self
    }

    pub fn project_name(mut self, name: impl Into<String>) -> Self {
        self.project_name = name.into();
        self
    }

    pub fn crate_type(mut self, ty: CrateType) -> Self {
        self.crate_type = ty;
        self
    }

    pub fn edition(mut self, edition: Edition) -> Self {
        self.edition = edition;
        self
    }

    pub fn dependency(mut self, dep: impl Into<String>) -> Self {
        self.dependencies.push(dep.into());
        self
    }

    pub fn feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    pub fn build(self) -> Option<LocalProject> {
        Some(LocalProject {
            root_file: self.root_file?,
            crate_type: self.crate_type,
            edition: self.edition,
            features: self.features,
            dependencies: self.dependencies,
            project_name: self.project_name,
        })
    }
}

/// Local project
#[derive(Serialize, Deserialize, Clone)]
pub struct LocalProject {
    /// path for `lib.rs` or `main.rs`
    root_file: PathBuf,
    project_name: String,
    crate_type: CrateType,
    edition: Edition,
    dependencies: Vec<String>,
    features: Vec<String>,
}

impl LocalProject {
    pub fn new(
        root_file: PathBuf,
        crate_name: Option<String>,
        crate_type: CrateType,
        edition: Edition,
        dependencies: Vec<String>,
        features: Vec<String>,
    ) -> Self {
        Self {
            project_name: crate_name
                .unwrap_or_else(|| root_file.file_stem().unwrap().to_string_lossy().to_string()),
            root_file,
            crate_type,
            edition,
            dependencies,
            features,
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
            let dep = env.get_project(dep)?;
            out.extend(dep.dependencies(env)?);
        }
        Ok(out)
    }

    pub fn build(&self, env: &BuildEnvironment) -> Result<BuildArtifacts> {
        let out = match self.crate_type() {
            CrateType::Bin => env.out_dir().join(self.crate_name()),
            CrateType::Lib => env.out_dir().join(format!("lib{}.rlib", self.crate_name())),
            CrateType::ProcMacro => env.out_dir().join(format!("lib{}.so", self.crate_name())),
        };

        if !out.exists() {
            let mut cmd = Command::new("rustc");
            cmd.arg(self.root_file())
                .arg("--crate-name")
                .arg(self.crate_name().replace("-", "_"))
                .arg("-L")
                .arg(env.out_dir())
                .arg("-o")
                .arg(&out)
                .rustc_flags(self.edition())
                .rustc_flags(self.crate_type())
                .rustc_flags(env.profile());

            for dep in self.dependencies.iter() {
                let dep_project = env.get_project(dep)?;
                let dep_out = dep_project.build(env)?;
                cmd.arg("--extern")
                    .arg(format!("{}={}", dep, dep_out.out.display()));
            }

            eprintln!("RUN: {:?}", cmd);

            cmd.spawn()?.wait()?.check("rustc")?;
        }

        Ok(BuildArtifacts { out })
    }
}
