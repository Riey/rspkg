#![no_std]
use core::panic::PanicInfo;
use rspkg::{build_file, CrateType, Edition};

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn dependencies() {}

#[no_mangle]
pub extern "C" fn build() -> u32 {
    build_file("hello", "./hello.rs", CrateType::Bin, Edition::Edition2018)
}
