mod ffi;

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct HostString {
    _handle: u32,
}

impl From<String> for HostString {
    fn from(s: String) -> Self {
        Self::new(s.as_str())
    }
}

impl<'a> From<&'a str> for HostString {
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl HostString {
    pub fn new(s: &str) -> Self {
        unsafe { ffi::alloc_host_string(s.as_ptr(), s.len()) }
    }
}

pub fn spawn_command(name: HostString, args: &[HostString]) -> bool {
    unsafe { ffi::spawn_command(name, args.as_ptr(), args.len()) }
}
