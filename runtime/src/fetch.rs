use reqwest::blocking::{Client, ClientBuilder};
use std::path::Path;
use std::process::{Command, Stdio};
use std::{io::Write, path::PathBuf};

use crate::{BuildEnvironment, CheckResult, LocalProject, Result};

pub struct CratesIoRegistry {
    client: Client,
}

impl CratesIoRegistry {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: ClientBuilder::new()
                .user_agent("rspkg (creeper844@gmail.com)")
                .build()?,
        })
    }

    pub fn fetch_lib(
        &self,
        out_dir: &Path,
        root: &str,
        project: LocalProject,
        version: &str,
        env: &mut BuildEnvironment,
    ) -> Result<()> {
        let path = self.fetch(out_dir, project.crate_name(), version)?;

        env.add_project(
            project.build_root_file(path.join("src").join(root))
                .into()
        );

        Ok(())
    }

    pub fn latest_version(&self, crate_name: &str) -> Result<String> {
        let mut versions: Versions = self
            .client
            .get(format!(
                "https://crates.io/api/v1/crates/{}/versions",
                crate_name
            ))
            .send()?
            .json()?;
        Ok(std::mem::take(&mut versions.versions[0].num))
    }

    pub fn fetch(&self, out_dir: &Path, crate_name: &str, version: &str) -> Result<PathBuf> {
        let src_dir = out_dir.join(format!("{}-{}", crate_name, version));

        if !src_dir.exists() {
            let download = self
                .client
                .get(format!(
                    "https://crates.io/api/v1/crates/{}/{}/download",
                    crate_name, version,
                ))
                .send()?
                .bytes()?;
            let mut tar = Command::new("tar")
                .arg("xvzf")
                .arg("-")
                .current_dir(out_dir)
                .stdin(Stdio::piped())
                .spawn()?;
            tar.stdin.as_mut().unwrap().write_all(&download)?;
            tar.wait()?.check("tar")?;
        }

        Ok(src_dir)
    }
}

#[derive(serde::Deserialize)]
struct Versions {
    versions: Vec<Version>,
}

#[derive(serde::Deserialize)]
struct Version {
    num: String,
}
