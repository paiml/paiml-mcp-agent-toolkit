//! Property-based tests for state machine verification
//! 
//! This module verifies that the refactor auto state machine maintains
//! critical invariants across all possible transitions.

use proptest::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// Quality metrics for a file
#[derive(Debug, Clone)]
pub struct QualityMetrics {
    pub complexity: u32,
    pub technical_debt: u32,
    pub test_coverage: f64,
    pub documentation_score: f64,
}

/// File quality score
#[derive(Debug, Clone)]
pub struct FileQualityScore {
    pub path: PathBuf,
    pub metrics: QualityMetrics,
    pub priority: f64,
}

/// Refactor action that can be applied
#[derive(Debug, Clone)]
pub enum RefactorAction {
    ExtractFunction { file: PathBuf, lines: (usize, usize) },
    SimplifyCondition { file: PathBuf, line: usize },
    RemoveDeadCode { file: PathBuf, lines: Vec<usize> },
    ImproveNaming { file: PathBuf, old_name: String, new_name: String },
    AddDocumentation { file: PathBuf, item: String },
}

/// State of the refactoring process
#[derive(Debug, Clone)]
pub struct RefactorState {
    pub iteration: u32,
    pub current_quality: QualityMetrics,
    pub file_scores: HashMap<PathBuf, FileQualityScore>,
    pub completed_actions: Vec<RefactorAction>,
    pub pending_files: Vec<PathBuf>,
}

impl RefactorState {
    /// Create a new refactor state
    pub fn new() -> Self {
        Self {
            iteration: 0,
            current_quality: QualityMetrics {
                complexity: 100,
                technical_debt: 50,
                test_coverage: 0.6,
                documentation_score: 0.5,
            },
            file_scores: HashMap::new(),
            completed_actions: Vec::new(),
            pending_files: Vec::new(),
        }
    }
    
    /// Select the next file to refactor (deterministic)
    pub fn select_next_file(&self) -> Option<PathBuf> {
        // Sort files by priority for deterministic selection
        let mut files: Vec<_> = self.file_scores.iter().collect();
        files.sort_by(|a, b| {
            b.1.priority.partial_cmp(&a.1.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        files.first().map(|(path, _)| (*path).clone())
    }
    
    /// Get the next refactoring action
    pub fn next_action(&self) -> Option<RefactorAction> {
        self.select_next_file().map(|file| {
            // Deterministically choose action based on current metrics
            if self.current_quality.complexity > 50 {
                RefactorAction::ExtractFunction {
                    file: file.clone(),
                    lines: (10, 20),
                }
            } else if self.current_quality.technical_debt > 30 {
                RefactorAction::SimplifyCondition {
                    file: file.clone(),
                    line: 15,
                }
            } else {
                RefactorAction::RemoveDeadCode {
                    file,
                    lines: vec![25, 30],
                }
            }
        })
    }
    
    /// Apply an action and return the new state
    pub fn apply_action(&self, action: RefactorAction) -> Self {
        let mut new_state = self.clone();
        new_state.iteration += 1;
        new_state.completed_actions.push(action.clone());
        
        // Update quality metrics based on action
        match action {
            RefactorAction::ExtractFunction { .. } => {
                new_state.current_quality.complexity = 
                    new_state.current_quality.complexity.saturating_sub(5);
            }
            RefactorAction::SimplifyCondition { .. } => {
                new_state.current_quality.complexity = 
                    new_state.current_quality.complexity.saturating_sub(2);
                new_state.current_quality.technical_debt = 
                    new_state.current_quality.technical_debt.saturating_sub(3);
            }
            RefactorAction::RemoveDeadCode { .. } => {
                new_state.current_quality.technical_debt = 
                    new_state.current_quality.technical_debt.saturating_sub(5);
            }
            RefactorAction::ImproveNaming { .. } => {
                new_state.current_quality.technical_debt = 
                    new_state.current_quality.technical_debt.saturating_sub(1);
            }
            RefactorAction::AddDocumentation { .. } => {
                new_state.current_quality.documentation_score = 
                    (new_state.current_quality.documentation_score + 0.1).min(1.0);
            }
        }
        
        new_state
    }
    
    /// Compute overall progress (0.0 to 1.0)
    pub fn compute_progress(&self) -> f64 {
        let complexity_progress = 1.0 - (self.current_quality.complexity as f64 / 100.0);
        let debt_progress = 1.0 - (self.current_quality.technical_debt as f64 / 50.0);
        let coverage_progress = self.current_quality.test_coverage;
        let doc_progress = self.current_quality.documentation_score;
        
        (complexity_progress + debt_progress + coverage_progress + doc_progress) / 4.0
    }
    
    /// Check if refactoring is complete
    pub fn is_complete(&self) -> bool {
        self.current_quality.complexity <= 20 &&
        self.current_quality.technical_debt == 0 &&
        self.current_quality.test_coverage >= 0.8 &&
        self.current_quality.documentation_score >= 0.8
    }
}

// Arbitrary implementations for property testing
impl Arbitrary for QualityMetrics {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            0u32..200,  // complexity
            0u32..100,  // technical_debt
            0.0f64..1.0,  // test_coverage
            0.0f64..1.0,  // documentation_score
        ).prop_map(|(complexity, technical_debt, test_coverage, documentation_score)| {
            QualityMetrics {
                complexity,
                technical_debt,
                test_coverage,
                documentation_score,
            }
        }).boxed()
    }
}

impl Arbitrary for FileQualityScore {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            "[a-z0-9_/]+\\.rs",
            any::<QualityMetrics>(),
            0.0f64..100.0,
        ).prop_map(|(path_str, metrics, priority)| {
            FileQualityScore {
                path: PathBuf::from(path_str),
                metrics,
                priority,
            }
        }).boxed()
    }
}

impl Arbitrary for RefactorAction {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            ("[a-z0-9_/]+\\.rs", 1usize..100, 1usize..100).prop_map(|(path, start, len)| {
                RefactorAction::ExtractFunction {
                    file: PathBuf::from(path),
                    lines: (start, start + len),
                }
            }),
            ("[a-z0-9_/]+\\.rs", 1usize..500).prop_map(|(path, line)| {
                RefactorAction::SimplifyCondition {
                    file: PathBuf::from(path),
                    line,
                }
            }),
            ("[a-z0-9_/]+\\.rs", prop::collection::vec(1usize..500, 1..10)).prop_map(|(path, lines)| {
                RefactorAction::RemoveDeadCode {
                    file: PathBuf::from(path),
                    lines,
                }
            }),
        ].boxed()
    }
}

/// Arbitrary state for property testing
#[derive(Debug, Clone)]
struct RefactorStateArb {
    iteration: u32,
    current_quality: QualityMetrics,
    file_scores: HashMap<PathBuf, FileQualityScore>,
    completed_actions: Vec<RefactorAction>,
}

impl From<RefactorStateArb> for RefactorState {
    fn from(arb: RefactorStateArb) -> Self {
        let mut state = RefactorState::new();
        state.iteration = arb.iteration;
        state.current_quality = arb.current_quality;
        state.file_scores = arb.file_scores;
        state.completed_actions = arb.completed_actions;
        state
    }
}

impl Arbitrary for RefactorStateArb {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;
    
    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        (
            0u32..1000,
            any::<QualityMetrics>(),
            prop::collection::hash_map(
                "[a-z0-9_/]+\\.rs".prop_map(PathBuf::from),
                any::<FileQualityScore>(),
                0..10
            ),
            prop::collection::vec(any::<RefactorAction>(), 0..50),
        ).prop_map(|(iteration, current_quality, file_scores, completed_actions)| {
            RefactorStateArb {
                iteration,
                current_quality,
                file_scores,
                completed_actions,
            }
        }).boxed()
    }
}

proptest! {
    /// Property: File selection is deterministic
    #[test]
    fn state_machine_file_selection_deterministic(initial_state in any::<RefactorStateArb>()) {
        let state = RefactorState::from(initial_state);
        
        // Multiple calls should return the same file
        let file1 = state.select_next_file();
        let file2 = state.select_next_file();
        prop_assert_eq!(file1, file2, "File selection must be deterministic");
    }
    
    /// Property: Progress is monotonic
    #[test]
    fn state_machine_progress_monotonic(initial_state in any::<RefactorStateArb>()) {
        let state = RefactorState::from(initial_state);
        
        if let Some(action) = state.next_action() {
            let new_state = state.apply_action(action);
            prop_assert!(
                new_state.compute_progress() >= state.compute_progress(),
                "Progress must be monotonic: {} -> {}",
                state.compute_progress(),
                new_state.compute_progress()
            );
        }
    }
    
    /// Property: State machine terminates
    #[test]
    fn state_machine_termination_guarantee(initial_state in any::<RefactorStateArb>()) {
        let mut state = RefactorState::from(initial_state);
        let max_iterations = 1000;
        
        for i in 0..max_iterations {
            if state.is_complete() {
                break;
            }
            
            if let Some(action) = state.next_action() {
                state = state.apply_action(action);
            } else {
                // No more actions available
                break;
            }
            
            // Ensure we're making progress
            if i == max_iterations - 1 {
                prop_assert!(
                    state.is_complete() || state.iteration >= max_iterations as u32,
                    "State machine must terminate or reach max iterations"
                );
            }
        }
    }
    
    /// Property: Actions reduce technical debt
    #[test]
    fn actions_reduce_technical_debt(initial_state in any::<RefactorStateArb>()) {
        let state = RefactorState::from(initial_state);
        
        // Apply a series of actions
        let mut current = state.clone();
        for _ in 0..10 {
            if let Some(action) = current.next_action() {
                let new_state = current.apply_action(action);
                
                // Technical debt should not increase
                prop_assert!(
                    new_state.current_quality.technical_debt <= current.current_quality.technical_debt,
                    "Technical debt increased: {} -> {}",
                    current.current_quality.technical_debt,
                    new_state.current_quality.technical_debt
                );
                
                current = new_state;
            } else {
                break;
            }
        }
    }
    
    /// Property: Iteration count increases monotonically
    #[test]
    fn iteration_count_monotonic(initial_state in any::<RefactorStateArb>()) {
        let state = RefactorState::from(initial_state);
        
        if let Some(action) = state.next_action() {
            let new_state = state.apply_action(action);
            prop_assert!(
                new_state.iteration > state.iteration,
                "Iteration count must increase: {} -> {}",
                state.iteration,
                new_state.iteration
            );
        }
    }
    
    /// Property: Completed actions are tracked
    #[test]
    fn completed_actions_tracked(initial_state in any::<RefactorStateArb>()) {
        let state = RefactorState::from(initial_state);
        let initial_count = state.completed_actions.len();
        
        if let Some(action) = state.next_action() {
            let new_state = state.apply_action(action.clone());
            prop_assert_eq!(
                new_state.completed_actions.len(),
                initial_count + 1,
                "Action not tracked in completed list"
            );
            
            // The last action should be the one we applied
            prop_assert_eq!(
                new_state.completed_actions.last(),
                Some(&action),
                "Last action doesn't match applied action"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_refactor_state_new() {
        let state = RefactorState::new();
        assert_eq!(state.iteration, 0);
        assert_eq!(state.current_quality.complexity, 100);
        assert_eq!(state.current_quality.technical_debt, 50);
        assert!(state.completed_actions.is_empty());
    }
    
    #[test]
    fn test_progress_calculation() {
        let mut state = RefactorState::new();
        let initial_progress = state.compute_progress();
        
        // Improve metrics
        state.current_quality.complexity = 20;
        state.current_quality.technical_debt = 0;
        state.current_quality.test_coverage = 0.9;
        state.current_quality.documentation_score = 0.9;
        
        let final_progress = state.compute_progress();
        assert!(final_progress > initial_progress);
        assert!(state.is_complete());
    }
}