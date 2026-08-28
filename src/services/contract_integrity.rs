//! Every contract citation must resolve, or it asserts nothing.
//!
//! # The defect this closes
//!
//! `#[provable_contracts_macros::contract("pmat-core.yaml", equation = "X")]`
//! takes two strings and checks neither. `build.rs` emits `CONTRACT_*=bound`
//! from the binding's `status: implemented` without ever opening the contract
//! file. So an annotation naming an equation that does not exist compiles, ships,
//! and reads to every later maintainer as a discharged proof obligation.
//!
//! Measured at the commit that added this module: **1,187 annotation sites in
//! `src/` cited an equation `pmat-core.yaml` did not declare** —
//! `path_exists` (1,174), `non_empty_index` (10), `lint_valid` (3) — plus 14 of
//! the 59 entries in `contracts/binding.yaml`. `pmat-core.yaml` declared eleven
//! equations and none of those three was among them.
//!
//! They were fixed by DECLARING the equations, not by deleting the annotations:
//! the functions carrying `path_exists` really do require the path to exist and
//! several assert it at runtime. The citations were right; the contract was
//! incomplete.
//!
//! # Why a test and not a lint
//!
//! No general linter can know that `"pmat-core.yaml"` names a file in
//! `contracts/` or that `"path_exists"` must appear in its `equations:` map.
//! That is project-specific by construction, which is exactly where a checked
//! invariant earns its keep over a lint.
//!
//! And it must be a `--lib` test: merge CI runs `cargo test --lib`, so a check
//! living in `tests/` would not execute at the gate that decides a merge.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    /// Equation names declared by a contract, or `None` when the file is absent.
    fn declared_equations(contract: &str) -> Option<BTreeSet<String>> {
        let path = repo_root().join("contracts").join(contract);
        let text = std::fs::read_to_string(path).ok()?;
        let doc: serde_yaml_ng::Value = serde_yaml_ng::from_str(&text).ok()?;
        let eqs = doc.get("equations")?;
        let mut out = BTreeSet::new();
        if let Some(map) = eqs.as_mapping() {
            for k in map.keys() {
                if let Some(s) = k.as_str() {
                    out.insert(s.to_string());
                }
            }
        }
        Some(out)
    }

    /// Every `(file, line, contract, equation)` cited from real code.
    ///
    /// Doc comments are skipped: this module's own header quotes the annotation,
    /// and several handlers document the macro's usage with placeholder names
    /// like `"yaml-name"`. A citation inside `///` is prose, not a binding.
    fn citations() -> Vec<(PathBuf, usize, String, String)> {
        fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "rs") {
                    out.push(p);
                }
            }
        }
        let mut files = Vec::new();
        walk(&repo_root().join("src"), &mut files);

        let mut found = Vec::new();
        for file in files {
            let Ok(text) = std::fs::read_to_string(&file) else {
                continue;
            };
            for (i, raw) in text.lines().enumerate() {
                let line = raw.trim_start();
                if line.starts_with("//") {
                    continue; // prose, including this module's own header
                }
                let Some(rest) = line.split_once("contract(\"") else {
                    continue;
                };
                let Some((contract, tail)) = rest.1.split_once('"') else {
                    continue;
                };
                let Some(eq_part) = tail.split_once("equation = \"") else {
                    continue;
                };
                let Some((equation, _)) = eq_part.1.split_once('"') else {
                    continue;
                };
                found.push((
                    file.clone(),
                    i + 1,
                    contract.to_string(),
                    equation.to_string(),
                ));
            }
        }
        found
    }

    /// Citations this repository deliberately makes to names that do not exist.
    ///
    /// Both are self-referential test fixtures: code that exercises the contract
    /// MACHINERY needs a contract name, and inventing a real one would couple a
    /// unit test to a production contract. Each entry is a claim someone has to
    /// defend — it is not a place to park a real phantom.
    const FIXTURE_CITATIONS: &[(&str, &str)] = &[
        // Exercises the annotation counter itself; the names are the example
        // from the macro's own documentation.
        ("softmax-v1", "softmax"),
        ("relu-v1", "relu"),
        // The inline comply test asserts on a contract that is deliberately
        // absent, to prove an absent contract is reported rather than ignored.
        ("demo.yaml", "my_fn"),
    ];

    fn is_fixture(contract: &str, equation: &str) -> bool {
        FIXTURE_CITATIONS
            .iter()
            .any(|(c, e)| *c == contract && *e == equation)
    }

    /// Every equation cited from code is declared by the contract it names.
    ///
    /// RED at the commit before this module: 1,187 sites across `src/` cited
    /// `pmat-core.yaml` equations that did not exist.
    #[test]
    fn every_cited_equation_is_declared() {
        let mut phantom: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
        let mut checked = 0usize;

        for (file, line, contract, equation) in citations() {
            if is_fixture(&contract, &equation) {
                continue;
            }
            checked += 1;
            let ok = declared_equations(&contract).is_some_and(|e| e.contains(&equation));
            if !ok {
                let rel = file
                    .strip_prefix(repo_root())
                    .unwrap_or(&file)
                    .display()
                    .to_string();
                phantom
                    .entry((contract.clone(), equation.clone()))
                    .or_default()
                    .push(format!("{rel}:{line}"));
            }
        }

        // A checker that found nothing to check is not a passing checker. This
        // repository has thousands of these annotations; if the walk returns a
        // handful, the walk is broken and its silence means nothing.
        assert!(
            checked > 1000,
            "only {checked} citations found — the source walk is broken, and a \
             checker that measured nothing must not report success"
        );

        assert!(
            phantom.is_empty(),
            "these contract citations name an equation the contract does not declare, \
             so they assert NOTHING while reading as a discharged proof obligation:\n{}\n\
             Either declare the equation in contracts/<file>.yaml, or remove the \
             annotation. Do not add it to FIXTURE_CITATIONS unless it is genuinely \
             exercising the contract machinery itself.",
            phantom
                .iter()
                .map(|((c, e), sites)| format!(
                    "  {c} / {e}  — {} site(s), e.g. {}",
                    sites.len(),
                    sites.first().map_or("?", String::as_str)
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    /// Every binding in the registry resolves to a declared equation.
    ///
    /// `build.rs` emits `CONTRACT_*=bound` from a binding's `status` alone,
    /// without opening the contract — so a binding naming a missing equation
    /// reports as bound. 14 of 59 did.
    #[test]
    fn every_binding_resolves() {
        let path = repo_root().join("contracts/binding.yaml");
        let text = std::fs::read_to_string(&path).expect("contracts/binding.yaml must exist");
        let doc: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(&text).expect("binding.yaml is YAML");

        let rows = doc
            .get("bindings")
            .and_then(|b| b.as_sequence())
            .expect("binding.yaml declares bindings[]");

        let mut dangling = Vec::new();
        for row in rows {
            let (Some(contract), Some(equation)) = (
                row.get("contract").and_then(|v| v.as_str()),
                row.get("equation").and_then(|v| v.as_str()),
            ) else {
                continue;
            };
            if !declared_equations(contract).is_some_and(|e| e.contains(equation)) {
                let func = row
                    .get("function")
                    .and_then(|v| v.as_str())
                    .unwrap_or("<unnamed>");
                dangling.push(format!("  {func}: {contract} / {equation}"));
            }
        }

        assert!(
            !rows.is_empty(),
            "binding.yaml declared no bindings — an empty registry cannot fail, \
             which is not the same as a registry that passed"
        );
        assert!(
            dangling.is_empty(),
            "these bindings name an equation their contract does not declare, and \
             build.rs reports them as bound anyway:\n{}",
            dangling.join("\n")
        );
    }
}
