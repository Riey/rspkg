use crate::HostString;

extern "C" {
    pub fn alloc_host_string(text: *const u8, text_len: usize) -> HostString;
    pub fn spawn_command(name: HostString, args: *const HostString, args_len: usize) -> bool;
}
