use crate::{CrateType, Edition};
use std::path::PathBuf;

/// Local project
pub struct Project {
    root_file: PathBuf,
    crate_name: String,
    crate_type: CrateType,
    edition: Edition,
    dependencies: Vec<String>,
}

impl Project {
    pub fn new(
        root_file: PathBuf,
        crate_name: Option<String>,
        crate_type: CrateType,
        edition: Edition,
        dependencies: Vec<String>,
    ) -> Self {
        Self {
            crate_name: crate_name
                .unwrap_or_else(|| root_file.file_stem().unwrap().to_string_lossy().to_string()),
            root_file,
            crate_type,
            edition,
            dependencies,
        }
    }

    pub fn root_file(&self) -> &std::path::Path {
        &self.root_file
    }

    pub fn edition(&self) -> Edition {
        self.edition
    }

    pub fn crate_name(&self) -> &str {
        &self.crate_name
    }

    pub fn crate_type(&self) -> CrateType {
        self.crate_type
    }

    pub fn dependencies(&self) -> &[String] {
        &self.dependencies
    }
}
