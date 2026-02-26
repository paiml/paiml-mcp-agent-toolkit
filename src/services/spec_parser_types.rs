/// Parsed specification with validation criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSpec {
    /// Specification file path
    pub path: PathBuf,

    /// Title from YAML frontmatter or first H1
    pub title: String,

    /// Issue/ticket references (e.g., "#118", "GH-102")
    pub issue_refs: Vec<String>,

    /// Status from frontmatter
    pub status: Option<String>,

    /// Extracted validation claims
    pub claims: Vec<ValidationClaim>,

    /// Code examples that can be validated
    pub code_examples: Vec<CodeExample>,

    /// Acceptance criteria (checkbox items)
    pub acceptance_criteria: Vec<AcceptanceCriterion>,

    /// Test requirements mentioned
    pub test_requirements: Vec<TestRequirement>,

    /// Raw content of the specification (for citation counting, etc.)
    #[serde(skip)]
    pub raw_content: String,
}

/// A falsifiable claim extracted from the specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationClaim {
    /// Unique ID for the claim (e.g., "A1", "B2")
    pub id: String,

    /// The claim text
    pub text: String,

    /// Source location (line number)
    pub line: usize,

    /// Claim category
    pub category: ClaimCategory,

    /// Whether this claim can be automatically validated
    pub automatable: bool,

    /// Validation command (if automatable)
    pub validation_cmd: Option<String>,

    /// Expected result pattern
    pub expected_pattern: Option<String>,
}

/// Claim categories for the 100-point Popperian framework
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClaimCategory {
    /// A. Falsifiability (25 pts) - GATEWAY
    Falsifiability,
    /// B. Implementation (25 pts)
    Implementation,
    /// C. Testing (20 pts)
    Testing,
    /// D. Documentation (15 pts)
    Documentation,
    /// E. Integration (15 pts)
    Integration,
}

impl ClaimCategory {
    pub fn max_points(&self) -> u32 {
        match self {
            Self::Falsifiability => 25,
            Self::Implementation => 25,
            Self::Testing => 20,
            Self::Documentation => 15,
            Self::Integration => 15,
        }
    }

    pub fn from_section(section: &str) -> Option<Self> {
        let lower = section.to_lowercase();
        if lower.contains("falsif") || lower.contains("testab") || lower.contains("claim") {
            Some(Self::Falsifiability)
        } else if lower.contains("implement")
            || lower.contains("code")
            || lower.contains("architecture")
        {
            Some(Self::Implementation)
        } else if lower.contains("test") || lower.contains("coverage") || lower.contains("mutation")
        {
            Some(Self::Testing)
        } else if lower.contains("doc") || lower.contains("readme") || lower.contains("changelog") {
            Some(Self::Documentation)
        } else if lower.contains("integrat") || lower.contains("ci") || lower.contains("deploy") {
            Some(Self::Integration)
        } else {
            None
        }
    }
}

/// Code example from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    /// Language (rust, bash, etc.)
    pub language: String,

    /// Code content
    pub code: String,

    /// Line number in source
    pub line: usize,

    /// Whether this is executable
    pub executable: bool,
}

/// Acceptance criterion with completion status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    /// Criterion text
    pub text: String,

    /// Whether marked as complete (checked)
    pub complete: bool,

    /// Line number in source
    pub line: usize,
}

/// Test requirement extracted from specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequirement {
    /// Requirement text
    pub text: String,

    /// Type of test (unit, integration, property, e2e)
    pub test_type: String,

    /// Related code path if mentioned
    pub code_path: Option<String>,
}
