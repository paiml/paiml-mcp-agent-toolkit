/// Defect category based on OIP CITL mappings (18 categories)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DefectCategory {
    // Memory & Concurrency
    MemorySafety,
    Concurrency,
    OwnershipBorrow,

    // Type System
    TypeErrors,
    TypeAnnotationGap,
    TraitBounds,
    OperatorPrecedence,

    // Performance & Security
    PerformanceIssues,
    Security,
    Configuration,

    // API & Integration
    ApiMisuse,
    IntegrationFailure,
    StdlibMapping,

    // Code Quality
    DocumentationGap,
    TestingGap,

    // Rust-specific
    ASTTransform,
    ComprehensionBug,
    IteratorChain,
}

impl DefectCategory {
    /// Map rustc error code to defect category
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn from_rustc_error(code: &str) -> Option<Self> {
        match code {
            "E0308" | "E0412" => Some(Self::TypeErrors),
            "E0382" | "E0502" | "E0503" | "E0505" | "E0499" | "E0597" | "E0716" | "E0515" => {
                Some(Self::OwnershipBorrow)
            }
            "E0507" | "E0133" => Some(Self::MemorySafety),
            "E0277" => Some(Self::TraitBounds),
            "E0425" | "E0433" => Some(Self::StdlibMapping),
            "E0599" => Some(Self::ASTTransform),
            "E0615" => Some(Self::OperatorPrecedence),
            "E0658" => Some(Self::Configuration),
            _ => None,
        }
    }

    /// Get confidence score for this category when detected via rustc
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn rustc_confidence(&self) -> f32 {
        match self {
            Self::TypeErrors => 0.95,
            Self::OwnershipBorrow => 0.92,
            Self::MemorySafety => 0.90,
            Self::TraitBounds => 0.95,
            Self::StdlibMapping => 0.85,
            Self::ASTTransform => 0.85,
            Self::OperatorPrecedence => 0.80,
            Self::Configuration => 0.75,
            _ => 0.70,
        }
    }
}
