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

/// Derive a proof annotation's id from what it asserts, instead of drawing a
/// fresh random one.
///
/// DETERMINISM (round-3 sweep): `annotation_id` was `Uuid::new_v4()` at every
/// construction site, so `analyze proof-annotations --format json` gave every
/// annotation a NEW identity on every invocation — 1298 annotations whose
/// content was identical run to run but whose `annotationId` was different
/// every time. An identifier that changes when nothing changed cannot identify
/// anything: no baseline can be diffed and no annotation can be referenced.
///
/// `seed` must name the site and the claim (file, span, property, method,
/// tool). Identical input therefore yields an identical id, and two distinct
/// annotations yield distinct ids.
///
/// The digest is `DefaultHasher`, which is SipHash-1-3 with FIXED keys (unlike
/// `RandomState`, which seeds per process) — the same construction
/// `analyze duplicates` already relies on for its stable block hashes. Two
/// independent 64-bit digests are taken over differently-prefixed inputs to
/// fill the 128 bits. The result is stamped as UUID version 8 (RFC 9562
/// custom), which is exactly what it is: a vendor-defined, name-derived id —
/// claiming v4 (random) or v5 (SHA-1 name-based) would misdescribe it.
#[must_use]
pub fn derive_annotation_id(seed: &str) -> Uuid {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn digest(prefix: &str, seed: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        prefix.hash(&mut hasher);
        seed.hash(&mut hasher);
        hasher.finish()
    }

    let mut bytes = [0u8; 16];
    bytes[..8].copy_from_slice(&digest("pmat.proof.hi\u{1}", seed).to_be_bytes());
    bytes[8..].copy_from_slice(&digest("pmat.proof.lo\u{2}", seed).to_be_bytes());

    // Version 8 (custom) in the high nibble of byte 6, RFC 4122 variant in the
    // top bits of byte 8.
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}
