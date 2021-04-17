pub use rspkg_shared::*;

/// rspkg wasm exported functions
pub mod ffi {
    use rspkg_shared::*;

    extern "C" {
        /// Add depenedency into this manifest
        pub fn add_dependency(ty: PackageType, path_ptr: *const u8, path_len: usize);
    }
}

pub fn add_dependency(ty: PackageType, path: &str) {
    unsafe {
        ffi::add_dependency(ty, path.as_ptr(), path.len());
    }
}
