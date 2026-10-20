// infra#1970: `work start` / `work complete` write ONE row, not the whole file.
// Included by roadmap_service.rs - shares parent module scope.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod upsert_raw_tests {
    use super::*;
    use crate::models::roadmap::ItemStatus;
    use tempfile::TempDir;

    /// The bytes a hand-maintained roadmap carries and the serde model does
    /// not round-trip: a comment, an explicit `epic: null`, and a quoted
    /// string. Measured in paiml/infra: one `work start` deleted two
    /// `epic: null` lines and re-quoted three strings in rows nobody touched.
    const FIXTURE: &str = "\
roadmap_version: '1.0'
github_enabled: true
github_repo: paiml/infra
roadmap:
- id: PMAT-001
  github_issue: null
  item_type: task
  title: 'first: a title YAML needs quoted'
  status: planned
  priority: medium
  assigned_to: null
  created: 2026-10-01T00:00:00Z
  updated: 2026-10-01T00:00:00Z
  spec: null
  acceptance_criteria:
  - \"a criterion in double quotes\"
  phases: []
  subtasks: []
  estimated_effort: null
  labels: []
  notes: null
  epic: null
# a comment between rows, which no model carries
- id: PMAT-002
  github_issue: null
  item_type: task
  title: the ticket being started
  status: planned
  priority: high
  assigned_to: null
  created: 2026-10-01T00:00:00Z
  updated: 2026-10-01T00:00:00Z
  spec: null
  acceptance_criteria: []
  phases: []
  subtasks: []
  estimated_effort: null
  labels: []
  notes: null
  epic: null
- id: PMAT-003
  github_issue: null
  item_type: task
  title: 'third: also quoted'
  status: completed
  priority: low
  assigned_to: null
  created: 2026-10-01T00:00:00Z
  updated: 2026-10-01T00:00:00Z
  spec: null
  acceptance_criteria: []
  phases: []
  subtasks: []
  estimated_effort: null
  labels: []
  notes: null
  epic: null
";

    fn service_with(text: &str) -> (TempDir, RoadmapService, std::path::PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("roadmap.yaml");
        std::fs::write(&path, text).unwrap();
        let service = RoadmapService::new(&path);
        (dir, service, path)
    }

    /// The row text for `id`, from `- id: <id>` up to the next row or comment.
    fn row(text: &str, id: &str) -> String {
        let start = text.find(&format!("- id: {id}\n")).expect("row present");
        let rest = &text[start + 1..];
        let end = rest
            .find("\n- id: ")
            .or_else(|| rest.find("\n#"))
            .map_or(text.len(), |i| start + 1 + i + 1);
        text[start..end].to_string()
    }

    fn started(service: &RoadmapService, id: &str) -> RoadmapItem {
        let mut item = service
            .load()
            .unwrap()
            .roadmap
            .into_iter()
            .find(|i| i.id == id)
            .expect("fixture row");
        item.status = ItemStatus::InProgress;
        item
    }

    #[test]
    fn starting_one_ticket_leaves_every_other_row_byte_for_byte() {
        let (_d, service, path) = service_with(FIXTURE);
        service.upsert_item_raw(&started(&service, "PMAT-002")).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();

        assert_eq!(row(&after, "PMAT-001"), row(FIXTURE, "PMAT-001"));
        assert_eq!(row(&after, "PMAT-003"), row(FIXTURE, "PMAT-003"));
        assert!(after.contains("# a comment between rows, which no model carries\n"));
        assert_eq!(after.matches("\n  epic: null\n").count(), 3, "{after}");
        assert!(row(&after, "PMAT-002").contains("status: inprogress"));
        assert_eq!(
            service.load().unwrap().roadmap.len(),
            3,
            "the edited file must still parse to the same three rows"
        );
    }

    /// The fixture exercises the defect: the whole-model save that `work
    /// start` used rewrites rows it was not asked about. Without this, the
    /// test above could pass on a fixture the old path also left alone.
    #[test]
    fn the_whole_model_save_rewrites_rows_it_was_not_asked_about() {
        let (_d, service, path) = service_with(FIXTURE);
        let mut roadmap = service.load().unwrap();
        roadmap.upsert_item(started(&service, "PMAT-002"));
        service.save(&roadmap).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert!(
            !after.contains("# a comment between rows")
                || row(&after, "PMAT-001") != row(FIXTURE, "PMAT-001"),
            "the old path no longer damages this fixture, so it proves nothing:\n{after}"
        );
    }

    #[test]
    fn a_ticket_with_no_row_is_appended_after_the_existing_bytes() {
        let (_d, service, path) = service_with(FIXTURE);
        let mut item = started(&service, "PMAT-002");
        item.id = "PMAT-004".to_string();
        item.title = "a new ticket".to_string();
        service.upsert_item_raw(&item).unwrap();
        let after = std::fs::read_to_string(&path).unwrap();
        assert!(after.starts_with(FIXTURE), "existing bytes moved:\n{after}");
        assert_eq!(service.load().unwrap().roadmap.len(), 4);
    }

    #[test]
    fn an_absent_roadmap_gets_a_valid_file_with_the_one_row() {
        let dir = TempDir::new().unwrap();
        let service = RoadmapService::new(dir.path().join("roadmap.yaml"));
        let item = RoadmapItem::from_github_issue(7, "only row".to_string());
        service.upsert_item_raw(&item).unwrap();
        let loaded = service.load().unwrap();
        assert_eq!(loaded.roadmap.len(), 1);
        assert_eq!(loaded.roadmap[0].id, "GH-7");
    }
}
