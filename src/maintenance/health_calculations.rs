// Health score calculation functions: velocity, aging, dependency analysis,
// overall scoring, and markdown report formatting.

/// Calculate sprint velocity metrics
///
/// # Complexity
/// - Time: O(n) where n is number of tickets
/// - Cyclomatic: 3
pub fn calculate_velocity(roadmap: &Roadmap) -> VelocityMetrics {
    let total_tickets: usize = roadmap.sprints.iter().map(|s| s.tickets.len()).sum();

    let completed_tickets: usize = roadmap
        .sprints
        .iter()
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
    }
}

/// Convert velocity metrics to score (0-100)
///
/// # Complexity
/// - Time: O(1)
/// - Cyclomatic: 1
fn velocity_to_score(velocity: &VelocityMetrics) -> f64 {
    // Completion rate is already 0-100
    velocity.completion_rate
}

/// Analyze ticket aging
///
/// # Complexity
/// - Time: O(n) where n is number of tickets
/// - Cyclomatic: 3
pub fn analyze_aging(tickets: &[TicketFile]) -> AgingMetrics {
    use super::ticket::TicketStatus;

    let red_tickets: Vec<String> = tickets
        .iter()
        .filter(|t| matches!(t.status, TicketStatus::Red))
        .map(|t| t.id.clone())
        .collect();

    // For simplicity, consider all RED tickets as stale
    let stale_tickets = red_tickets.clone();

    AgingMetrics {
        red_tickets,
        stale_tickets,
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

/// Calculate overall project health score
///
/// # Complexity
/// - Time: O(n*m) where n=tickets, m=avg dependencies
/// - Cyclomatic: 2
pub fn calculate_health_score(roadmap: &Roadmap, tickets: &[TicketFile]) -> Result<HealthScore> {
    // Calculate individual metrics
    let velocity = calculate_velocity(roadmap);
    let aging = analyze_aging(tickets);

    let ticket_map: HashMap<_, _> = tickets.iter().map(|t| (t.id.clone(), t)).collect();
    let dependencies = analyze_dependencies(tickets, &ticket_map);

    // Convert to scores
    let velocity_score = velocity_to_score(&velocity);
    let aging_score = aging_to_score(&aging, tickets.len());
    let dependency_score = dependency_to_score(&dependencies);
    let quality_score = 100.0; // Default: assume quality gates pass

    // Aggregate into overall score (weighted average)
    let overall_score =
        velocity_score * 0.4 + aging_score * 0.3 + dependency_score * 0.2 + quality_score * 0.1;

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
        "\u{2705}"
    } else if health.overall_score >= 60.0 {
        "\u{26a0}\u{fe0f}"
    } else {
        "\u{274c}"
    };

    output.push_str(&format!(
        "## {} Overall Health: {:.1}%\n\n",
        status_emoji, health.overall_score
    ));

    // Individual metrics
    output.push_str("### Detailed Metrics\n\n");
    output.push_str(&format!(
        "- **Velocity**: {:.1}%\n",
        health.metrics.velocity_score
    ));
    output.push_str(&format!(
        "- **Ticket Aging**: {:.1}%\n",
        health.metrics.aging_score
    ));
    output.push_str(&format!(
        "- **Dependencies**: {:.1}%\n",
        health.metrics.dependency_score
    ));
    output.push_str(&format!(
        "- **Quality Gates**: {:.1}%\n",
        health.metrics.quality_score
    ));

    output
}
