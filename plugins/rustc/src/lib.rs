use rspkg_plugin::{Interner, Key, Plugin};
use std::{path::PathBuf, process::Command, sync::Arc};
use wasmer::{Array, Function, LazyInit, Memory, WasmPtr, WasmerEnv};

#[derive(Clone, Debug, WasmerEnv)]
struct RustcEnv {
    #[wasmer(export)]
    memory: LazyInit<Memory>,
    interner: Arc<Interner>,
    out_dir: PathBuf,
}

impl RustcEnv {
    pub fn new(interner: &Arc<Interner>, out_dir: PathBuf) -> Self {
        Self {
            memory: LazyInit::default(),
            interner: interner.clone(),
            out_dir,
        }
    }

    fn alloc_arg(&self, arg: WasmPtr<u8, Array>, arg_len: u32) -> Key {
        let arg = unsafe { arg.get_utf8_str(&self.memory.get_ref().unwrap(), arg_len) }
            .expect("Get arg string");

        self.interner.get_or_intern(arg)
    }

    fn build(&self, args: WasmPtr<u32, Array>, args_len: u32) -> i32 {
        let mut cmd = Command::new("rustc");

        let args = args
            .deref(&self.memory.get_ref().unwrap(), 0, args_len)
            .expect("Deref args");

        for arg in args {
            cmd.arg(self.interner.resolve(&Key::from_u32(arg.get()).unwrap()));
        }

        dbg!(&cmd);

        cmd.spawn()
            .expect("Spawn rustc")
            .wait()
            .expect("Wait rustc")
            .code()
            .unwrap_or(0)
    }
}

pub struct RustcPlugin {
    pub out_dir: PathBuf,
}

impl Plugin for RustcPlugin {
    fn name(&self) -> &str {
        "rustc"
    }

    fn exports(&self, store: &wasmer::Store, interner: &Arc<Interner>) -> wasmer::Exports {
        let mut ret = wasmer::Exports::new();

        ret.insert(
            "build",
            Function::new_native_with_env(
                store,
                RustcEnv::new(interner, self.out_dir.clone()),
                RustcEnv::build,
            ),
        );

        ret.insert(
            "alloc_arg",
            Function::new_native_with_env(
                store,
                RustcEnv::new(interner, self.out_dir.clone()),
                RustcEnv::alloc_arg,
            ),
        );

        ret
    }
}

