// #1306: refresh-bindings work-contract stub reporting.

/// Work-contract YAMLs written before the #357 schema fix lack a `metadata:` block.
pub(crate) fn count_stale_work_contracts(dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "yaml"))
        .filter_map(|e| fs::read_to_string(e.path()).ok())
        .filter(|c| !c.lines().any(|l| l.starts_with("metadata:")))
        .count()
}

/// One line for `refresh-bindings`: the stub count is never silent.
pub(crate) fn work_contract_report_line(generated: usize, work_dir_exists: bool, stale_before: usize) -> String {
    let mut line = if work_dir_exists {
        format!("{generated} contracts/work/*.yaml file(s) generated")
    } else {
        "0 contracts/work/*.yaml file(s) generated (.pmat-work/ not found)".to_string()
    };
    if stale_before > 0 {
        line.push_str(&format!(
            "; {stale_before} committed work contract(s) predated the current writer (no metadata block)"
        ));
    }
    line
}

#[cfg(test)]
mod work_contract_report_tests {
    use super::*;

    #[test]
    fn zero_stubs_say_why() {
        assert_eq!(
            work_contract_report_line(0, false, 0),
            "0 contracts/work/*.yaml file(s) generated (.pmat-work/ not found)"
        );
    }

    #[test]
    fn stale_files_are_named_in_the_line() {
        let l = work_contract_report_line(3, true, 2);
        assert!(l.starts_with("3 contracts/work/*.yaml file(s) generated"));
        assert!(l.contains("2 committed work contract(s) predated"));
    }

    #[test]
    fn only_files_without_a_metadata_block_are_stale() {
        let t = tempfile::tempdir().unwrap();
        fs::write(t.path().join("a.yaml"), "name: a\nsurface: work-contract\n").unwrap();
        fs::write(t.path().join("b.yaml"), "metadata:\n  kind: schema\nname: b\n").unwrap();
        fs::write(t.path().join("c.txt"), "x").unwrap();
        assert_eq!(count_stale_work_contracts(t.path()), 1);
        assert_eq!(count_stale_work_contracts(&t.path().join("missing")), 0);
    }
}

/// Step 5 of refresh-bindings (R-4): generate contracts/work/*.yaml and say what happened.
fn refresh_work_contracts(project_path: &Path) -> anyhow::Result<()> {
    let stale_before = count_stale_work_contracts(&project_path.join("contracts/work"));
    let yaml_count = generate_work_contract_yamls(project_path)?;
    let line = work_contract_report_line(yaml_count, project_path.join(".pmat-work").exists(), stale_before);
    crate::status_println!("   {}", line);
    Ok(())
}
