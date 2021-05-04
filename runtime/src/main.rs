use rspkg_runtime::{
    Result,
};
use std::path::PathBuf;
use std::{env, sync::Arc};

fn main() -> Result<()> {
    let arg = env::args().nth(1).expect("No argument");
    let tmp_dir = PathBuf::from("rspkg-result");

    let deps = Arc::new(DependencyStore::default());
    let manifest_env = ManifestWasmEnv::new(build_env, deps.clone(), Arc::new(rspkg.out));

    let manifest = Project::new("root", manifest_env, arg.into())?;
    let manifest_out = deps.get_artifact(manifest.build(DependencyType::Normal)?)?;

    println!("Built out: {}", manifest_out.out.display());

    Ok(())
}
