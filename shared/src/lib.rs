#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum PackageType {
    Local = 0,
    Rspkg = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum CrateType {
    Bin = 0,
    Lib = 1,
    Cdylib = 2,
    ProcMacro = 3,
}

impl Default for CrateType {
    fn default() -> Self {
        Self::Lib
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum Edition {
    Edition2015 = 0,
    Edition2018 = 1,
}

impl Default for Edition {
    fn default() -> Self {
        Self::Edition2018
    }
}
