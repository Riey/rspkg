#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CrateType {
    Bin = 0,
    Lib = 1,
    Cdylib = 2,
    ProcMacro = 3,
}

impl CrateType {
    pub fn from_u32(n: u32) -> Option<Self> {
        match n {
            0 => Some(CrateType::Bin),
            1 => Some(CrateType::Lib),
            2 => Some(CrateType::Cdylib),
            3 => Some(CrateType::ProcMacro),
            _ => None,
        }
    }
}

impl Default for CrateType {
    fn default() -> Self {
        Self::Lib
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Edition {
    Edition2015 = 0,
    Edition2018 = 1,
}

impl Edition {
    pub fn from_u32(n: u32) -> Option<Self> {
        match n {
            0 => Some(Edition::Edition2015),
            1 => Some(Edition::Edition2018),
            _ => None,
        }
    }
}

impl Default for Edition {
    fn default() -> Self {
        Self::Edition2018
    }
}
