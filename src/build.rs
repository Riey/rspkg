use crate::{CheckResult, CrateType, Error, Profile, Project, Result, RustcFlags};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

pub struct BuildEnvironment {
    profile: Profile,
    projects: HashMap<String, Project>,
    out_dir: PathBuf,
}

impl BuildEnvironment {
    pub fn new(profile: Profile, out_dir: PathBuf) -> Self {
        Self {
            profile,
            out_dir,
            projects: HashMap::new(),
        }
    }

    pub fn add_project(&mut self, project: Project) {
        self.projects.insert(project.crate_name().into(), project);
    }

    pub fn build(&self, crate_name: &str) -> Result<BuildArtifacts> {
        let project = self
            .projects
            .get(crate_name)
            .ok_or_else(|| Error::CrateNotFound(crate_name.into()))?;

        let out = match project.crate_type() {
            CrateType::Bin => self.out_dir.join(project.crate_name()),
            CrateType::Lib => self
                .out_dir
                .join(format!("lib{}.rlib", project.crate_name())),
        };

        if !out.exists() {
            let mut cmd = Command::new("rustc");
            cmd.arg(project.root_file())
                .arg("--crate-name")
                .arg(project.crate_name())
                .arg("-L")
                .arg(&self.out_dir)
                .arg("-o")
                .arg(&out)
                .rustc_flags(project.edition())
                .rustc_flags(project.crate_type())
                .rustc_flags(self.profile);

            for dep in project.dependencies() {
                let dep_out = self.build(dep)?;
                cmd.arg("--extern")
                    .arg(format!("{}={}", dep, dep_out.out.display()));
            }

            eprintln!("RUN: {:?}", cmd);

            cmd.spawn()?.wait()?.check("rustc")?;
        }

        Ok(BuildArtifacts { out })
    }
}

pub struct BuildArtifacts {
    pub out: PathBuf,
}
