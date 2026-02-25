// Tests for TDG alerts
// Extracted to separate file for file health compliance (CB-040)

use super::*;

mod tests {
    use super::*;

    include!("alerts_tests_basic.rs");
}

mod property_tests {
    use proptest::prelude::*;

    include!("alerts_tests_property.rs");
}

mod comprehensive_tests {
    use super::*;

    include!("alerts_tests_comprehensive_part1.rs");
    include!("alerts_tests_comprehensive_part2.rs");
    include!("alerts_tests_comprehensive_part3.rs");
}
