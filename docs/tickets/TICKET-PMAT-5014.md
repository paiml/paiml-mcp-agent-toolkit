# TICKET-PMAT-5014: Health Score Calculation

**Status**: GREEN
**Priority**: P0
**Complexity**: 8
**Estimated Time**: 4 hours
**Dependencies**: TICKET-PMAT-5010, TICKET-PMAT-5011, TICKET-PMAT-5012
**Sprint**: Sprint 17 - Maintenance Engine

## Objective

Calculate project health scores based on roadmap/ticket metrics. This provides quantitative measurement of project progress, velocity, and quality compliance - enabling data-driven decisions about project health.

## Success Criteria

- [ ] Calculate sprint completion velocity
- [ ] Track ticket aging metrics (tickets in RED status for >7 days)
- [ ] Analyze dependency graph health (circular dependencies, broken deps)
- [ ] Measure quality gate compliance rate
- [ ] Generate health dashboard data structure
- [ ] All quality gates pass (complexity <10, coverage >80%, no SATD)

## Test Strategy

### Unit Tests
- [ ] `test_calculate_sprint_velocity` - Measure completion rate
- [ ] `test_calculate_ticket_aging` - Identify stale tickets
- [ ] `test_analyze_dependency_graph` - Detect circular/broken deps
- [ ] `test_quality_gate_compliance` - Measure compliance rate
- [ ] `test_health_score_aggregation` - Combine metrics into overall score
- [ ] `test_health_trend_analysis` - Track score over time

### Property Tests
- [ ] Property: Health score is always 0-100
- [ ] Property: More completed tickets → higher score
- [ ] Property: Older RED tickets → lower score
- [ ] Property: Broken dependencies → lower score

### Integration Tests
- [ ] `integration_calculate_pmat_health` - Calculate PMAT's actual health
- [ ] `integration_health_report_generation` - Generate full report

## Quality Gates

- [ ] Cyclomatic complexity <10 for all functions
- [ ] Cognitive complexity <15 for all functions
- [ ] Line coverage >80%
- [ ] Branch coverage >80%
- [ ] 0 SATD violations
- [ ] 0 clippy warnings
- [ ] All tests pass

## Implementation Plan

### Phase 1: Data Structures

```rust
// server/src/maintenance/health.rs

use super::roadmap::Roadmap;
use super::ticket::TicketFile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Project health metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthScore {
    /// Overall health score (0-100)
    pub overall_score: f64,
    /// Individual metric scores
    pub metrics: HealthMetrics,
    /// Timestamp of calculation
    pub timestamp: String,
    /// Project version
    pub version: String,
}

/// Individual health metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Sprint velocity score (0-100)
    pub velocity_score: f64,
    /// Ticket aging score (0-100)
    pub aging_score: f64,
    /// Dependency health score (0-100)
    pub dependency_score: f64,
    /// Quality compliance score (0-100)
    pub quality_score: f64,
}

/// Sprint velocity metrics
#[derive(Debug, Clone, PartialEq)]
pub struct VelocityMetrics {
    /// Total tickets in roadmap
    pub total_tickets: usize,
    /// Completed tickets
    pub completed_tickets: usize,
    /// Completion percentage
    pub completion_rate: f64,
    /// Average time to complete (days)
    pub avg_completion_time: f64,
}

/// Ticket aging metrics
#[derive(Debug, Clone, PartialEq)]
pub struct AgingMetrics {
    /// Tickets in RED status
    pub red_tickets: Vec<String>,
    /// Tickets aged >7 days in RED
    pub stale_tickets: Vec<String>,
    /// Average age of RED tickets (days)
    pub avg_red_age: f64,
}

/// Dependency graph metrics
#[derive(Debug, Clone, PartialEq)]
pub struct DependencyMetrics {
    /// Total dependencies
    pub total_dependencies: usize,
    /// Broken dependencies
    pub broken_dependencies: usize,
    /// Circular dependency chains
    pub circular_dependencies: Vec<Vec<String>>,
}

/// Health calculation errors
#[derive(Debug, thiserror::Error)]
pub enum HealthError {
    #[error("Roadmap error: {0}")]
    RoadmapError(#[from] super::roadmap::RoadmapError),

    #[error("Ticket error: {0}")]
    TicketError(#[from] super::ticket::TicketError),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, HealthError>;
```

### Phase 2: Velocity Calculation

```rust
/// Calculate sprint velocity metrics
///
/// # Complexity
/// - Time: O(n) where n is number of tickets
/// - Cyclomatic: 3
pub fn calculate_velocity(roadmap: &Roadmap) -> VelocityMetrics {
    let total_tickets: usize = roadmap.sprints.iter()
        .map(|s| s.tickets.len())
        .sum();

    let completed_tickets: usize = roadmap.sprints.iter()
        .flat_map(|s| &s.tickets)
        .filter(|t| t.completed)
        .count();

    let completion_rate = if total_tickets > 0 {
        (completed_tickets as f64 / total_tickets as f64) * 100.0
    } else {
        0.0
    };

    VelocityMetrics {
        total_tickets,
        completed_tickets,
        completion_rate,
        avg_completion_time: 0.0, // TODO: Calculate from commit timestamps
    }
}

/// Convert velocity metrics to score (0-100)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 2
fn velocity_to_score(velocity: &VelocityMetrics) -> f64 {
    // Completion rate is already 0-100
    velocity.completion_rate
}
```

### Phase 3: Ticket Aging Analysis

```rust
/// Analyze ticket aging
///
/// # Complexity
/// - Time: O(n) where n is number of tickets
/// - Cyclomatic: 4
pub fn analyze_aging(tickets: &[TicketFile]) -> AgingMetrics {
    use super::ticket::TicketStatus;

    let red_tickets: Vec<String> = tickets.iter()
        .filter(|t| matches!(t.status, TicketStatus::Red))
        .map(|t| t.id.clone())
        .collect();

    // For now, consider all RED tickets as stale
    // TODO: Parse creation dates and calculate actual age
    let stale_tickets = red_tickets.clone();

    AgingMetrics {
        red_tickets: red_tickets.clone(),
        stale_tickets,
        avg_red_age: 0.0, // TODO: Calculate from ticket metadata
    }
}

/// Convert aging metrics to score (0-100)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn aging_to_score(aging: &AgingMetrics, total_tickets: usize) -> f64 {
    if total_tickets == 0 {
        return 100.0;
    }

    // More stale tickets = lower score
    let stale_ratio = aging.stale_tickets.len() as f64 / total_tickets as f64;
    (1.0 - stale_ratio) * 100.0
}
```

### Phase 4: Dependency Analysis

```rust
/// Analyze dependency graph health
///
/// # Complexity
/// - Time: O(n*m) where n=tickets, m=avg dependencies
/// - Cyclomatic: 5
pub fn analyze_dependencies(
    tickets: &[TicketFile],
    ticket_map: &HashMap<String, &TicketFile>,
) -> DependencyMetrics {
    let mut total_dependencies = 0;
    let mut broken_dependencies = 0;

    for ticket in tickets {
        total_dependencies += ticket.dependencies.len();

        for dep in &ticket.dependencies {
            if !ticket_map.contains_key(dep) {
                broken_dependencies += 1;
            }
        }
    }

    DependencyMetrics {
        total_dependencies,
        broken_dependencies,
        circular_dependencies: vec![], // TODO: Detect cycles using DFS
    }
}

/// Convert dependency metrics to score (0-100)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 3
fn dependency_to_score(deps: &DependencyMetrics) -> f64 {
    if deps.total_dependencies == 0 {
        return 100.0;
    }

    // More broken deps = lower score
    let broken_ratio = deps.broken_dependencies as f64 / deps.total_dependencies as f64;
    (1.0 - broken_ratio) * 100.0
}
```

### Phase 5: Health Score Aggregation

```rust
/// Calculate overall project health score
///
/// # Complexity
/// - Time: O(n*m) where n=tickets, m=avg dependencies
/// - Cyclomatic: 3
pub fn calculate_health_score(
    roadmap: &Roadmap,
    tickets: &[TicketFile],
) -> Result<HealthScore> {
    // Calculate individual metrics
    let velocity = calculate_velocity(roadmap);
    let aging = analyze_aging(tickets);

    let ticket_map: HashMap<_, _> = tickets.iter()
        .map(|t| (t.id.clone(), t))
        .collect();
    let dependencies = analyze_dependencies(tickets, &ticket_map);

    // Convert to scores
    let velocity_score = velocity_to_score(&velocity);
    let aging_score = aging_to_score(&aging, tickets.len());
    let dependency_score = dependency_to_score(&dependencies);
    let quality_score = 100.0; // TODO: Calculate from quality gate results

    // Aggregate into overall score (weighted average)
    let overall_score = (
        velocity_score * 0.4 +
        aging_score * 0.3 +
        dependency_score * 0.2 +
        quality_score * 0.1
    );

    Ok(HealthScore {
        overall_score,
        metrics: HealthMetrics {
            velocity_score,
            aging_score,
            dependency_score,
            quality_score,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: roadmap.version.clone(),
    })
}

/// Format health score as markdown report
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 4
pub fn format_health_report(health: &HealthScore) -> String {
    let mut output = String::new();

    output.push_str(&format!("# Project Health Report: {}\n\n", health.version));
    output.push_str(&format!("**Timestamp**: {}\n\n", health.timestamp));

    // Overall score with color coding
    let status_emoji = if health.overall_score >= 80.0 {
        "✅"
    } else if health.overall_score >= 60.0 {
        "⚠️"
    } else {
        "❌"
    };

    output.push_str(&format!(
        "## {} Overall Health: {:.1}%\n\n",
        status_emoji, health.overall_score
    ));

    // Individual metrics
    output.push_str("### Detailed Metrics\n\n");
    output.push_str(&format!("- **Velocity**: {:.1}%\n", health.metrics.velocity_score));
    output.push_str(&format!("- **Ticket Aging**: {:.1}%\n", health.metrics.aging_score));
    output.push_str(&format!("- **Dependencies**: {:.1}%\n", health.metrics.dependency_score));
    output.push_str(&format!("- **Quality Gates**: {:.1}%\n", health.metrics.quality_score));

    output
}
```

### Phase 6: Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::maintenance::roadmap::{Sprint, SprintStatus, Ticket};
    use crate::maintenance::ticket::{TicketStatus, Priority};
    use std::path::PathBuf;

    #[test]
    fn test_calculate_velocity_empty() {
        let roadmap = Roadmap {
            version: "v1.0.0".into(),
            sprints: vec![],
        };

        let velocity = calculate_velocity(&roadmap);

        assert_eq!(velocity.total_tickets, 0);
        assert_eq!(velocity.completed_tickets, 0);
        assert_eq!(velocity.completion_rate, 0.0);
    }

    #[test]
    fn test_calculate_velocity_partial() {
        let roadmap = Roadmap {
            version: "v1.0.0".into(),
            sprints: vec![Sprint {
                number: 1,
                name: "Test".into(),
                focus: "Testing".into(),
                status: SprintStatus::InProgress,
                duration: "2 days".into(),
                tickets: vec![
                    Ticket {
                        id: "TICKET-1".into(),
                        description: "Test".into(),
                        completed: true,
                        commit: Some("abc".into()),
                    },
                    Ticket {
                        id: "TICKET-2".into(),
                        description: "Test".into(),
                        completed: false,
                        commit: None,
                    },
                ],
                quality_gates: vec![],
            }],
        };

        let velocity = calculate_velocity(&roadmap);

        assert_eq!(velocity.total_tickets, 2);
        assert_eq!(velocity.completed_tickets, 1);
        assert_eq!(velocity.completion_rate, 50.0);
    }

    #[test]
    fn test_analyze_aging_no_red_tickets() {
        let tickets = vec![
            TicketFile {
                id: "TICKET-1".into(),
                title: "Test".into(),
                status: TicketStatus::Green,
                priority: Priority::P0,
                complexity: 5,
                estimated_time: "1h".into(),
                dependencies: vec![],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec![],
                file_path: PathBuf::new(),
            },
        ];

        let aging = analyze_aging(&tickets);

        assert_eq!(aging.red_tickets.len(), 0);
        assert_eq!(aging.stale_tickets.len(), 0);
    }

    #[test]
    fn test_analyze_aging_with_red_tickets() {
        let tickets = vec![
            TicketFile {
                id: "TICKET-1".into(),
                title: "Test".into(),
                status: TicketStatus::Red,
                priority: Priority::P0,
                complexity: 5,
                estimated_time: "1h".into(),
                dependencies: vec![],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec![],
                file_path: PathBuf::new(),
            },
        ];

        let aging = analyze_aging(&tickets);

        assert_eq!(aging.red_tickets.len(), 1);
        assert_eq!(aging.red_tickets[0], "TICKET-1");
    }

    #[test]
    fn test_analyze_dependencies_no_broken() {
        let tickets = vec![
            TicketFile {
                id: "TICKET-1".into(),
                title: "Test".into(),
                status: TicketStatus::Green,
                priority: Priority::P0,
                complexity: 5,
                estimated_time: "1h".into(),
                dependencies: vec!["TICKET-2".into()],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec![],
                file_path: PathBuf::new(),
            },
            TicketFile {
                id: "TICKET-2".into(),
                title: "Test".into(),
                status: TicketStatus::Green,
                priority: Priority::P0,
                complexity: 5,
                estimated_time: "1h".into(),
                dependencies: vec![],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec![],
                file_path: PathBuf::new(),
            },
        ];

        let ticket_map: HashMap<_, _> = tickets.iter()
            .map(|t| (t.id.clone(), t))
            .collect();

        let deps = analyze_dependencies(&tickets, &ticket_map);

        assert_eq!(deps.total_dependencies, 1);
        assert_eq!(deps.broken_dependencies, 0);
    }

    #[test]
    fn test_analyze_dependencies_with_broken() {
        let tickets = vec![
            TicketFile {
                id: "TICKET-1".into(),
                title: "Test".into(),
                status: TicketStatus::Green,
                priority: Priority::P0,
                complexity: 5,
                estimated_time: "1h".into(),
                dependencies: vec!["TICKET-MISSING".into()],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec![],
                file_path: PathBuf::new(),
            },
        ];

        let ticket_map: HashMap<_, _> = tickets.iter()
            .map(|t| (t.id.clone(), t))
            .collect();

        let deps = analyze_dependencies(&tickets, &ticket_map);

        assert_eq!(deps.total_dependencies, 1);
        assert_eq!(deps.broken_dependencies, 1);
    }

    #[test]
    fn test_calculate_health_score() {
        let roadmap = Roadmap {
            version: "v1.0.0".into(),
            sprints: vec![Sprint {
                number: 1,
                name: "Test".into(),
                focus: "Testing".into(),
                status: SprintStatus::InProgress,
                duration: "2 days".into(),
                tickets: vec![
                    Ticket {
                        id: "TICKET-1".into(),
                        description: "Test".into(),
                        completed: true,
                        commit: Some("abc".into()),
                    },
                ],
                quality_gates: vec![],
            }],
        };

        let tickets = vec![
            TicketFile {
                id: "TICKET-1".into(),
                title: "Test".into(),
                status: TicketStatus::Green,
                priority: Priority::P0,
                complexity: 5,
                estimated_time: "1h".into(),
                dependencies: vec![],
                sprint: "Sprint 1".into(),
                objective: "Test".into(),
                success_criteria: vec![],
                file_path: PathBuf::new(),
            },
        ];

        let health = calculate_health_score(&roadmap, &tickets).unwrap();

        assert!(health.overall_score >= 0.0);
        assert!(health.overall_score <= 100.0);
        assert_eq!(health.version, "v1.0.0");
    }

    #[test]
    fn test_format_health_report() {
        let health = HealthScore {
            overall_score: 85.5,
            metrics: HealthMetrics {
                velocity_score: 90.0,
                aging_score: 80.0,
                dependency_score: 85.0,
                quality_score: 90.0,
            },
            timestamp: "2025-10-05T10:00:00Z".into(),
            version: "v1.0.0".into(),
        };

        let report = format_health_report(&health);

        assert!(report.contains("Project Health Report"));
        assert!(report.contains("85.5%"));
        assert!(report.contains("Velocity: 90.0%"));
    }
}
```

## Complexity Analysis

Functions with complexity:
- `calculate_velocity`: CC=3
- `velocity_to_score`: CC=2
- `analyze_aging`: CC=4
- `aging_to_score`: CC=3
- `analyze_dependencies`: CC=5
- `dependency_to_score`: CC=3
- `calculate_health_score`: CC=3
- `format_health_report`: CC=4

All functions under CC=10 threshold ✓

## Verification Commands

```bash
# Run tests
cargo test --lib maintenance::health

# Calculate PMAT's health
cargo run --bin pmat -- maintain health

# Generate health report
cargo run --bin pmat -- maintain health --report
```

## Files to Create/Modify

### New Files
- `server/src/maintenance/health.rs` - Health score calculation

### Modified Files
- `server/src/maintenance/mod.rs` - Add health module

## Risk Assessment

**Low Risk:**
- Read-only calculations
- No side effects
- Clear metric definitions

**Mitigation:**
- Comprehensive unit tests
- Property tests for score bounds
- Integration test on real PMAT data

## Notes

This ticket provides quantitative health metrics for project management:
- **Velocity**: Track sprint completion progress
- **Aging**: Identify stuck/stale tickets
- **Dependencies**: Detect broken/circular dependencies
- **Quality**: Measure quality gate compliance

Combined with TICKET-PMAT-5013 (auto-updates), this creates a fully automated project health monitoring system.

**TDD Cycle Duration**: Estimated 3-4 hours for RED → GREEN → REFACTOR
