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
        pub fn dependency_new(name: *const u8, name_len: usize, ty: DependencyType) -> Dependency;
        pub fn dependency_add_feature(index: Dependency, feature: *const u8, feature_len: usize);
        pub fn dependency_add_flag(index: Dependency, flag: *const u8, flag_len: usize);
        pub fn dependency_add_cfg(index: Dependency, cfg: *const u8, cfg_len: usize);
        pub fn dependency_build(
            index: Dependency,
            root_path: *const u8,
            root_path_len: usize,
            crate_ty: CrateType,
            edition: Edition,
        ) -> Artifact;
    }
}

impl Dependency {
    #[inline(always)]
    pub fn new(name: &str, ty: DependencyType) -> Self {
        unsafe { ffi::dependency_new(name.as_ptr(), name.len(), ty) }
    }

    #[inline(always)]
    pub fn add_cfg(self, cfg: &str) {
        unsafe {
            ffi::dependency_add_cfg(self, cfg.as_ptr(), cfg.len());
        }
    }

    #[inline(always)]
    pub fn add_flag(self, flag: &str) {
        unsafe {
            ffi::dependency_add_flag(self, flag.as_ptr(), flag.len());
        }
    }

    #[inline(always)]
    pub fn add_feature(self, feature: &str) {
        unsafe {
            ffi::dependency_add_feature(self, feature.as_ptr(), feature.len());
        }
    }

    #[inline(always)]
    pub fn build(self, root_path: &str, crate_ty: CrateType, edition: Edition) -> Artifact {
        unsafe {
            ffi::dependency_build(self, root_path.as_ptr(), root_path.len(), crate_ty, edition)
        }
    }
}
