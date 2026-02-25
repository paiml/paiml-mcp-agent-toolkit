// Tests for context service
// Extracted for file health compliance (CB-040)
// Split into include!() submodules for file size compliance (PMAT-503)

use super::*;

mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    include!("context_tests_basic.rs");
}

mod property_tests {
    use super::*;
    use proptest::prelude::*;
    include!("context_tests_property.rs");
}

mod coverage_tests {
    use super::*;

    mod visitor_tests {
        use super::*;
        include!("context_tests_visitor.rs");
    }

    mod grouping_tests {
        use super::*;
        use std::fs;
        use tempfile::TempDir;
        include!("context_tests_grouping.rs");
    }

    mod serde_tests {
        use super::*;
        include!("context_tests_serde.rs");
    }
}
