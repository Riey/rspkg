use rspkg_plugin_rustc::{BuildEnvironment, BuildFlags, CrateType, Edition, Profile};
use std::path::Path;

#[no_mangle]
pub extern "C" fn build() {
    let build_env = BuildEnvironment::new(Profile::Dev, Path::new("rspkg-result"));

    BuildFlags::new(&build_env)
        .root_file("./hello.rs")
        .crate_type(CrateType::Bin)
        .crate_name("hello")
        .edition(Edition::E2018)
        .build()
        .expect("Build hello");
}
