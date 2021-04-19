use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::Command;

use rspkg_shared::{CrateType, DependencyType, Edition};

use crate::{CheckResult, Result, RustcFlags};

#[derive(Clone)]
pub struct BuildEnvironment {
    profile: Profile,
    out_dir: PathBuf,
    build_target: Option<String>,
    host_target: Option<String>,
}

impl BuildEnvironment {
    pub fn new(
        profile: Profile,
        out_dir: PathBuf,
        build_target: Option<String>,
        host_target: Option<String>,
    ) -> Self {
        Self {
            profile,
            out_dir,
            build_target,
            host_target,
        }
    }

    pub fn build_target(&self) -> Option<&str> {
        self.build_target.as_deref()
    }

    pub fn host_target(&self) -> Option<&str> {
        self.host_target.as_deref()
    }

    pub fn get_target(&self, dep_ty: DependencyType) -> Option<&str> {
        match dep_ty {
            DependencyType::Normal => self.host_target(),
            DependencyType::Build | DependencyType::Dev => self.build_target(),
            DependencyType::Manifest => Some("wasm32-unknown-unknown"),
        }
    }

    pub fn out_dir(&self) -> &Path {
        &self.out_dir
    }

    pub fn target_out_dir(&self, dep_ty: DependencyType) -> PathBuf {
        if let Some(t) = self.get_target(dep_ty) {
            self.out_dir.join(t)
        } else {
            self.out_dir.clone()
        }
    }

    pub fn profile(&self) -> Profile {
        self.profile
    }
}

pub struct BuildArtifacts {
    pub out: PathBuf,
}

#[derive(Clone)]
pub struct BuildInfo {
    /// path for `lib.rs` or `main.rs`
    root_file: PathBuf,
    project_name: String,
    crate_type: CrateType,
    edition: Edition,
    flags: Vec<String>,
}

impl BuildInfo {
    pub fn new(root_file: impl Into<PathBuf>, project_name: impl Into<String>) -> Self {
        Self {
            root_file: root_file.into(),
            project_name: project_name.into(),
            crate_type: CrateType::default(),
            edition: Edition::default(),
            flags: Vec::default(),
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

    pub fn out_file_name(&self, env: &BuildEnvironment, target: DependencyType) -> String {
        if env.get_target(target).map_or(false, |t| t.contains("wasm")) {
            match self.crate_type() {
                CrateType::Bin | CrateType::Cdylib => {
                    format!("{}.wasm", self.crate_name().replace("-", "_"))
                }
                CrateType::ProcMacro => {
                    format!("lib{}.so", self.crate_name().replace("-", "_"))
                }
                CrateType::Lib => format!("lib{}.rlib", self.crate_name().replace("-", "_")),
            }
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

    pub fn build(&self, env: &BuildEnvironment, target: DependencyType) -> Result<BuildArtifacts> {
        let out_dir = env.target_out_dir(target);

        std::fs::create_dir_all(&out_dir).unwrap();

        let out = out_dir.join(self.out_file_name(env, target));

        eprintln!("{} out: {}", self.crate_name(), out.display());

        if !out.exists() {
            let mut cmd = Command::new("rustc");
            cmd.arg(self.root_file())
                .arg("--crate-name")
                .arg(self.crate_name().replace("-", "_"))
                .arg("-L")
                .arg(&out_dir)
                .arg("-L")
                .arg(&env.out_dir())
                .arg("--out-dir")
                .arg(out_dir)
                .rustc_flags(self.edition())
                .rustc_flags(self.crate_type())
                .rustc_flags(env.profile());

            if let Some(target) = env.get_target(target) {
                cmd.arg("--target").arg(target);
            }

            for flag in self.flags.iter() {
                cmd.arg(flag);
            }

            eprintln!("RUN: {:?}", cmd);

            cmd.spawn()?.wait()?.check("rustc")?;
        }

        Ok(BuildArtifacts { out })
    }

    pub fn build_crate_type(mut self, ty: CrateType) -> Self {
        self.crate_type = ty;
        self
    }

    pub fn build_edition(mut self, edition: Edition) -> Self {
        self.edition = edition;
        self
    }

    pub fn build_flag(mut self, flag: impl Into<String>) -> Self {
        self.flags.push(flag.into());
        self
    }

    pub fn build_feature(self, feature: impl Display) -> Self {
        self.build_flag(format!("--cfg=feature=\"{}\"", feature))
    }

    pub fn build_features<S: Display>(mut self, features: impl Iterator<Item = S>) -> Self {
        for feature in features {
            self = self.build_feature(feature);
        }
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Profile {
    Dev,
    Release,
}

impl Default for Profile {
    fn default() -> Self {
        Self::Dev
    }
}
