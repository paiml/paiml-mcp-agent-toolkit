use std::collections::HashMap;

/// Entropy calculator.
pub struct EntropyCalculator;

impl Default for EntropyCalculator {
    fn default() -> Self {
        Self::new()
    }
}

include!("entropy_calculation.rs");
include!("entropy_tests.rs");
