use rspkg_runtime::{
    BuildEnvironment, CrateType, CratesIoRegistry, Dependency, Edition, LocalProject, Profile,
    Result, RspkgProject,
};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let arg = env::args().nth(1).expect("No argument");

    let tmp_dir = PathBuf::from("/tmp/rspkg");
    let mut manifest_env = BuildEnvironment::new(Profile::Release, tmp_dir.clone());

    // bootstrap
    let rspkg_shared = LocalProject::default()
        .build_root_file("./shared/src/lib.rs")
        .build_project_name("rspkg-shared");
    let rspkg_runtime = LocalProject::default()
        .build_root_file("./src/lib.rs")
        .build_project_name("rspkg")
        .build_dependency(Dependency::new("rspkg-shared"));
    let manifest = RspkgProject::new("sample".into(), arg.into());

    manifest_env.add_project(rspkg_shared.into());
    manifest_env.add_project(rspkg_runtime.into());
    manifest_env.add_project(manifest.into());
    manifest_env.prepare_deps()?;

    let manifest_out = manifest_env.build("sample-manifest")?;

    println!("Built out: {}", manifest_out.out.display());

    Ok(())
}
