use rspkg_plugin_rustc_shared::{CrateType, Edition, Profile};
use rspkg_runtime::{CheckResult, Plugin};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use wasmer::{Array, Function, LazyInit, Memory, WasmPtr, WasmerEnv};

#[derive(Clone, Default, Debug, WasmerEnv)]
struct RustcEnv {
    #[wasmer(export)]
    memory: LazyInit<Memory>,
    out_dir: PathBuf,
}

impl RustcEnv {
    pub fn new(out_dir: PathBuf) -> Self {
        Self {
            memory: LazyInit::default(),
            out_dir,
        }
    }

    fn build_impl(
        &self,
        root_file: &str,
        crate_type: u32,
        edition: u32,
        profile: u32,
    ) -> Option<()> {
        let crate_type = CrateType::from_u32(crate_type)?;
        let edition = Edition::from_u32(edition)?;
        let profile = Profile::from_u32(profile)?;

        Command::new("rustc")
            .arg("--out-dir")
            .arg(&self.out_dir)
            .arg(root_file)
            .spawn()
            .ok()?
            .wait()
            .ok()?
            .check("rustc")
            .ok()?;

        Some(())
    }

    fn build(
        &self,
        root_file: WasmPtr<u8, Array>,
        root_file_len: u32,
        crate_type: u32,
        edition: u32,
        profile: Profile,
    ) -> u32 {
        let root_file = unsafe {
            root_file
                .get_utf8_str(&self.memory, root_file_len)
                .expect("Read root file path")
        };

        self.build_impl(root_file, crate_type, edition, profile)
            .map_or(1, |_| 0)
    }
}

pub struct RustcPlugin {
    pub out_dir: PathBuf,
}

impl Plugin for RustcPlugin {
    fn name(&self) -> &str {
        "rustc"
    }

    fn exports(&self, store: &wasmer::Store) -> wasmer::Exports {
        let mut ret = wasmer::Exports::new();

        ret.insert(
            "build",
            Function::new_native_with_env(store, RustcEnv::default(), RustcEnv::build),
        );

        ret
    }
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
