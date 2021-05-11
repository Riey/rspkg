use rspkg_plugin_rustc_shared::{CrateType, Profile, Edition};

#[link(wasm_import_module = "rustc")]
extern "C" {
    pub fn build(root_file: *const u8, root_file_len: usize, crate_type: CrateType, edition: Edition, profile: Profile);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
