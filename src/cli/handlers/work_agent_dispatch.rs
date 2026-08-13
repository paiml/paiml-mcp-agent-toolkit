// Subcommand dispatch for `pmat work claim` and `pmat work triage`.
//
// Kept here rather than in the command dispatcher so the whole feature lives
// with the ledger it extends; the dispatcher only needs to know these two
// entry points exist.

use crate::cli::commands::{WorkClaimCommands, WorkTriageCommands};

/// `--force` is an accountability lever, so it must carry its reason. Silent
/// forcing is how a claim system stops meaning anything.
fn force_reason(force: bool, reason: &Option<String>) -> Result<Option<String>> {
    match (
        force,
        reason.as_ref().map(|r| r.trim()).filter(|r| !r.is_empty()),
    ) {
        (false, _) => Ok(None),
        (true, Some(r)) => Ok(Some(r.to_string())),
        (true, None) => anyhow::bail!(
            "--force requires --reason: taking a path from another agent has to be attributable"
        ),
    }
}

/// Route `pmat work claim <sub>`.
pub async fn dispatch_work_claim(command: &WorkClaimCommands) -> Result<()> {
    match command {
        WorkClaimCommands::Acquire {
            paths,
            agent,
            ttl,
            work_item,
            note,
            force,
            reason,
            format,
            path,
        } => {
            handle_work_claim_acquire(
                paths.clone(),
                agent.clone(),
                *ttl,
                work_item.clone(),
                note.clone(),
                force_reason(*force, reason)?,
                *format,
                path.clone(),
            )
            .await
        }
        WorkClaimCommands::Release {
            paths,
            agent,
            all,
            force,
            reason,
            format,
            path,
        } => {
            handle_work_claim_release(
                paths.clone(),
                agent.clone(),
                *all,
                force_reason(*force, reason)?,
                *format,
                path.clone(),
            )
            .await
        }
        WorkClaimCommands::List {
            agent,
            include_expired,
            format,
            path,
        } => handle_work_claim_list(agent.clone(), *include_expired, *format, path.clone()).await,
        WorkClaimCommands::Check {
            paths,
            agent,
            format,
            path,
        } => handle_work_claim_check(paths.clone(), agent.clone(), *format, path.clone()).await,
    }
}

/// Route `pmat work triage <sub>`.
pub async fn dispatch_work_triage(command: &WorkTriageCommands) -> Result<()> {
    match command {
        WorkTriageCommands::Record {
            agent,
            scope,
            examined,
            acted,
            deferred,
            reason,
            work_item,
            format,
            path,
        } => {
            handle_work_triage_record(
                agent.clone(),
                scope.clone(),
                *examined,
                *acted,
                deferred.clone(),
                reason.clone(),
                work_item.clone(),
                *format,
                path.clone(),
            )
            .await
        }
        WorkTriageCommands::Verify {
            work_item,
            agent,
            format,
            path,
        } => {
            handle_work_triage_verify(work_item.clone(), agent.clone(), *format, path.clone()).await
        }
    }
}
