use rspkg::add_dependency;

#[no_mangle]
pub extern "C" fn dependencies() {
    add_dependency(PackageType::Local, "./src/lib.rs");
}

