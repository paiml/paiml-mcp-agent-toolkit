// CB-1646 — CoT derivation SHA fresh. Detects manual edits that bypass
// `pmat work cot derive` by recomputing the canonical SHA-256 of each
// ticket's `chain_of_thought` array and comparing to the recorded digest.
// Included from `check_cot_proof.rs` — do NOT add `use` imports or `#!` attributes here.

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let d = h.finalize();
    let mut s = String::with_capacity(d.len() * 2);
    for b in d {
        use std::fmt::Write;
        let _ = write!(&mut s, "{:02x}", b);
    }
    s
}

/// Emit `v` into `out` as canonical JSON: object keys sorted lexicographically,
/// no whitespace, arrays preserve order. Downstream deps enable
/// `serde_json/preserve_order` (IndexMap), so raw `to_vec` would hash
/// differently depending on which author typed which key first — this
/// walker erases that non-determinism. Any RFC 8785-compatible producer
/// will agree on the bytes.
fn canonicalize(v: &Value, out: &mut String) {
    use std::fmt::Write;
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            let _ = write!(out, "{}", n);
        }
        Value::String(s) => {
            // serde_json handles escape rules; we just borrow the String emitter.
            if let Ok(escaped) = serde_json::to_string(s) {
                out.push_str(&escaped);
            }
        }
        Value::Array(arr) => {
            out.push('[');
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                canonicalize(item, out);
            }
            out.push(']');
        }
        Value::Object(obj) => {
            let mut keys: Vec<&String> = obj.keys().collect();
            keys.sort();
            out.push('{');
            for (i, k) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if let Ok(escaped) = serde_json::to_string(k) {
                    out.push_str(&escaped);
                }
                out.push(':');
                canonicalize(&obj[*k], out);
            }
            out.push('}');
        }
    }
}

/// Canonical SHA-256 of a contract's `chain_of_thought` array.
fn canonical_cot_sha(contract: &Value) -> String {
    let cot = contract
        .get("chain_of_thought")
        .cloned()
        .unwrap_or(Value::Null);
    let mut buf = String::new();
    canonicalize(&cot, &mut buf);
    sha256_hex(buf.as_bytes())
}

/// Read a recorded CoT digest from `cot-digest.json`. Accepts either a
/// `{"sha": "..."}` or `{"digest": "..."}` shape — both naming conventions
/// are plausible for the forthcoming `pmat work cot derive` output.
fn read_recorded_digest(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let v: Value = serde_json::from_slice(&bytes).ok()?;
    v.get("sha")
        .or_else(|| v.get("digest"))
        .and_then(|s| s.as_str())
        .map(|s| s.to_string())
}

/// CB-1646 (L1): recomputes the canonical SHA of each ticket's
/// `chain_of_thought` and compares it to the digest recorded in
/// `.pmat-work/<ID>/cot-digest.json`. A mismatch means the CoT was edited
/// by hand after `pmat work cot derive` last ran, which defeats the
/// derivation pipeline's witness trail. Skip-if-absent: tickets without a
/// digest file are ignored, and the check skips overall when no ticket
/// has one (the digest emitter hasn't run yet).
pub(crate) fn check_cot_derivation_sha_fresh(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1646: CoT Derivation SHA";
    let contracts = load_contract_values(project_path);

    let mut mismatches: Vec<String> = Vec::new();
    let mut malformed: Vec<String> = Vec::new();
    let mut checked = 0usize;

    for (ticket, contract) in &contracts {
        let digest_path = project_path
            .join(".pmat-work")
            .join(ticket)
            .join("cot-digest.json");
        if !digest_path.exists() {
            continue;
        }
        let Some(recorded) = read_recorded_digest(&digest_path) else {
            malformed.push(ticket.clone());
            continue;
        };
        checked += 1;
        let actual = canonical_cot_sha(contract);
        if !recorded.eq_ignore_ascii_case(&actual) {
            mismatches.push(format!(
                "{}: recorded={}, actual={}",
                ticket, recorded, actual
            ));
        }
    }

    if checked == 0 && malformed.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Skip,
            message: "No `.pmat-work/<ID>/cot-digest.json` files — `pmat work cot derive` hasn't emitted digests yet".into(),
            severity: Severity::Info,
        };
    }

    if mismatches.is_empty() && malformed.is_empty() {
        return ComplianceCheck {
            name: name.into(),
            status: CheckStatus::Pass,
            message: format!(
                "{} ticket(s): cot-digest.json matches canonical SHA",
                checked
            ),
            severity: Severity::Info,
        };
    }

    let mut msg = String::new();
    if !mismatches.is_empty() {
        msg.push_str(&format!(
            "{} ticket(s) with CoT drift — `pmat work cot derive` to refresh:\n  {}\n",
            mismatches.len(),
            mismatches.join("\n  ")
        ));
    }
    if !malformed.is_empty() {
        msg.push_str(&format!(
            "{} ticket(s) with unreadable cot-digest.json: {}",
            malformed.len(),
            malformed.join(", ")
        ));
    }
    ComplianceCheck {
        name: name.into(),
        status: CheckStatus::Fail,
        message: msg,
        severity: Severity::Error,
    }
}
