use std::num::NonZeroU32;
use std::sync::Arc;

pub type Interner = lasso::ThreadedRodeo<Key>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Key(NonZeroU32);

impl Key {
    pub fn from_u32(n: u32) -> Option<Self> {
        NonZeroU32::new(n).map(Self)
    }
}

unsafe impl lasso::Key for Key {
    #[inline]
    fn into_usize(self) -> usize {
        self.0.get() as usize
    }

    #[inline]
    fn try_from_usize(int: usize) -> Option<Self> {
        use std::convert::TryFrom;
        u32::try_from(int).ok().and_then(NonZeroU32::new).map(Self)
    }
}

unsafe impl wasmer::FromToNativeWasmType for Key {
    type Native = i32;

    fn from_native(native: i32) -> Self {
        match NonZeroU32::new(native as u32) {
            Some(n) => Self(n),
            None => panic!("Key can't be zero"),
        }
    }

    #[inline]
    fn to_native(self) -> i32 {
        self.0.get() as i32
    }
}

pub trait Plugin {
    /// Name of plugin this variable must be **unique** and used when connect wasm import
    fn name(&self) -> &str;
    /// Plugin Exports
    fn exports(&self, store: &wasmer::Store, interner: &Arc<Interner>) -> wasmer::Exports;
}
