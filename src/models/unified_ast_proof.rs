/// Proof annotation system for formal verification metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProofAnnotation {
    #[serde(rename = "annotationId")]
    pub annotation_id: Uuid,

    #[serde(rename = "propertyProven")]
    pub property_proven: PropertyType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub specification_id: Option<String>,

    pub method: VerificationMethod,

    #[serde(rename = "toolName")]
    pub tool_name: String,

    #[serde(rename = "toolVersion")]
    pub tool_version: String,

    #[serde(rename = "confidenceLevel")]
    pub confidence_level: ConfidenceLevel,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub assumptions: Vec<String>,

    #[serde(rename = "evidenceType")]
    pub evidence_type: EvidenceType,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_location: Option<String>,

    #[serde(rename = "dateVerified")]
    pub date_verified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
/// Type classification for property.
pub enum PropertyType {
    MemorySafety,
    ThreadSafety,
    DataRaceFreeze,
    Termination,
    FunctionalCorrectness(String), // spec_id
    ResourceBounds {
        cpu: Option<u64>,
        memory: Option<u64>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
/// Level classification for confidence.
pub enum ConfidenceLevel {
    Low = 1,    // Heuristic-based (e.g., pattern matching)
    Medium = 2, // Sound static analysis with assumptions
    High = 3,   // Machine-checkable proof
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Verification method.
pub enum VerificationMethod {
    BorrowChecker,
    FormalProof { prover: String },
    StaticAnalysis { tool: String },
    ModelChecking { bounded: bool },
    AbstractInterpretation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Type classification for evidence.
pub enum EvidenceType {
    ImplicitTypeSystemGuarantee,
    ProofScriptReference {
        uri: String,
    },
    TheoremName {
        theorem: String,
        theory: Option<String>,
    },
    StaticAnalysisReport {
        report_id: String,
    },
    CertificateHash {
        hash: String,
        algorithm: String,
    },
}
