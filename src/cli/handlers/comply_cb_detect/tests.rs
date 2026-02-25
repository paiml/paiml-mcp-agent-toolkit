#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::fs;

// =============================================================================
// Tests for CB-130 Agent Context Adoption
// =============================================================================

mod cb130_tests {
    use super::*;
    use tempfile::TempDir;

    include!("tests_cb130_agent_context.rs");
}

// =============================================================================
// Tests for OIP Tarantula Pattern Detection
// =============================================================================

#[cfg(test)]
mod oip_tarantula_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    include!("tests_oip_tarantula.rs");
}

// =============================================================================
// Tests for CB-081 Dependency Count Detection
// =============================================================================

#[cfg(test)]
mod cb081_dependency_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    include!("tests_cb081_dependencies.rs");
}

// =============================================================================
// Tests for CB-600 Lua Best Practices Detection (PMAT-487)
// =============================================================================

#[cfg(test)]
mod cb600_lua_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    include!("tests_cb600_lua_part1.rs");
    include!("tests_cb600_lua_part2.rs");
}

// =============================================================================
// Tests for CB-700 SQL Best Practices
// =============================================================================

mod cb700_sql_tests {
    use super::*;
    use tempfile::TempDir;

    include!("tests_cb700_sql.rs");
}

// =============================================================================
// Tests for CB-900 Markdown Best Practices
// =============================================================================

mod cb900_markdown_tests {
    use super::*;
    use tempfile::TempDir;

    include!("tests_cb900_markdown.rs");
}

// =============================================================================
// Tests for CB-950 YAML Best Practices
// =============================================================================

mod cb950_yaml_tests {
    use super::*;
    use tempfile::TempDir;

    include!("tests_cb950_yaml.rs");
}
