use std::sync::Arc;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Lightweight Provability Analyzer using Abstract Interpretation
/// Replaces heavyweight SMT solver approach with dataflow analysis
pub struct LightweightProvabilityAnalyzer {
    abstract_interpreter: AbstractInterpreter,
    proof_cache: Arc<DashMap<FunctionId, ProofSummary>>,
    current_version: u64,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct FunctionId {
    pub file_path: String,
    pub function_name: String,
    pub line_number: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofSummary {
    pub provability_score: f64,
    pub verified_properties: Vec<VerifiedProperty>,
    pub analysis_time_us: u128,
    pub version: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiedProperty {
    pub property_type: PropertyType,
    pub confidence: f64,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PropertyType {
    NullSafety,
    BoundsCheck,
    NoAliasing,
    PureFunction,
    MemorySafety,
    ThreadSafety,
}

/// Abstract domain for property analysis
#[derive(Clone, Debug)]
pub struct PropertyDomain {
    pub nullability: NullabilityLattice,
    pub bounds: IntervalLattice,
    pub aliasing: AliasLattice,
    pub purity: PurityLattice,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NullabilityLattice {
    Top,       // Unknown
    NotNull,   // Definitely not null
    MaybeNull, // Possibly null
    Null,      // Definitely null
    Bottom,    // Unreachable
}

#[derive(Clone, Debug)]
pub struct IntervalLattice {
    pub lower: Option<i64>,
    pub upper: Option<i64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AliasLattice {
    Top,       // Unknown aliasing
    NoAlias,   // No aliasing
    MayAlias,  // May have aliases
    MustAlias, // Definitely aliases
    Bottom,    // Unreachable
}

#[derive(Clone, Debug, PartialEq)]
pub enum PurityLattice {
    Top,         // Unknown purity
    Pure,        // Pure function
    ReadOnly,    // Only reads state
    WriteLocal,  // Only writes local state
    WriteGlobal, // Writes global state
    Bottom,      // Unreachable
}

pub struct AbstractInterpreter {
    #[allow(dead_code)]
    analysis_depth: usize,
}

impl PropertyDomain {
    pub fn top() -> Self {
        Self {
            nullability: NullabilityLattice::Top,
            bounds: IntervalLattice {
                lower: None,
                upper: None,
            },
            aliasing: AliasLattice::Top,
            purity: PurityLattice::Top,
        }
    }

    pub fn join(&self, other: &Self) -> Self {
        Self {
            nullability: self.nullability.join(&other.nullability),
            bounds: self.bounds.widen(&other.bounds),
            aliasing: self.aliasing.join(&other.aliasing),
            purity: self.purity.meet(&other.purity),
        }
    }

    pub fn widen(&self, other: &Self) -> Self {
        Self {
            nullability: self.nullability.join(&other.nullability),
            bounds: self.bounds.widen(&other.bounds),
            aliasing: self.aliasing.join(&other.aliasing),
            purity: self.purity.meet(&other.purity),
        }
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        self.nullability == other.nullability
            && self.bounds.is_equal(&other.bounds)
            && self.aliasing == other.aliasing
            && self.purity == other.purity
    }
}

impl NullabilityLattice {
    pub fn join(&self, other: &Self) -> Self {
        use NullabilityLattice::*;
        match (self, other) {
            (Bottom, x) | (x, Bottom) => x.clone(),
            (Top, _) | (_, Top) => Top,
            (NotNull, NotNull) => NotNull,
            (Null, Null) => Null,
            (NotNull, Null) | (Null, NotNull) => Bottom, // Contradiction
            _ => MaybeNull,
        }
    }
}

impl IntervalLattice {
    pub fn widen(&self, other: &Self) -> Self {
        let lower = match (self.lower, other.lower) {
            (Some(a), Some(b)) if a > b => None, // Widening to -∞
            (l, _) => l,
        };

        let upper = match (self.upper, other.upper) {
            (Some(a), Some(b)) if a < b => None, // Widening to +∞
            (u, _) => u,
        };

        Self { lower, upper }
    }

    pub fn is_equal(&self, other: &Self) -> bool {
        self.lower == other.lower && self.upper == other.upper
    }
}

impl AliasLattice {
    pub fn join(&self, other: &Self) -> Self {
        use AliasLattice::*;
        match (self, other) {
            (Bottom, x) | (x, Bottom) => x.clone(),
            (Top, _) | (_, Top) => Top,
            (NoAlias, NoAlias) => NoAlias,
            (MustAlias, MustAlias) => MustAlias,
            _ => MayAlias,
        }
    }
}

impl PurityLattice {
    pub fn meet(&self, other: &Self) -> Self {
        use PurityLattice::*;
        match (self, other) {
            (Bottom, _) | (_, Bottom) => Bottom,
            (WriteGlobal, _) | (_, WriteGlobal) => WriteGlobal,
            (WriteLocal, _) | (_, WriteLocal) => WriteLocal,
            (ReadOnly, ReadOnly) => ReadOnly,
            (Pure, Pure) => Pure,
            _ => Top,
        }
    }
}

impl LightweightProvabilityAnalyzer {
    pub fn new() -> Self {
        Self {
            abstract_interpreter: AbstractInterpreter { analysis_depth: 10 },
            proof_cache: Arc::new(DashMap::new()),
            current_version: 1,
        }
    }

    pub async fn analyze_incrementally(
        &self,
        changed_functions: &[FunctionId],
    ) -> Vec<ProofSummary> {
        let impact_set = self.compute_impact_set(changed_functions);

        impact_set
            .into_iter()
            .map(|func_id| {
                if let Some(cached) = self.proof_cache.get(&func_id) {
                    if cached.version == self.current_version {
                        return cached.clone();
                    }
                }

                let summary = self.analyze_function_fast(&func_id);
                self.proof_cache.insert(func_id.clone(), summary.clone());
                summary
            })
            .collect()
    }

    fn analyze_function_fast(&self, _func_id: &FunctionId) -> ProofSummary {
        let start = std::time::Instant::now();

        // Simulated fast analysis
        let mut state = PropertyDomain::top();
        let mut verified_properties = Vec::new();

        // Fixed-point iteration with widening
        for iteration in 0..3 {
            let new_state = self.abstract_interpreter.analyze_iteration(&state);

            if new_state.is_equal(&state) || iteration > 2 {
                break;
            }

            state = if iteration > 1 {
                state.widen(&new_state)
            } else {
                new_state
            };
        }

        // Extract verified properties
        if state.nullability == NullabilityLattice::NotNull {
            verified_properties.push(VerifiedProperty {
                property_type: PropertyType::NullSafety,
                confidence: 0.9,
                evidence: "Abstract interpretation proves non-null".to_string(),
            });
        }

        if state.bounds.lower.is_some() && state.bounds.upper.is_some() {
            verified_properties.push(VerifiedProperty {
                property_type: PropertyType::BoundsCheck,
                confidence: 0.85,
                evidence: format!(
                    "Bounds: [{:?}, {:?}]",
                    state.bounds.lower, state.bounds.upper
                ),
            });
        }

        if state.aliasing == AliasLattice::NoAlias {
            verified_properties.push(VerifiedProperty {
                property_type: PropertyType::NoAliasing,
                confidence: 0.8,
                evidence: "No aliasing detected".to_string(),
            });
        }

        if state.purity == PurityLattice::Pure {
            verified_properties.push(VerifiedProperty {
                property_type: PropertyType::PureFunction,
                confidence: 0.95,
                evidence: "Function has no side effects".to_string(),
            });
        }

        let provability_score = self.compute_confidence(&state);

        ProofSummary {
            provability_score,
            verified_properties,
            analysis_time_us: start.elapsed().as_micros(),
            version: self.current_version,
        }
    }

    fn compute_impact_set(&self, changed_functions: &[FunctionId]) -> Vec<FunctionId> {
        // In a real implementation, this would use call graph analysis
        // For now, just return the changed functions
        changed_functions.to_vec()
    }

    fn compute_confidence(&self, state: &PropertyDomain) -> f64 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Nullability confidence
        max_score += 1.0;
        match state.nullability {
            NullabilityLattice::NotNull => score += 1.0,
            NullabilityLattice::MaybeNull => score += 0.5,
            _ => {}
        }

        // Bounds confidence
        max_score += 1.0;
        if state.bounds.lower.is_some() && state.bounds.upper.is_some() {
            score += 1.0;
        } else if state.bounds.lower.is_some() || state.bounds.upper.is_some() {
            score += 0.5;
        }

        // Aliasing confidence
        max_score += 1.0;
        match state.aliasing {
            AliasLattice::NoAlias => score += 1.0,
            AliasLattice::MayAlias => score += 0.3,
            _ => {}
        }

        // Purity confidence
        max_score += 1.0;
        match state.purity {
            PurityLattice::Pure => score += 1.0,
            PurityLattice::ReadOnly => score += 0.7,
            PurityLattice::WriteLocal => score += 0.3,
            _ => {}
        }

        if max_score > 0.0 {
            score / max_score
        } else {
            0.0
        }
    }

    /// Calculate provability factor for TDG integration
    pub fn calculate_provability_factor(&self, summary: &ProofSummary) -> f64 {
        // Convert provability score (0-1) to factor (0-5) for TDG
        // Higher provability = lower technical debt
        let base_factor = 5.0 * (1.0 - summary.provability_score);

        // Adjust based on critical properties
        let critical_properties = summary
            .verified_properties
            .iter()
            .filter(|p| {
                matches!(
                    p.property_type,
                    PropertyType::MemorySafety | PropertyType::ThreadSafety
                )
            })
            .count();

        if critical_properties > 0 {
            base_factor * 0.7 // Reduce debt if critical properties are verified
        } else {
            base_factor
        }
    }
}

impl AbstractInterpreter {
    fn analyze_iteration(&self, state: &PropertyDomain) -> PropertyDomain {
        // Simplified iteration - in reality would analyze CFG
        let mut new_state = state.clone();

        // Simulate some analysis progress
        if state.nullability == NullabilityLattice::Top {
            new_state.nullability = NullabilityLattice::MaybeNull;
        }

        if state.bounds.lower.is_none() {
            new_state.bounds.lower = Some(0);
        }

        if state.purity == PurityLattice::Top {
            new_state.purity = PurityLattice::ReadOnly;
        }

        new_state
    }
}

impl Default for LightweightProvabilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nullability_lattice() {
        use NullabilityLattice::*;

        assert_eq!(NotNull.join(&NotNull), NotNull);
        assert_eq!(NotNull.join(&MaybeNull), MaybeNull);
        assert_eq!(NotNull.join(&Null), Bottom);
        assert_eq!(Top.join(&NotNull), Top);
    }

    #[test]
    fn test_property_domain_join() {
        let domain1 = PropertyDomain::top();
        let domain2 = PropertyDomain {
            nullability: NullabilityLattice::NotNull,
            bounds: IntervalLattice {
                lower: Some(0),
                upper: Some(100),
            },
            aliasing: AliasLattice::NoAlias,
            purity: PurityLattice::Pure,
        };

        let joined = domain1.join(&domain2);
        assert_eq!(joined.nullability, NullabilityLattice::Top);
    }

    #[tokio::test]
    async fn test_incremental_analysis() {
        let analyzer = LightweightProvabilityAnalyzer::new();

        let functions = vec![FunctionId {
            file_path: "src/main.rs".to_string(),
            function_name: "main".to_string(),
            line_number: 10,
        }];

        let summaries = analyzer.analyze_incrementally(&functions).await;
        assert!(!summaries.is_empty());
        assert!(summaries[0].analysis_time_us < 1000); // Should be fast (<1ms)
    }
}
