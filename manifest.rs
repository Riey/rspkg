#![no_std]

use rspkg::add_local_dependency;

#[no_mangle]
pub extern "C" fn dependencies() {
    add_local_dependency(
        "helloworld",
        "./hello.rs",
        CrateType::Bin,
        Edition::Edition2018,
    );
}

#[no_mangle]
pub extern "C" fn build() {}
