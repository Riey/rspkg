use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Key(NonZeroU32);

impl<'a> From<&'a str> for Key {
    fn from(s: &'a str) -> Key {
        unsafe { ffi::alloc_arg(s.as_ptr(), s.len()) }
    }
}

impl From<String> for Key {
    fn from(s: String) -> Key {
        unsafe { ffi::alloc_arg(s.as_ptr(), s.len()) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CrateType {
    Bin,
    Lib,
    Cdylib,
    ProcMacro,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Edition {
    E2015,
    E2018,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Profile {
    Dev,
    Release,
}

#[derive(Clone)]
pub struct BuildEnvironment {
    profile: Profile,
    out_dir: PathBuf,
}

impl BuildEnvironment {
    pub fn new(profile: Profile, target_dir: &Path) -> Self {
        Self {
            profile,
            out_dir: target_dir.join(match profile {
                Profile::Dev => "debug",
                Profile::Release => "release",
            }),
        }
    }

    pub fn out_dir(&self) -> &Path {
        &self.out_dir
    }

    pub fn profile(&self) -> Profile {
        self.profile
    }
}

#[derive(Clone)]
pub struct BuildFlags {
    root_file: PathBuf,
    crate_name: String,
    crate_type: CrateType,
    edition: Edition,
    out_dir: PathBuf,
    flags: Vec<Key>,
}

impl BuildFlags {
    pub fn new(env: &BuildEnvironment) -> Self {
        let mut flags = Vec::new();

        match env.profile() {
            Profile::Dev => {
                flags.push("-Cdebuginfo=2".into());
            }
            Profile::Release => {
                flags.push("-Copt-level=3".into());
                flags.push("-Clto".into());
            }
        }

        flags.push("-L".into());
        let out_dir = Key::from(env.out_dir().to_str().unwrap());
        flags.push(out_dir);
        flags.push("--out-dir".into());
        flags.push(out_dir);

        Self {
            crate_type: CrateType::Lib,
            edition: Edition::E2018,
            crate_name: String::new(),
            root_file: PathBuf::new(),
            out_dir: env.out_dir().to_path_buf(),
            flags,
        }
    }

    pub fn root_file(&mut self, path: impl Into<PathBuf>) -> &mut Self {
        self.root_file = path.into();
        self
    }

    pub fn build(&mut self) -> PathBuf {
        let crate_name = self.crate_name.replace("-", "_");

        self.flags.push("--edition".into());
        self.flags.push(
            match self.edition {
                Edition::E2015 => "2015",
                Edition::E2018 => "2018",
            }
            .into(),
        );
        self.flags.push("--crate-type".into());
        self.flags.push(
            match self.crate_type {
                CrateType::Bin => "bin",
                CrateType::Cdylib => "cdylib",
                CrateType::Lib => "lib",
                CrateType::ProcMacro => "proc-macro",
            }
            .into(),
        );
        self.flags.push("--crate-name".into());
        self.flags.push(crate_name.as_str().into());
        self.flags.push(self.root_file.to_str().unwrap().into());

        let ret = unsafe { ffi::build(self.flags.as_ptr(), self.flags.len()) };

        if ret != 0 {
            panic!("build exit code: {}", ret);
        }

        let out_file_path = match self.crate_type {
            CrateType::Bin => crate_name,
            CrateType::Lib => format!("lib{}.rlib", crate_name),
            CrateType::Cdylib | CrateType::ProcMacro => format!("lib{}.so", crate_name),
        };
        self.out_dir.join(out_file_path)
    }

    pub fn extra_flag(&mut self, flag: impl Into<Key>) -> &mut Self {
        self.flags.push(flag.into());
        self
    }

    pub fn edition(&mut self, edition: Edition) -> &mut Self {
        self.edition = edition;
        self
    }

    pub fn crate_type(&mut self, ty: CrateType) -> &mut Self {
        self.crate_type = ty;
        self
    }

    pub fn crate_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.crate_name = name.into();
        self
    }
}

mod ffi {
    use crate::Key;

    #[link(wasm_import_module = "rustc")]
    extern "C" {
        pub fn build(args: *const Key, args_len: usize) -> i32;

        pub fn alloc_arg(arg: *const u8, arg_len: usize) -> Key;
    }
}
