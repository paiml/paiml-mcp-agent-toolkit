// #1426: a state change (`pmat work start` / `complete`) rewrites ONE row.
// Included by roadmap_service.rs - shares parent module scope.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod in_place_tests {
    use super::*;
    use crate::models::roadmap::ItemStatus;
    use tempfile::TempDir;

    // Mixed quoting the whole-model serialiser would re-emit differently: quoted
    // plain scalars, a `''` escape, a double-quoted string, a comment, and a
    // key the model does not know.
    const HEADER: &str =
        "# roadmap — hand-edited\nroadmap_version: '1.0'\ngithub_enabled: false\nroadmap:\n";
    const ROW_A: &str = "- id: PMAT-001\n  title: 'First: a quoted title'\n  status: planned\n  priority: high\n  created: '2026-01-01T00:00:00Z'\n  updated: '2026-01-01T00:00:00Z'\n  acceptance_criteria:\n  - 'machines/intel.yaml carries the pin'\n  - 'it''s escaped'\n";
    const ROW_B: &str = "- id: PMAT-002\n  title: \"Second, double-quoted\"\n  status: planned\n  priority: medium\n  created: '2026-01-02T00:00:00Z'\n  updated: '2026-01-02T00:00:00Z'\n  acceptance_criteria:\n  - 'docs/x.md: names the owner'  # a comment\n";

    fn fixture() -> (TempDir, RoadmapService, String) {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("roadmap.yaml");
        let raw = format!("{HEADER}{ROW_A}{ROW_B}");
        std::fs::write(&path, &raw).unwrap();
        (temp, RoadmapService::new(&path), raw)
    }

    fn read(service: &RoadmapService) -> String {
        std::fs::read_to_string(service.path()).unwrap()
    }

    fn with_status(service: &RoadmapService, id: &str, status: ItemStatus) -> RoadmapItem {
        let mut item = service.find_item(id).unwrap().unwrap();
        item.status = status;
        item.updated = "2026-09-24T00:00:00Z".to_string();
        item
    }

    #[test]
    fn start_of_the_first_row_leaves_every_other_row_byte_identical() {
        let (_t, service, _) = fixture();
        let item = with_status(&service, "PMAT-001", ItemStatus::InProgress);
        service.upsert_item_in_place(&item).unwrap();
        let after = read(&service);
        assert!(after.starts_with(HEADER), "header churned:\n{after}");
        assert!(after.ends_with(ROW_B), "an untouched row churned:\n{after}");
        let got = service.find_item("PMAT-001").unwrap().unwrap();
        assert_eq!(got.status, ItemStatus::InProgress);
    }

    #[test]
    fn complete_of_the_last_row_leaves_every_other_row_byte_identical() {
        let (_t, service, _) = fixture();
        let item = with_status(&service, "PMAT-002", ItemStatus::Completed);
        service.upsert_item_in_place(&item).unwrap();
        let after = read(&service);
        assert!(
            after.starts_with(&format!("{HEADER}{ROW_A}")),
            "an untouched row churned:\n{after}"
        );
        let got = service.find_item("PMAT-002").unwrap().unwrap();
        assert_eq!(got.status, ItemStatus::Completed);
    }

    #[test]
    fn a_new_row_is_appended_after_the_untouched_file() {
        let (_t, service, raw) = fixture();
        let mut item = with_status(&service, "PMAT-001", ItemStatus::InProgress);
        item.id = "PMAT-003".to_string();
        service.upsert_item_in_place(&item).unwrap();
        let after = read(&service);
        assert!(
            after.starts_with(&raw),
            "the existing text churned:\n{after}"
        );
        assert!(service.find_item("PMAT-003").unwrap().is_some());
    }

    #[test]
    fn an_unchanged_row_round_trips_to_the_same_bytes_outside_it() {
        let (_t, service, _) = fixture();
        let item = service.find_item("PMAT-002").unwrap().unwrap();
        service.upsert_item_in_place(&item).unwrap();
        assert!(read(&service).starts_with(&format!("{HEADER}{ROW_A}")));
    }

    #[test]
    fn a_duplicated_id_is_refused_and_nothing_is_written() {
        let (_t, service, raw) = fixture();
        let dup = format!("{raw}{ROW_A}");
        std::fs::write(service.path(), &dup).unwrap();
        let mut item: RoadmapItem = serde_yaml_ng::from_str::<Vec<RoadmapItem>>(ROW_A)
            .unwrap()
            .remove(0);
        item.status = ItemStatus::InProgress;
        assert!(service.upsert_item_in_place(&item).is_err());
        assert_eq!(read(&service), dup);
    }

    #[test]
    fn an_empty_roadmap_file_gains_the_row() {
        let temp = TempDir::new().unwrap();
        let service = RoadmapService::new(temp.path().join("roadmap.yaml"));
        let item: RoadmapItem = serde_yaml_ng::from_str::<Vec<RoadmapItem>>(ROW_A)
            .unwrap()
            .remove(0);
        service.upsert_item_in_place(&item).unwrap();
        assert!(service.find_item("PMAT-001").unwrap().is_some());
    }
}
