use rspkg_runtime::{
    BuildEnvironment, BuildInfo, DependencyStore, DependencyType, ManifestWasmEnv, Profile,
    Project, Result,
};
use std::path::PathBuf;
use std::{env, sync::Arc};

fn main() -> Result<()> {
    let arg = env::args().nth(1).expect("No argument");
    let tmp_dir = PathBuf::from("rspkg-result");

    let build_env = Arc::new(BuildEnvironment::new(
        Profile::Release,
        tmp_dir.clone(),
        None,
        None,
    ));
    let rspkg_shared = BuildInfo::new("./shared/src/lib.rs", "rspkg-shared")
        .build(&build_env, DependencyType::Manifest)?;
    let rspkg = BuildInfo::new("./src/lib.rs", "rspkg")
        .build_flag(format!(
            "--extern=rspkg_shared={}",
            rspkg_shared.out.display()
        ))
        .build(&build_env, DependencyType::Manifest)?;
    let deps = Arc::new(DependencyStore::default());
    let manifest_env = ManifestWasmEnv::new(build_env, deps.clone(), Arc::new(rspkg.out));

    let manifest = Project::new("root", manifest_env, arg.into())?;
    manifest.dependencies()?;
    let manifest_out = deps.get_artifact(manifest.build(DependencyType::Normal)?)?;

    println!("Built out: {}", manifest_out.out.display());

    Ok(())
}
