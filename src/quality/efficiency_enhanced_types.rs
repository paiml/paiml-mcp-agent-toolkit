/// Helper function to convert a syn::Path to a string
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Complexity.
pub enum Complexity {
    O1,         // O(1) - Constant
    OLogN,      // O(log n) - Logarithmic
    ON,         // O(n) - Linear
    ONLogN,     // O(n log n) - Linearithmic
    ON2,        // O(n²) - Quadratic
    ON3,        // O(n³) - Cubic
    OExp,       // O(2^n) - Exponential
    OFactorial, // O(n!) - Factorial
}

impl Display for Complexity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Complexity::O1 => write!(f, "O(1)"),
            Complexity::OLogN => write!(f, "O(log n)"),
            Complexity::ON => write!(f, "O(n)"),
            Complexity::ONLogN => write!(f, "O(n log n)"),
            Complexity::ON2 => write!(f, "O(n^2)"),
            Complexity::ON3 => write!(f, "O(n^3)"),
            Complexity::OExp => write!(f, "O(2^n)"),
            Complexity::OFactorial => write!(f, "O(n!)"),
        }
    }
}

impl Complexity {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Combine.
    pub fn combine(&self, other: &Complexity) -> Complexity {
        // When combining complexities (e.g., nested loops), multiply
        use Complexity::*;
        match (self, other) {
            (O1, x) | (x, O1) => x.clone(),
            (OLogN, OLogN) => ON, // log n * log n ≈ O(n) for practical purposes
            (OLogN, ON) | (ON, OLogN) => ONLogN,
            (ON, ON) => ON2,
            (ON, ON2) | (ON2, ON) => ON3,
            (ON2, ON2) => ON3, // Simplified - could be O(n^4)
            _ => OExp,         // Conservative estimate for complex combinations
        }
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Find the maximum value.
    pub fn max(&self, other: &Complexity) -> Complexity {
        if self > other {
            self.clone()
        } else {
            other.clone()
        }
    }
}

#[derive(Debug, Clone)]
/// Algorithm pattern.
pub enum AlgorithmPattern {
    Sorting,
    Search,
    Graph,
    DynamicProgramming,
    Greedy,
    DivideAndConquer,
    Backtracking,
}
