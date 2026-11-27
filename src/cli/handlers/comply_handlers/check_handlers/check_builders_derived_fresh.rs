// CB-2120 builder. Included from check.rs — no `use` imports here.

/// CB-2120: is every declared committed derived file fresh? `selected` is the
/// `--checks` list: generators run only when it names the rule.
fn build_derived_fresh_checks(
    project_path: &Path,
    comply_config: &crate::models::comply_config::ComplyConfig,
    selected: &[String],
) -> Vec<ComplianceCheck> {
    // A disabled rule runs no generator: an empty selection returns Skip before any
    // runs, and filter_check_by_config then reports the disable.
    let selected: &[String] = if comply_config.is_check_enabled("cb-2120") { selected } else { &[] };
    vec![filter_check_by_config(
        check_derived_fresh(project_path, selected),
        "cb-2120",
        comply_config,
    )]
}
