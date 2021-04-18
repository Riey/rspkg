#![no_std]

pub use rspkg_shared::*;

/// rspkg wasm exported functions
pub mod ffi {
    use rspkg_shared::*;

    extern "C" {
        // pub fn load_dependency(path_ptr: *const u8, path_len: usize) -> usize;

        pub fn build_file(
            name: *const u8,
            name_len: usize,
            root_path: *const u8,
            root_path_len: usize,
            crate_ty: CrateType,
            edition: Edition,
        ) -> u32;
    }
}

pub fn build_file(name: &str, root_path: &str, crate_ty: CrateType, edition: Edition) -> u32 {
    unsafe {
        ffi::build_file(
            name.as_ptr(),
            name.len(),
            root_path.as_ptr(),
            root_path.len(),
            crate_ty,
            edition,
        )
    }
}
