// Tests for `deserialize_phases` in roadmap_types.rs — isolated into its own
// file so `roadmap_tests.rs` stays under the 500-line pre-commit gate.
//
// Included by roadmap.rs — shares parent scope, no `use` imports needed.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod deserialize_phases_tests {
    use super::*;

    fn parse_item(yaml: &str) -> Result<RoadmapItem, serde_yaml_ng::Error> {
        serde_yaml_ng::from_str::<RoadmapItem>(yaml)
    }

    #[test]
    fn test_phases_empty_sequence() {
        let yaml = "\
id: T-1
title: t
status: planned
phases: []
";
        let item = parse_item(yaml).unwrap();
        assert!(item.phases.is_empty());
    }

    #[test]
    fn test_phases_omitted_defaults_empty() {
        let yaml = "\
id: T-1
title: t
status: planned
";
        let item = parse_item(yaml).unwrap();
        assert!(item.phases.is_empty());
    }

    #[test]
    fn test_phases_mapping_parsed() {
        let yaml = "\
id: T-1
title: t
status: planned
phases:
  - name: Phase A
    status: planned
  - name: Phase B
    status: in_progress
    estimated_effort: 2d
    completion: 50
";
        let item = parse_item(yaml).unwrap();
        assert_eq!(item.phases.len(), 2);
        assert_eq!(item.phases[0].name, "Phase A");
        assert_eq!(item.phases[0].status, ItemStatus::Planned);
        assert_eq!(item.phases[0].completion, 0);
        assert_eq!(item.phases[1].name, "Phase B");
        assert_eq!(item.phases[1].estimated_effort.as_deref(), Some("2d"));
        assert_eq!(item.phases[1].completion, 50);
    }

    #[test]
    fn test_phases_string_rejected_with_helpful_message() {
        let yaml = "\
id: T-1
title: t
status: planned
phases:
  - Plan it
";
        let err = parse_item(yaml).unwrap_err().to_string();
        assert!(err.contains("phases[0]"), "got: {err}");
        assert!(err.contains("invalid type"), "got: {err}");
        assert!(err.contains("Plan it"), "got: {err}");
        assert!(err.contains("name:"), "got: {err}");
    }

    #[test]
    fn test_phases_string_error_reports_index() {
        let yaml = "\
id: T-1
title: t
status: planned
phases:
  - name: ok
    status: planned
  - bad-string
";
        let err = parse_item(yaml).unwrap_err().to_string();
        assert!(err.contains("phases[1]"), "got: {err}");
        assert!(err.contains("bad-string"), "got: {err}");
    }

    #[test]
    fn test_phases_non_mapping_non_string_rejected() {
        let yaml = "\
id: T-1
title: t
status: planned
phases:
  - 42
";
        let err = parse_item(yaml).unwrap_err().to_string();
        assert!(err.contains("phases[0]"), "got: {err}");
        assert!(err.contains("expected a Phase struct"), "got: {err}");
    }

    #[test]
    fn test_phases_invalid_mapping_surfaces_inner_error() {
        let yaml = "\
id: T-1
title: t
status: planned
phases:
  - name: ok
";
        let err = parse_item(yaml).unwrap_err().to_string();
        assert!(
            err.contains("status") || err.contains("missing"),
            "got: {err}"
        );
    }

    /// roadmap_types.rs:216 — `PhasesVisitor::expecting` fires when the YAML
    /// supplies a non-sequence value (string, mapping, scalar) where a
    /// `Vec<Phase>` is expected. Serde's default visit_* impls call
    /// expecting() to build the error message.
    #[test]
    fn test_phases_non_sequence_surfaces_expecting_message() {
        // A bare string is neither a sequence nor null — default visit_str
        // generates an Unexpected error that embeds expecting()'s output.
        let yaml = "\
id: T-1
title: t
status: planned
phases: \"not a list\"
";
        let err = parse_item(yaml).unwrap_err().to_string();
        assert!(
            err.contains("a sequence of Phase structs"),
            "expecting() message must appear in error: {err}"
        );
    }

    /// Mapping at the top-level `phases:` key (not a sequence) — also a
    /// non-sequence input, triggers the expecting message through
    /// visit_map's default impl.
    #[test]
    fn test_phases_mapping_instead_of_sequence_surfaces_expecting() {
        let yaml = "\
id: T-1
title: t
status: planned
phases:
  name: not-a-list
  status: planned
";
        let err = parse_item(yaml).unwrap_err().to_string();
        assert!(
            err.contains("a sequence of Phase structs"),
            "expecting() message must appear: {err}"
        );
    }
}
