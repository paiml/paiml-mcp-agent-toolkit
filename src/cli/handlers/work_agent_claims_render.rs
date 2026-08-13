// Rendering for `pmat work claim` (ULTRA-002).
//
// Text output goes through `crate::cli::colors`, which honours `--color`;
// JSON output is the same data with nothing dropped, so an orchestrator can
// parse the verdict instead of grepping prose.

use crate::cli::colors as c;

/// Print JSON, or fall back to a parseable error object rather than silence.
fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{s}"),
        Err(e) => println!("{{\"error\":\"failed to serialize: {e}\"}}"),
    }
}

/// Render the outcome of an acquire.
pub fn render_acquire_outcome(outcome: &FileClaimAcquireOutcome, format: QaOutputFormat) {
    if matches!(format, QaOutputFormat::Json) {
        print_json(outcome);
        return;
    }
    if outcome.granted {
        render_acquire_granted(outcome);
    } else {
        render_acquire_refused(outcome);
    }
}

fn render_acquire_granted(outcome: &FileClaimAcquireOutcome) {
    println!(
        "{}",
        c::pass(&format!(
            "claim granted to {} for {} path(s), ttl {}s",
            outcome.agent,
            outcome.paths.len(),
            outcome.ttl_secs
        ))
    );
    for p in &outcome.paths {
        println!("    {}", c::path(p));
    }
    for a in &outcome.superseded_expired {
        println!(
            "  {}",
            c::warn(&format!(
                "superseded a lapsed claim held by {a} (its TTL had run out)"
            ))
        );
    }
    if let Some(id) = &outcome.record_id {
        println!("  {}", c::dim(&format!("record {id}")));
    }
}

fn render_acquire_refused(outcome: &FileClaimAcquireOutcome) {
    println!(
        "{}",
        c::fail(&format!(
            "claim refused for {}: {} of {} path(s) are held",
            outcome.agent,
            outcome.conflicts.len(),
            outcome.paths.len()
        ))
    );
    for k in &outcome.conflicts {
        println!("    {}", render_conflict_line(k));
    }
    if let Some(k) = outcome.conflicts.first() {
        println!(
            "  {}",
            c::dim(&format!(
                "the holder frees it with: pmat work claim release {} --agent {}",
                k.held_path, k.held_by
            ))
        );
    }
}

/// One conflict row: which path, who holds it, and how long that lasts.
fn render_conflict_line(k: &FileClaimConflict) -> String {
    let ticket = k
        .work_item_id
        .as_ref()
        .map(|t| format!(" [{t}]"))
        .unwrap_or_default();
    format!(
        "{} held by {} via {} for another {}s{}",
        c::path(&k.path),
        k.held_by,
        k.held_path,
        k.seconds_remaining,
        ticket
    )
}

/// Render a release record.
pub fn render_release(record: &FileClaimRecord, format: QaOutputFormat) {
    if matches!(format, QaOutputFormat::Json) {
        print_json(record);
        return;
    }
    println!(
        "{}",
        c::pass(&format!(
            "released {} path(s) held by {}",
            record.paths.len(),
            record.agent
        ))
    );
    for p in &record.paths {
        println!("    {}", c::path(p));
    }
    if let Some(reason) = &record.forced_reason {
        println!("  {}", c::warn(&format!("forced release: {reason}")));
    }
}

/// Payload for `pmat work claim list --format json`.
#[derive(Debug, Serialize)]
struct ClaimListReport<'a> {
    journal: String,
    include_expired: bool,
    active: usize,
    expired: usize,
    claims: &'a [ActiveFileClaim],
}

/// Render the active-claim table.
pub fn render_claim_list(
    claims: &[ActiveFileClaim],
    include_expired: bool,
    journal: &Path,
    format: QaOutputFormat,
) {
    let expired = claims.iter().filter(|c| c.expired).count();
    if matches!(format, QaOutputFormat::Json) {
        print_json(&ClaimListReport {
            journal: journal.display().to_string(),
            include_expired,
            active: claims.len() - expired,
            expired,
            claims,
        });
        return;
    }
    println!(
        "{}",
        c::label(&format!(
            "🔒 File claims: {} live{}",
            claims.len() - expired,
            if include_expired {
                format!(", {expired} lapsed")
            } else {
                String::new()
            }
        ))
    );
    for claim in claims {
        println!("    {}", render_claim_line(claim));
    }
    if claims.is_empty() {
        println!(
            "  {}",
            c::dim(&format!("no claims recorded in {}", journal.display()))
        );
    }
}

fn render_claim_line(claim: &ActiveFileClaim) -> String {
    let ticket = claim
        .work_item_id
        .as_ref()
        .map(|t| format!(" [{t}]"))
        .unwrap_or_default();
    let state = if claim.expired { " (LAPSED)" } else { "" };
    format!(
        "{}  {}{}{}  until {}",
        c::path(&claim.path),
        claim.agent,
        ticket,
        state,
        claim.expires_at
    )
}

/// Payload for `pmat work claim check --format json`.
#[derive(Debug, Serialize)]
struct ClaimCheckReport<'a> {
    checked: &'a [String],
    free: usize,
    claimed: usize,
    conflicts: &'a [FileClaimConflict],
}

/// Render a check verdict.
pub fn render_check(paths: &[String], conflicts: &[FileClaimConflict], format: QaOutputFormat) {
    if matches!(format, QaOutputFormat::Json) {
        print_json(&ClaimCheckReport {
            checked: paths,
            free: paths.len().saturating_sub(conflicts.len()),
            claimed: conflicts.len(),
            conflicts,
        });
        return;
    }
    if conflicts.is_empty() {
        println!(
            "{}",
            c::pass(&format!("all {} path(s) are free to claim", paths.len()))
        );
        return;
    }
    println!(
        "{}",
        c::fail(&format!(
            "{} of {} path(s) are already claimed",
            conflicts.len(),
            paths.len()
        ))
    );
    for k in conflicts {
        println!("    {}", render_conflict_line(k));
    }
}
