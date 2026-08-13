// CLI handlers for `pmat work triage` (ULTRA-003).

/// `pmat work triage record` — declare the bound of a bounded pass.
///
/// Refuses when the arithmetic does not close. The refusal is the feature: an
/// agent cannot log "examined 39, acted on 7" without naming the 32 it
/// dropped, so the omission has to become visible before it reaches a summary.
#[allow(clippy::too_many_arguments)]
pub async fn handle_work_triage_record(
    agent: String,
    scope: String,
    examined: u32,
    acted: u32,
    deferred: Vec<String>,
    reason: Option<String>,
    work_item: Option<String>,
    format: QaOutputFormat,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = claim_project_path(path)?;
    let now = chrono::Utc::now();
    let mut record = new_triage_record(&agent, &scope, examined, acted, now);
    record.deferred = deferred
        .into_iter()
        .filter(|d| !d.trim().is_empty())
        .collect();
    record.reason = reason.filter(|r| !r.trim().is_empty());
    record.work_item_id = work_item;

    let defects = record.audit();
    if !defects.is_empty() {
        render_triage_refusal(&record, &defects, format);
        anyhow::bail!(
            "work triage record: refusing to record an unaccounted pass ({} problem(s)); \
             name the {} unacted item(s) with --deferred and say why with --reason",
            defects.len(),
            record.gap()
        );
    }

    TriageLedger::new(&project_path).append(&record)?;
    render_triage_record(&record, format);
    Ok(())
}

/// `pmat work triage verify` — gate a work item on stated coverage.
///
/// Fails when nothing was measured. A ticket with zero triage records has not
/// proven full coverage; it has proven nothing, and that must not exit 0.
pub async fn handle_work_triage_verify(
    work_item: Option<String>,
    agent: Option<String>,
    format: QaOutputFormat,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = claim_project_path(path)?;
    let ledger = TriageLedger::new(&project_path);
    let all = ledger.load_records()?;
    let selected: Vec<TriageRecord> = all
        .into_iter()
        .filter(|r| {
            work_item
                .as_ref()
                .is_none_or(|w| r.work_item_id.as_ref() == Some(w))
        })
        .filter(|r| agent.as_ref().is_none_or(|a| &r.agent == a))
        .collect();
    let report = verify_triage_records(&selected);
    render_triage_verification(
        &report,
        work_item.as_deref(),
        &ledger.journal_path(),
        format,
    );

    if report.records == 0 {
        anyhow::bail!(
            "work triage verify: no triage record{} in {}; coverage of that pass was never \
             stated, which is not the same as complete",
            work_item
                .as_ref()
                .map(|w| format!(" for {w}"))
                .unwrap_or_default(),
            ledger.journal_path().display()
        );
    }
    if !report.unaccounted.is_empty() {
        anyhow::bail!(
            "work triage verify: {} record(s) do not account for every item examined",
            report.unaccounted.len()
        );
    }
    Ok(())
}

/// Render a refused triage record with every reason it was refused.
fn render_triage_refusal(record: &TriageRecord, defects: &[String], format: QaOutputFormat) {
    if matches!(format, QaOutputFormat::Json) {
        print_json(&serde_json::json!({
            "recorded": false,
            "record": record,
            "defects": defects,
        }));
        return;
    }
    println!(
        "{}",
        c::fail(&format!(
            "triage refused: examined {}, acted on {}, {} unaccounted",
            record.examined,
            record.acted,
            record.gap()
        ))
    );
    for d in defects {
        println!("    {d}");
    }
}

/// Render an accepted triage record.
fn render_triage_record(record: &TriageRecord, format: QaOutputFormat) {
    if matches!(format, QaOutputFormat::Json) {
        print_json(&serde_json::json!({ "recorded": true, "record": record }));
        return;
    }
    println!(
        "{}",
        c::pass(&format!(
            "triage recorded: {} examined {}, acted on {}, deferred {}",
            record.agent,
            record.examined,
            record.acted,
            record.deferred.len()
        ))
    );
    for d in &record.deferred {
        println!("    {} {}", c::dim("deferred"), d);
    }
    if let Some(reason) = &record.reason {
        println!("  {}", c::dim(&format!("reason: {reason}")));
    }
}

/// Render the verification report.
fn render_triage_verification(
    report: &TriageVerification,
    work_item: Option<&str>,
    journal: &Path,
    format: QaOutputFormat,
) {
    if matches!(format, QaOutputFormat::Json) {
        print_json(report);
        return;
    }
    println!(
        "{}",
        c::label(&format!(
            "🧮 Triage coverage{}: {} record(s), {} examined, {} acted, {} deferred",
            work_item.map(|w| format!(" for {w}")).unwrap_or_default(),
            report.records,
            report.examined,
            report.acted,
            report.deferred
        ))
    );
    for u in &report.unaccounted {
        println!(
            "  {}",
            c::fail(&format!("{} ({}): {}", u.record_id, u.agent, u.scope))
        );
        for d in &u.defects {
            println!("      {d}");
        }
    }
    if report.records == 0 {
        println!(
            "  {}",
            c::warn(&format!("nothing recorded in {}", journal.display()))
        );
    } else if report.ok() {
        println!("{}", c::pass("every examined item is accounted for"));
    }
}
