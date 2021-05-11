macro_rules! define_enum {
	($name:ident {
        $($var:ident = $value:expr,)+
    }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        #[repr(u32)]
		pub enum $name {
            $(
                $var = $value,
            )+
        }

        impl $name {
            pub fn from_u32(n: u32) -> Option<Self> {
                match n {
                    $(
                        $value => Some(Self::$var),
                    )+
                    _ => None,
                }
            }
        }
	};
}

define_enum! {
    CrateType {
        Bin = 0,
        Lib = 1,
        Cdylib = 2,
        ProcMacro = 3,
    }
}

impl Default for CrateType {
    fn default() -> Self {
        Self::Lib
    }
}

define_enum! {
    Edition {
        E2015 = 0,
        E2018 = 1,
    }
}

impl Default for Edition {
    fn default() -> Self {
        Self::E2018
    }
}

define_enum! {
    Profile {
        Dev = 0,
        Release = 1,
    }
}
