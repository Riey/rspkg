#![no_std]
use core::panic::PanicInfo;
use rspkg::{
    add_local_dependency,
    CrateType,
    Edition,
};

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

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
