//! Component 27: Work Contract Binding resolver.
//!
//! Resolves `<contract>/<equation>` tokens into `ContractBinding` records
//! by locating the YAML file, computing SHA-256, and recording bind time.
//!
//! Sub-spec: `docs/specifications/components/pmat-work-contract-binding.md`.

use anyhow::{anyhow, Context, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use crate::cli::handlers::work_contract::ContractBinding;

/// Resolve a `<contract>/<equation>` token against the project and environment.
///
/// Search order (matches spec §CLI Surface):
/// 1. `<project_path>/contracts/<contract>.yaml`
/// 2. `<project_path>/../provable-contracts/contracts/<contract>.yaml`
/// 3. Each entry in `$PMAT_CONTRACTS_PATH` (colon-separated)
///
/// Returns a `ContractBinding` with SHA-256 of the file bytes; does not
/// parse the YAML or validate that the `equation` key exists — that
/// responsibility belongs to CB-1607 at check time.
pub fn resolve_binding(project_path: &Path, token: &str) -> Result<ContractBinding> {
    let (contract, equation) = ContractBinding::parse_token(token).ok_or_else(|| {
        anyhow!(
            "invalid --implements token '{}': expected <contract>/<equation>",
            token
        )
    })?;

    let file = locate_contract_file(project_path, &contract)
        .with_context(|| format!("contract '{}' not found in search path", contract))?;

    let bytes =
        std::fs::read(&file).with_context(|| format!("failed to read {}", file.display()))?;
    let sha = hex_digest(&bytes);
    require_equation(&file, &bytes, &equation)?;

    Ok(ContractBinding {
        contract,
        equation,
        file,
        sha,
        bound_at: chrono::Utc::now(),
    })
}

/// A binding names an equation, so the equation has to exist in the contract it
/// names (#1186). Before this check `--implements fx-v1/no_such_equation` bound
/// happily: the sha was of the file, and nothing ever looked inside it.
fn require_equation(file: &Path, bytes: &[u8], equation: &str) -> Result<()> {
    let doc: serde_yaml_ng::Value = serde_yaml_ng::from_slice(bytes)
        .with_context(|| format!("{} is not valid YAML", file.display()))?;
    let equations = doc
        .get("equations")
        .and_then(|e| e.as_mapping())
        .ok_or_else(|| anyhow!("{} declares no `equations:` mapping", file.display()))?;
    if equations.contains_key(equation) {
        return Ok(());
    }
    let mut known: Vec<String> = equations
        .keys()
        .filter_map(|k| k.as_str().map(str::to_string))
        .collect();
    known.sort();
    Err(anyhow!(
        "equation '{}' is not declared in {} (declared: {})",
        equation,
        file.display(),
        if known.is_empty() {
            "none".to_string()
        } else {
            known.join(", ")
        }
    ))
}

fn locate_contract_file(project_path: &Path, contract: &str) -> Result<PathBuf> {
    let candidates = candidate_paths(project_path, contract);
    for candidate in &candidates {
        if candidate.exists() {
            return Ok(candidate.clone());
        }
    }
    Err(anyhow!(
        "no contract YAML found. Searched:\n  - {}",
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join("\n  - ")
    ))
}

fn candidate_paths(project_path: &Path, contract: &str) -> Vec<PathBuf> {
    let filename = format!("{}.yaml", contract);
    let mut out = vec![project_path.join("contracts").join(&filename)];

    if let Some(parent) = project_path.parent() {
        out.push(
            parent
                .join("provable-contracts")
                .join("contracts")
                .join(&filename),
        );
    }

    if let Ok(env_path) = std::env::var("PMAT_CONTRACTS_PATH") {
        for dir in env_path.split(':').filter(|s| !s.is_empty()) {
            out.push(PathBuf::from(dir).join(&filename));
        }
    }

    out
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut s = String::with_capacity(digest.len() * 2);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

/// Resolve many tokens at once. Accumulates errors across tokens rather than
/// failing on the first malformed entry — a user running
/// `--implements a/b --implements c/d` gets a complete report.
pub fn resolve_all(project_path: &Path, tokens: &[String]) -> Result<Vec<ContractBinding>> {
    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    let mut bindings = Vec::with_capacity(tokens.len());
    let mut errors = Vec::new();
    for token in tokens {
        match resolve_binding(project_path, token) {
            Ok(b) => bindings.push(b),
            Err(e) => errors.push(format!("  --implements {}: {}", token, e)),
        }
    }
    if !errors.is_empty() {
        anyhow::bail!(
            "failed to resolve {} binding(s):\n{}",
            errors.len(),
            errors.join("\n")
        );
    }

    // CB-1623 dedup: multiple --implements flags must not repeat the same key
    let mut seen = std::collections::HashSet::new();
    for b in &bindings {
        let k = b.key();
        if !seen.insert(k.clone()) {
            anyhow::bail!("duplicate --implements target '{}'", k);
        }
    }

    Ok(bindings)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_yaml(dir: &Path, contract: &str, body: &str) -> PathBuf {
        let contracts_dir = dir.join("contracts");
        std::fs::create_dir_all(&contracts_dir).unwrap();
        let path = contracts_dir.join(format!("{}.yaml", contract));
        std::fs::write(&path, body).unwrap();
        path
    }

    #[test]
    fn resolve_finds_contract_in_project_root() {
        let temp = tempdir().unwrap();
        write_yaml(temp.path(), "rope-kernel-v1", "equations:\n  rope: {}\n");
        let binding = resolve_binding(temp.path(), "rope-kernel-v1/rope").unwrap();
        assert_eq!(binding.contract, "rope-kernel-v1");
        assert_eq!(binding.equation, "rope");
        assert_eq!(binding.sha.len(), 64); // sha256 hex
    }

    #[test]
    fn resolve_computes_stable_sha() {
        let temp = tempdir().unwrap();
        write_yaml(temp.path(), "k", "equations:\n  eq: {body: 1}\n");
        let a = resolve_binding(temp.path(), "k/eq").unwrap();
        let b = resolve_binding(temp.path(), "k/eq").unwrap();
        assert_eq!(a.sha, b.sha);
    }

    #[test]
    fn resolve_sha_changes_when_bytes_change() {
        let temp = tempdir().unwrap();
        write_yaml(temp.path(), "k", "equations:\n  eq: {body: 1}\n");
        let a = resolve_binding(temp.path(), "k/eq").unwrap();
        write_yaml(temp.path(), "k", "equations:\n  eq: {body: 2}\n");
        let b = resolve_binding(temp.path(), "k/eq").unwrap();
        assert_ne!(a.sha, b.sha, "SHA must reflect file content changes");
    }

    #[test]
    fn resolve_errors_on_missing_file() {
        let temp = tempdir().unwrap();
        let err = resolve_binding(temp.path(), "nope/eq").unwrap_err();
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("nope.yaml"),
            "error should list the missing file, got: {}",
            msg
        );
    }

    #[test]
    fn resolve_errors_on_malformed_token() {
        let temp = tempdir().unwrap();
        let err = resolve_binding(temp.path(), "no-slash-token").unwrap_err();
        assert!(format!("{:#}", err).contains("invalid --implements token"));
    }

    #[test]
    fn resolve_refuses_an_equation_the_contract_does_not_declare() {
        let temp = tempdir().expect("tempdir");
        write_yaml(temp.path(), "fx-v1", "equations:\n  one_is_one: {}\n");
        let err = resolve_binding(temp.path(), "fx-v1/no_such_equation").unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("no_such_equation") && msg.contains("one_is_one"),
            "{msg}"
        );
        assert!(resolve_binding(temp.path(), "fx-v1/one_is_one").is_ok());
    }

    #[test]
    fn resolve_all_empty_is_ok() {
        let temp = tempdir().unwrap();
        let out = resolve_all(temp.path(), &[]).unwrap();
        assert!(out.is_empty());
    }

    #[test]
    fn resolve_all_detects_duplicates() {
        let temp = tempdir().unwrap();
        write_yaml(temp.path(), "k", "equations:\n  eq: {}\n");
        let err = resolve_all(temp.path(), &["k/eq".to_string(), "k/eq".to_string()]).unwrap_err();
        assert!(format!("{:#}", err).contains("duplicate"));
    }

    #[test]
    fn resolve_all_aggregates_errors() {
        let temp = tempdir().unwrap();
        // Neither file exists — expect a multi-line error mentioning both
        let err = resolve_all(temp.path(), &["a/eq".to_string(), "b/eq".to_string()]).unwrap_err();
        let msg = format!("{:#}", err);
        assert!(msg.contains("a/eq"));
        assert!(msg.contains("b/eq"));
    }

    #[test]
    fn resolve_honors_pmat_contracts_path() {
        let temp = tempdir().unwrap();
        let ext_dir = tempdir().unwrap();
        let yaml_path = ext_dir.path().join("external.yaml");
        std::fs::write(&yaml_path, "equations:\n  eq: {}\n").unwrap();

        // Set env var to point at ext_dir
        // SAFETY: test runs serially for env modification
        let prev = std::env::var("PMAT_CONTRACTS_PATH").ok();
        unsafe {
            std::env::set_var("PMAT_CONTRACTS_PATH", ext_dir.path());
        }
        let result = resolve_binding(temp.path(), "external/eq");
        // Restore
        unsafe {
            match prev {
                Some(v) => std::env::set_var("PMAT_CONTRACTS_PATH", v),
                None => std::env::remove_var("PMAT_CONTRACTS_PATH"),
            }
        }
        let binding = result.unwrap();
        assert_eq!(binding.file, yaml_path);
    }
}
