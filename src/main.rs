use rspkg::{
    BuildEnvironment, CrateType, CratesIoRegistry, Edition, LocalProjectBuilder, Profile, Result,
    RspkgProject,
};
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let arg = env::args().nth(1).expect("No argument");
    let crates_io = CratesIoRegistry::new()?;

    let tmp_dir = PathBuf::from("/tmp/rspkg");
    let mut manifest_env = BuildEnvironment::new(Profile::Release, tmp_dir.clone());

    crates_io.fetch_lib(
        &tmp_dir,
        "lib.rs",
        LocalProjectBuilder::default(),
        "unicode-xid",
        "0.2.1",
        &mut manifest_env,
    )?;
    crates_io.fetch_lib(
        &tmp_dir,
        "lib.rs",
        LocalProjectBuilder::default()
            .dependency("unicode-xid")
            .cfg("use_proc_macro")
            .feature("proc-macro"),
        "proc-macro2",
        "1.0.26",
        &mut manifest_env,
    )?;
    crates_io.fetch_lib(
        &tmp_dir,
        "lib.rs",
        LocalProjectBuilder::default()
            .dependency("proc-macro2")
            .feature("default")
            .feature("proc-macro"),
        "quote",
        "1.0.9",
        &mut manifest_env,
    )?;
    crates_io.fetch_lib(
        &tmp_dir,
        "lib.rs",
        LocalProjectBuilder::default()
            .features(&["derive", "parsing", "printing", "clone-impls", "proc-macro"])
            .dependency("proc-macro2")
            .dependency("unicode-xid")
            .dependency("quote"),
        "syn",
        "1.0.69",
        &mut manifest_env,
    )?;
    crates_io.fetch_lib(
        &tmp_dir,
        "lib.rs",
        LocalProjectBuilder::default()
            .crate_type(CrateType::ProcMacro)
            .dependency("proc-macro2")
            .dependency("syn")
            .dependency("quote")
            .edition(Edition::Edition2015),
        "serde_derive",
        "1.0.125",
        &mut manifest_env,
    )?;
    crates_io.fetch_lib(
        &tmp_dir,
        "lib.rs",
        LocalProjectBuilder::default()
            .dependency("serde_derive")
            .cfg("serde_derive")
            .features(&["default", "std", "alloc", "serde_derive"])
            .edition(Edition::Edition2015),
        "serde",
        "1.0.125",
        &mut manifest_env,
    )?;
    crates_io.fetch_lib(
        &tmp_dir,
        "lib.rs",
        LocalProjectBuilder::default(),
        "itoa",
        "0.4.7",
        &mut manifest_env,
    )?;
    crates_io.fetch_lib(
        &tmp_dir,
        "lib.rs",
        LocalProjectBuilder::default(),
        "ryu",
        "1.0.5",
        &mut manifest_env,
    )?;
    crates_io.fetch_lib(
        &tmp_dir,
        "lib.rs",
        LocalProjectBuilder::default()
            .features(&["default", "std", "alloc"])
            .dependency("itoa")
            .dependency("ryu")
            .dependency("serde"),
        "serde_json",
        "1.0.64",
        &mut manifest_env,
    )?;

    // bootstrap
    let rspkg_runtime = LocalProjectBuilder::default()
        .root_file("./src/lib.rs")
        .project_name("rspkg")
        .dependency("serde")
        .dependency("serde_json")
        .build()
        .unwrap();
    let manifest = RspkgProject::new("sample".into(), arg.into());

    manifest_env.add_project(rspkg_runtime.into());
    manifest_env.add_project(manifest.into());
    manifest_env.prepare_deps()?;

    let manifest_out = manifest_env.build("sample-manifest")?;

    println!("Built out: {}", manifest_out.out.display());

    Ok(())
}
