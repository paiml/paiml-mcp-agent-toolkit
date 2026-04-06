// UnifiedContextBuilder - Integrates all advanced annotations into unified context output
// use crate::services::simple_deep_context::SimpleDeepContext;
#![allow(dead_code)]

use crate::services::context::ProjectContext;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

pub struct UnifiedContextBuilder {
    output: String,
    project_path: PathBuf,
    #[allow(dead_code)]
    annotations: HashMap<String, String>,
}

impl UnifiedContextBuilder {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub fn new(project_path: &Path) -> Self {
        debug_assert!(
            project_path.exists(),
            "project_path must exist: {}",
            project_path.display()
        );
        Self {
            output: String::new(),
            project_path: project_path.to_path_buf(),
            annotations: HashMap::new(),
        }
    }
}

impl Display for UnifiedContextBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.output)
    }
}

include!("unified_context_sync_methods.rs");
include!("unified_context_async_methods.rs");
include!("unified_context_analysis.rs");

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_temp_project() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    include!("unified_context_builder_tests.rs");
    include!("unified_context_builder_tests_async.rs");
}
