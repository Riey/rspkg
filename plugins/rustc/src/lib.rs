use rspkg_runtime_ffi::{spawn_command, HostString};
use std::path::{Path, PathBuf};

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
    flags: Vec<HostString>,
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
        let out_dir = HostString::new(env.out_dir().to_str().unwrap());
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

    pub fn build(&mut self) -> Result<PathBuf, ()> {
        let crate_name = self.crate_name.replace("-", "_");
        let edition = self.edition;
        let crate_type = self.crate_type;

        self.extra_flag("--edition")
            .extra_flag(match edition {
                Edition::E2015 => "2015",
                Edition::E2018 => "2018",
            })
            .extra_flag("--crate-type")
            .extra_flag(match crate_type {
                CrateType::Bin => "bin",
                CrateType::Cdylib => "cdylib",
                CrateType::Lib => "lib",
                CrateType::ProcMacro => "proc-macro",
            })
            .extra_flag("--crate-name")
            .extra_flag(crate_name.as_str());

        if spawn_command("rustc".into(), &self.flags) {
            let out_file_path = match self.crate_type {
                CrateType::Bin => crate_name,
                CrateType::Lib => format!("lib{}.rlib", crate_name),
                CrateType::Cdylib | CrateType::ProcMacro => format!("lib{}.so", crate_name),
            };
            Ok(self.out_dir.join(out_file_path))
        } else {
            Err(())
        }
    }

    pub fn extra_flag(&mut self, flag: impl Into<HostString>) -> &mut Self {
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
