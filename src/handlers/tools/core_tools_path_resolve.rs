// R22-2 / D102: Glob-aware `project_path` resolution.
//
// The shared implementation lives in `crate::services::path_glob` so both
// the live `handlers/tools` dispatcher and the parallel `mcp_pmcp` tree call
// one helper (no copy-paste). This thin re-export gives the `core_tools.rs`
// include graph the short names its callers use.
use crate::services::path_glob::{resolve_project_path_with_globs, ResolvedProjectPath};
