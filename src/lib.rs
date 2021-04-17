#![no_std]

pub use rspkg_shared::*;

/// rspkg wasm exported functions
pub mod ffi {
    use rspkg_shared::*;

    extern "C" {
        /// Add rspkg depenedency into this manifest
        pub fn add_rspkg_dependency(
            name_ptr: *const u8,
            name_len: usize,
            path_ptr: *const u8,
            path_len: usize,
        );

        pub fn add_local_dependency(
            name_ptr: *const u8,
            name_len: usize,
            path_ptr: *const u8,
            path_len: usize,
            crate_ty: CrateType,
            edition: Edition,
        );
    }
}

pub fn add_rspkg_dependency(name: &str, path: &str) {
    unsafe {
        ffi::add_rspkg_dependency(name.as_ptr(), name.len(), path.as_ptr(), path.len());
    }
}

pub fn add_local_dependency(name: &str, path: &str, crate_ty: CrateType, edition: Edition) {
    unsafe {
        ffi::add_local_dependency(
            name.as_ptr(),
            name.len(),
            path.as_ptr(),
            path.len(),
            crate_ty,
            edition,
        );
    }
}
