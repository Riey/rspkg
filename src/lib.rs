#![no_std]

pub use crate::ffi::{Artifact, Dependency};
pub use rspkg_shared::*;

/// rspkg wasm exported functions
pub mod ffi {
    use rspkg_shared::*;

    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct Dependency(u32);

    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct Artifact(u32);

    extern "C" {
        // wasm `trap`
        pub fn unreachable() -> !;
        pub fn dependency_new(name: *const u8, name_len: usize, ty: DependencyType) -> Dependency;
        pub fn dependency_add_feature(index: Dependency, feature: *const u8, feature_len: usize);
        pub fn dependency_add_cfg(index: Dependency, cfg: *const u8, cfg_len: usize);
        pub fn dependency_build(
            index: Dependency,
            root_path: *const u8,
            root_path_len: usize,
            crate_ty: CrateType,
            edition: Edition,
        ) -> Artifact;

        pub fn build_file(
            name: *const u8,
            name_len: usize,
            root_path: *const u8,
            root_path_len: usize,
            crate_ty: CrateType,
            edition: Edition,
        ) -> Artifact;
    }
}

#[macro_export]
macro_rules! nostd_template {
    () => {
        #[panic_handler]
        fn panic(_panic: &::core::panic::PanicInfo<'_>) -> ! {
            unsafe {
                ::core::arch::wasm32::unreachable()   
            }
        }
    };
}

#[inline(always)]
pub fn dependency_new(name: &str, ty: DependencyType) -> Dependency {
    unsafe { ffi::dependency_new(name.as_ptr(), name.len(), ty) }
}

#[inline(always)]
pub fn dependency_add_cfg(index: Dependency, cfg: &str) {
    unsafe {
        ffi::dependency_add_cfg(index, cfg.as_ptr(), cfg.len());
    }
}

#[inline(always)]
pub fn dependency_add_feature(index: Dependency, feature: &str) {
    unsafe {
        ffi::dependency_add_feature(index, feature.as_ptr(), feature.len());
    }
}

#[inline(always)]
pub fn dependency_build(
    index: Dependency,
    root_path: &str,
    crate_ty: CrateType,
    edition: Edition,
) -> Artifact {
    unsafe {
        ffi::dependency_build(
            index,
            root_path.as_ptr(),
            root_path.len(),
            crate_ty,
            edition,
        )
    }
}

#[inline(always)]
pub fn build_file(name: &str, root_path: &str, crate_ty: CrateType, edition: Edition) -> Artifact {
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
