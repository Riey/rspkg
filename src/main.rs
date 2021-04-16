use rspkg::{BuildEnvironment, CheckResult, CrateType, Edition, Profile, Project, Result};
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::{env, process::Stdio};

fn main() -> Result<()> {
    let arg = env::args().nth(1).expect("No argument");
    let mut tmp_dir = String::new();
    Command::new("mktemp")
        .stdout(Stdio::piped())
        .arg("-d")
        .spawn()?
        .stdout
        .unwrap()
        .read_to_string(&mut tmp_dir)?;
    let tmp_dir = PathBuf::from(tmp_dir.trim_end());

    // bootstrap
    let rspkg_runtime = Project::new(
        // TODO: replace online source
        "./src/lib.rs".into(),
        Some("rspkg".into()),
        CrateType::Lib,
        Edition::Edition2018,
        vec![],
    );
    let manifest = Project::new(
        arg.into(),
        None,
        CrateType::Bin,
        Edition::Edition2018,
        vec!["rspkg".into()],
    );

    let mut manifest_env = BuildEnvironment::new(Profile::Release, tmp_dir);
    manifest_env.add_project(rspkg_runtime);
    manifest_env.add_project(manifest);
    let manifest_out = manifest_env.build("manifest")?;
    Command::new(manifest_out.out)
        .spawn()?
        .wait()?
        .check("manifest")?;
    Ok(())
}
