//! `pmat work link` (AD-07): record a commit or a pull request on a ticket.
//!
//! The commit trailer `Pmat-Ticket:` is the machine-checkable record that a
//! commit belongs to a ticket (comply check CB-1340 judges it). This command
//! records the other direction — ticket → commit / PR — on the roadmap item
//! itself, so `pmat work annotate`, release notes and receipts can name the
//! artefacts a ticket produced.
//!
//! Links are written through [`RoadmapService`], the same loader/saver
//! `pmat work edit` uses; nothing here writes YAML by hand. Recording a link
//! that is already present is a no-op (idempotent), so an orchestrator may
//! replay its Phase-4 step safely.

use crate::models::roadmap::WorkLink;
use crate::services::roadmap_service::RoadmapService;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// Resolve a commit-ish to its full 40-char sha inside `project_path`.
///
/// `git rev-parse --verify <rev>^{commit}` fails for a tag, a tree or an
/// unknown object, so a typo cannot be recorded as a link.
pub fn resolve_commit(project_path: &Path, rev: &str) -> Result<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(project_path)
        .args(["rev-parse", "--verify", "--quiet"])
        .arg(format!("{rev}^{{commit}}"))
        .output()
        .context("failed to run git rev-parse")?;
    if !out.status.success() {
        anyhow::bail!(
            "'{}' is not a commit in {} (git rev-parse --verify failed)",
            rev,
            project_path.display()
        );
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if sha.is_empty() {
        anyhow::bail!("git rev-parse returned no sha for '{}'", rev);
    }
    Ok(sha)
}

/// Add `link` to `links` unless the same target is already recorded.
/// Returns true when the list grew.
pub fn push_link_idempotent(links: &mut Vec<WorkLink>, link: WorkLink) -> bool {
    if links.iter().any(|l| l.target == link.target) {
        return false;
    }
    links.push(link);
    true
}

/// Handle `pmat work link <id> --commit <sha> | --pr <n>`.
pub async fn handle_work_link(
    id: String,
    commit: Option<String>,
    pr: Option<u64>,
    path: Option<PathBuf>,
) -> Result<()> {
    use crate::cli::colors as c;

    if commit.is_none() && pr.is_none() {
        anyhow::bail!("one of --commit <sha> or --pr <number> is required");
    }

    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    let roadmap_path = project_path.join("docs/roadmaps/roadmap.yaml");
    let service = RoadmapService::new(&roadmap_path);
    if !service.exists() {
        anyhow::bail!(
            "No roadmap found at {}. Run 'pmat work init' first.",
            roadmap_path.display()
        );
    }

    let roadmap = service.load()?;
    let mut item = roadmap
        .roadmap
        .iter()
        .find(|i| i.id == id || i.id.eq_ignore_ascii_case(&id))
        .cloned()
        .with_context(|| format!("No ticket '{}' in {}", id, roadmap_path.display()))?;

    let now = chrono::Utc::now().to_rfc3339();
    let mut recorded = Vec::new();

    if let Some(rev) = commit {
        let sha = resolve_commit(&project_path, &rev)?;
        let link = WorkLink::commit(sha, now.clone());
        let text = link.display();
        if push_link_idempotent(&mut item.links, link) {
            recorded.push(format!("Linked {} to {}", text, item.id));
        } else {
            recorded.push(format!("{} already linked to {}", text, item.id));
        }
    }

    if let Some(number) = pr {
        let link = WorkLink::pr(number, now);
        let text = link.display();
        if push_link_idempotent(&mut item.links, link) {
            recorded.push(format!("Linked {} to {}", text, item.id));
        } else {
            recorded.push(format!("{} already linked to {}", text, item.id));
        }
    }

    service.upsert_item(item)?;
    for line in recorded {
        println!("{}", c::pass(&line));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::roadmap::{ItemStatus, Roadmap, RoadmapItem, WorkLinkTarget};

    fn fixture(dir: &Path) -> RoadmapService {
        std::fs::create_dir_all(dir.join("docs/roadmaps")).expect("mkdir");
        let mut item = RoadmapItem::new("PMAT-1".to_string(), "in progress".to_string());
        item.status = ItemStatus::InProgress;
        let roadmap = Roadmap {
            roadmap: vec![item],
            ..Default::default()
        };
        let service = RoadmapService::new(dir.join("docs/roadmaps/roadmap.yaml"));
        service.save(&roadmap).expect("save");
        service
    }

    fn git(dir: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .expect("git");
        assert!(out.status.success(), "git {args:?} failed");
    }

    fn init_repo(dir: &Path) -> String {
        git(dir, &["init", "-q", "--template=", "-b", "master"]);
        git(dir, &["config", "user.email", "t@t"]);
        git(dir, &["config", "user.name", "t"]);
        git(dir, &["config", "commit.gpgsign", "false"]);
        std::fs::write(dir.join("a.txt"), "a").expect("write");
        git(dir, &["add", "."]);
        git(dir, &["commit", "-qm", "init"]);
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("git");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    #[tokio::test]
    async fn link_commit_stores_full_sha_and_is_idempotent() {
        let tmp = tempfile::tempdir().expect("tmp");
        let dir = tmp.path();
        let service = fixture(dir);
        let sha = init_repo(dir);

        handle_work_link(
            "PMAT-1".into(),
            Some(sha[..8].to_string()),
            None,
            Some(dir.to_path_buf()),
        )
        .await
        .expect("link commit");

        let item = service.find_item("PMAT-1").expect("load").expect("item");
        assert_eq!(item.links.len(), 1);
        assert_eq!(
            item.links[0].target,
            WorkLinkTarget::Commit(sha.clone()),
            "the full sha is stored"
        );

        // Replaying the same link records nothing new.
        handle_work_link(
            "PMAT-1".into(),
            Some(sha.clone()),
            None,
            Some(dir.to_path_buf()),
        )
        .await
        .expect("link again");
        let item = service.find_item("PMAT-1").expect("load").expect("item");
        assert_eq!(item.links.len(), 1, "duplicate link is a no-op");
    }

    #[tokio::test]
    async fn link_pr_is_stored_as_a_number() {
        let tmp = tempfile::tempdir().expect("tmp");
        let dir = tmp.path();
        let service = fixture(dir);
        init_repo(dir);

        handle_work_link("PMAT-1".into(), None, Some(42), Some(dir.to_path_buf()))
            .await
            .expect("link pr");
        let item = service.find_item("PMAT-1").expect("load").expect("item");
        assert_eq!(item.links.len(), 1);
        assert_eq!(item.links[0].target, WorkLinkTarget::Pr(42));
        assert_eq!(item.links[0].display(), "PR #42");
    }

    #[tokio::test]
    async fn link_requires_commit_or_pr() {
        let tmp = tempfile::tempdir().expect("tmp");
        let dir = tmp.path();
        fixture(dir);
        let err = handle_work_link("PMAT-1".into(), None, None, Some(dir.to_path_buf()))
            .await
            .expect_err("must refuse");
        assert!(err.to_string().contains("--commit"), "{err}");
    }

    #[tokio::test]
    async fn link_refuses_an_unknown_commit() {
        let tmp = tempfile::tempdir().expect("tmp");
        let dir = tmp.path();
        fixture(dir);
        init_repo(dir);
        let err = handle_work_link(
            "PMAT-1".into(),
            Some("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".into()),
            None,
            Some(dir.to_path_buf()),
        )
        .await
        .expect_err("must refuse");
        assert!(err.to_string().contains("not a commit"), "{err}");
    }
}
