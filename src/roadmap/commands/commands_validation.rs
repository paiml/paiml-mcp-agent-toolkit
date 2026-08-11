// Validation and quality check commands
// Included from mod.rs - shares parent module scope

/// What `roadmap validate` could actually establish about a sprint.
///
/// The verdict used to be computed from `incomplete_tasks` alone, so a sprint
/// with eight unchecked Definition-of-Done items — and a sprint with no tasks at
/// all — both printed "ready for release!".
#[derive(Debug, PartialEq, Eq)]
enum SprintReadiness {
    /// Every task complete and nothing left unevaluated.
    Ready,
    /// The sprint carries no tasks, so "all tasks completed" is vacuous.
    NoTasks,
    /// At least one task is not Completed.
    IncompleteTasks(usize),
    /// Tasks are done, but pmat holds no completion state for the sprint's
    /// Definition-of-Done items / quality gates, so readiness is not certified.
    Unverified { dod: usize, gates: usize },
}

fn assess_sprint_readiness(
    total_tasks: usize,
    incomplete_tasks: usize,
    unevaluated_dod: usize,
    unevaluated_gates: usize,
) -> SprintReadiness {
    if total_tasks == 0 {
        return SprintReadiness::NoTasks;
    }
    if incomplete_tasks > 0 {
        return SprintReadiness::IncompleteTasks(incomplete_tasks);
    }
    if unevaluated_dod > 0 || unevaluated_gates > 0 {
        return SprintReadiness::Unverified {
            dod: unevaluated_dod,
            gates: unevaluated_gates,
        };
    }
    SprintReadiness::Ready
}

async fn validate_sprint(
    roadmap_path: &Path,
    sprint_id: &str,
    strict: bool,
    config: &RoadmapConfig,
) -> Result<()> {
    println!("🔍 Validating sprint {sprint_id} for release...");

    let roadmap = Roadmap::from_file(roadmap_path)?;
    let sprint = roadmap
        .get_sprint(sprint_id)
        .context(format!("Sprint {sprint_id} not found"))?;

    // Check all tasks completed
    let incomplete_tasks: Vec<_> = sprint
        .tasks
        .iter()
        .filter(|t| t.status != TaskStatus::Completed)
        .collect();

    if sprint.tasks.is_empty() {
        println!("⚠️  Sprint has no tasks — there is nothing to validate");
    } else if incomplete_tasks.is_empty() {
        println!("✅ All tasks completed");
    } else {
        println!("❌ Incomplete tasks:");
        for task in &incomplete_tasks {
            println!("    {} - {}", task.id, task.description);
        }
    }

    // Definition of Done: these were rendered as "- [ ]" — a literal unchecked
    // box, printed whatever the real state was, and never folded into the
    // verdict. A `Sprint` stores no completion state for them, so render them
    // as what they are (a list pmat cannot check) rather than as a checkbox
    // whose state is invented.
    if !sprint.definition_of_done.is_empty() {
        println!("\n📋 Definition of Done (pmat holds no completion state for these):");
        for item in &sprint.definition_of_done {
            println!("  • {item}");
        }
    }

    let evaluated_gates = config.enforce_quality_gates;
    if evaluated_gates && !sprint.quality_gates.is_empty() {
        // Same defect, same fix: the loop only printed the gate names.
        println!("\n🔍 Quality Gates (declared, not evaluated here — run `pmat roadmap quality-check`):");
        for gate in &sprint.quality_gates {
            println!("  • {gate}");
        }
    }

    let unevaluated_gates = if evaluated_gates {
        sprint.quality_gates.len()
    } else {
        0
    };

    let readiness = assess_sprint_readiness(
        sprint.tasks.len(),
        incomplete_tasks.len(),
        sprint.definition_of_done.len(),
        unevaluated_gates,
    );

    match readiness {
        SprintReadiness::Ready => {
            println!("\n✅ Sprint {sprint_id} is ready for release!");
            Ok(())
        }
        SprintReadiness::NoTasks => {
            println!("\n❌ Sprint {sprint_id}: no tasks to validate — readiness not established");
            anyhow::bail!("Sprint {sprint_id} has no tasks; readiness cannot be established")
        }
        SprintReadiness::IncompleteTasks(n) => {
            println!("\n❌ Sprint {sprint_id} is NOT ready for release ({n} incomplete task(s))");
            // `--strict` used to be the only way this exited nonzero. A
            // validation command that exits 0 on a failed validation is a gate
            // that always passes, so the failure is now reported either way and
            // --strict only changes the wording.
            if strict {
                anyhow::bail!("Sprint validation failed (strict)");
            }
            anyhow::bail!("Sprint {sprint_id} is not ready for release: {n} incomplete task(s)")
        }
        SprintReadiness::Unverified { dod, gates } => {
            println!(
                "\n❌ Sprint {sprint_id}: readiness NOT certified — {dod} Definition-of-Done \
                 item(s) and {gates} quality gate(s) are not evaluated by pmat"
            );
            anyhow::bail!(
                "Sprint {sprint_id} readiness cannot be certified: {dod} definition-of-done item(s) \
                 and {gates} quality gate(s) are unevaluated"
            )
        }
    }
}

async fn quality_check(task_id: &str, config: &RoadmapConfig) -> Result<()> {
    // The id used to appear in nothing but the two println!s: `quality-check
    // --task-id TOTALLY-MADE-UP` passed, and its output differed from a real
    // id's by exactly those two lines. Resolve it against the roadmap first.
    let roadmap = Roadmap::from_file(&config.path)?;
    if roadmap.get_task(task_id).is_none() {
        anyhow::bail!(
            "unknown task '{task_id}' — not found in {}",
            config.path.display()
        );
    }

    // The checks below are repo-wide: a roadmap `Task` carries no file list, so
    // there is nothing to scope them to. Say that rather than letting the
    // heading imply the checks were about this task's code.
    println!("🔍 Running repo-wide quality checks for task {task_id}...");

    // Run complexity check. `--fail-on-violation` is what makes this a check:
    // without it `pmat analyze complexity` prints its findings and exits 0, so
    // the "Complexity check passed" line below was unconditional.
    let complexity_result = std::process::Command::new("pmat")
        .args([
            "analyze",
            "complexity",
            "--max-cyclomatic",
            &config.quality_gates.complexity_max.to_string(),
            "--fail-on-violation",
        ])
        .output()?;

    if !complexity_result.status.success() {
        println!("❌ Complexity check failed");
        anyhow::bail!("Complexity exceeds limit");
    }
    println!("✅ Complexity check passed");

    // Run SATD check
    let satd_result = std::process::Command::new("pmat")
        .args(["analyze", "satd", "--strict"])
        .output()?;

    if !satd_result.status.success() && config.quality_gates.satd_tolerance == 0 {
        println!("❌ SATD check failed");
        anyhow::bail!("SATD violations found");
    }
    println!("✅ SATD check passed");

    // Run lint check
    if config.quality_gates.lint_compliance {
        let lint_result = std::process::Command::new("make").args(["lint"]).output()?;

        if !lint_result.status.success() {
            println!("❌ Lint check failed");
            anyhow::bail!("Lint violations found");
        }
        println!("✅ Lint check passed");
    }

    println!("✅ All quality checks passed for task {task_id}");
    Ok(())
}

#[cfg(test)]
mod validation_verdict_tests {
    use super::{assess_sprint_readiness, quality_check, RoadmapConfig, SprintReadiness};

    #[test]
    fn a_sprint_with_no_tasks_is_not_ready() {
        // `roadmap validate --sprint v2.18.0` printed "All tasks completed"
        // and "ready for release!" for a sprint with no task rows at all.
        assert_eq!(
            assess_sprint_readiness(0, 0, 0, 0),
            SprintReadiness::NoTasks
        );
    }

    #[test]
    fn unevaluated_definition_of_done_blocks_the_ready_verdict() {
        // `--sprint v2.6.0`: every task complete, eight DoD items rendered as a
        // literal "- [ ]", verdict "ready for release!".
        assert_eq!(
            assess_sprint_readiness(4, 0, 8, 0),
            SprintReadiness::Unverified { dod: 8, gates: 0 }
        );
    }

    #[test]
    fn unevaluated_quality_gates_block_the_ready_verdict() {
        assert_eq!(
            assess_sprint_readiness(4, 0, 0, 3),
            SprintReadiness::Unverified { dod: 0, gates: 3 }
        );
    }

    #[test]
    fn incomplete_tasks_still_dominate_the_verdict() {
        assert_eq!(
            assess_sprint_readiness(4, 2, 8, 3),
            SprintReadiness::IncompleteTasks(2)
        );
    }

    #[test]
    fn ready_requires_tasks_done_and_nothing_left_unevaluated() {
        assert_eq!(assess_sprint_readiness(4, 0, 0, 0), SprintReadiness::Ready);
    }

    #[tokio::test]
    async fn quality_check_rejects_an_unknown_task_id() {
        // `roadmap quality-check --task-id TOTALLY-MADE-UP` exited 0 with
        // "All quality checks passed for task TOTALLY-MADE-UP".
        let dir = tempfile::tempdir().unwrap();
        let roadmap_path = dir.path().join("roadmap.md");
        std::fs::write(
            &roadmap_path,
            "# Roadmap\n\n## Sprint v1.0.0: Test\n\n### Tasks\n\n- [ ] PMAT-0001: Real task\n",
        )
        .unwrap();

        let config = RoadmapConfig {
            path: roadmap_path,
            ..Default::default()
        };

        let err = quality_check("TOTALLY-MADE-UP", &config)
            .await
            .expect_err("an unresolvable task id must fail the check");
        assert!(
            err.to_string().contains("unknown task"),
            "unexpected error: {err}"
        );
    }
}
