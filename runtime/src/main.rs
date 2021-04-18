use rspkg_runtime::{
    BuildEnvironment, Dependency, DependencyStore, DependencyType, LocalProject, Profile, Result,
    RspkgProject,
};
use std::path::PathBuf;
use std::{env, sync::Arc};
use wasmer::{Singlepass, Store, JIT};

fn main() -> Result<()> {
    let arg = env::args().nth(1).expect("No argument");

    let engine = JIT::new(Singlepass::new());
    let store = Store::new(&engine.engine());

    let tmp_dir = PathBuf::from("~/.rspkg");
    let manifest_env = BuildEnvironment::new(Profile::Release, tmp_dir.clone(), None, None);

    let deps = Arc::new(DependencyStore::default());

    // bootstrap
    let rspkg_shared = LocalProject::default()
        .build_root_file("./shared/src/lib.rs")
        .build_project_name("rspkg-shared");
    let rspkg_runtime = LocalProject::default()
        .build_root_file("./src/lib.rs")
        .build_project_name("rspkg")
        .build_dependency(Dependency::new("rspkg-shared").build_type(DependencyType::Manifest));
    let manifest = RspkgProject::new("sample".into(), arg.into());

    deps.add_project(rspkg_shared);
    deps.add_project(rspkg_runtime);
    deps.add_project(manifest);

    let manifest = deps.get_project("sample-manifest").unwrap();

    manifest.dependencies(&manifest_env, &store, &deps).unwrap();
    let manifest_out = manifest.build(&manifest_env, &deps, DependencyType::Manifest)?;

    println!("Built out: {}", manifest_out.out.display());

    Ok(())
}
