// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

/// CB-1666: `.agents/mcp_config.json` is a usable MCP client config
/// (PMAT-INIT-002 claim 4).
///
/// Structural half, applied to every server: `mcpServers` is a non-empty
/// object, and each server declares a non-empty `command` plus, when present,
/// an `args` array of strings.
///
/// Known-dead-entrypoint half — a DENYLIST, not a liveness probe. It applies
/// ONLY to servers that start **pmat**, because pmat's entrypoints are the only
/// ones this binary has a source of truth for. That restriction is CB-1656's
/// lesson (#1007): scoring a foreign server against pmat's own facts produced
/// findings no repo could ever satisfy.
///
/// SAY WHAT IT DOES NOT DO, because the gap is load-bearing. This check never
/// executes anything. It recognises the two dead invocations listed below and
/// nothing else, so a pmat entrypoint that is broken in some NEW way still
/// passes. Verified: `{"command":"pmat","args":[]}` passes here while really
/// exiting 2 and printing 8951 bytes of help text to stdout — and that is the
/// exact shape this repository commits at `.kiro/settings/mcp.json`.
///
/// An earlier revision of this comment called it a "liveness half", which it
/// has never been. A doc that claims a stronger check than the code performs is
/// the same defect class this release exists to remove, so it is corrected here
/// rather than quietly narrowed.
///
/// Actually spawning each configured server would close the gap, and is the
/// right eventual design — `pmat init`'s own test suite does exactly that. It is
/// deliberately not done inside a compliance check: `comply check` would then
/// execute arbitrary commands out of a config file it is auditing, which is a
/// worse property than the incompleteness it would fix.
///
/// The two dead pmat entrypoints below are measured against pmat 3.32.0, not
/// inferred:
///
/// ```text
/// $ pmat serve --transport stdio          # 0 bytes on stdout
/// error: invalid value 'stdio' for '--transport <TRANSPORT>'
///   [possible values: http, web-socket, http-sse, both, all]
/// $ cd /tmp && cargo run --bin pmat -- --version
/// error: could not find `Cargo.toml` in `/tmp` or any parent directory
/// $ pmat --mode mcp   # <- works: initialize returns a JSON-RPC result
/// {"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05", ...}}
/// ```
pub(crate) fn check_agy_mcp_config(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1666: AGY MCP Client Config";
    let agents_dir = project_path.join(AGY_DIR);
    if !agents_dir.is_dir() {
        return skip_check(
            name,
            "no .agents/ directory in this project — 0 MCP servers judged",
        );
    }
    let config_path = agents_dir.join("mcp_config.json");
    if !config_path.is_file() {
        return skip_check(
            name,
            ".agents/ is present but has no mcp_config.json — 0 MCP servers judged \
             (this project wires up no AGY MCP server)",
        );
    }
    let Ok(text) = std::fs::read_to_string(&config_path) else {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: ".agents/mcp_config.json is unreadable".to_string(),
            severity: Severity::Error,
        };
    };
    let value: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            return ComplianceCheck {
                name: name.to_string(),
                status: CheckStatus::Fail,
                message: format!(
                    ".agents/mcp_config.json is not valid JSON (line {} column {}): {e}",
                    e.line(),
                    e.column()
                ),
                severity: Severity::Error,
            }
        }
    };

    let Some(servers) = value.get("mcpServers").and_then(|s| s.as_object()) else {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: ".agents/mcp_config.json has no `mcpServers` object — \
                      no client would start anything from it"
                .to_string(),
            severity: Severity::Error,
        };
    };
    if servers.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: ".agents/mcp_config.json declares 0 MCP servers under `mcpServers` \
                      — the file is inert"
                .to_string(),
            severity: Severity::Error,
        };
    }

    let mut hard: Vec<String> = Vec::new();
    let mut soft: Vec<String> = Vec::new();
    for (server, spec) in servers {
        agy_judge_mcp_server(server, spec, &mut hard, &mut soft);
    }

    let summary = format!(
        "judged {} MCP server entr(ies) in .agents/mcp_config.json",
        servers.len()
    );
    agy_verdict(name, &summary, &hard, &soft)
}

fn agy_judge_mcp_server(
    server: &str,
    spec: &serde_json::Value,
    hard: &mut Vec<String>,
    soft: &mut Vec<String>,
) {
    let command = match spec.get("command").and_then(|c| c.as_str()) {
        Some(c) if !c.trim().is_empty() => c,
        Some(_) => {
            hard.push(format!("mcpServers.{server}: `command` is empty"));
            return;
        }
        None => {
            hard.push(format!(
                "mcpServers.{server}: missing `command` (string) — nothing to launch"
            ));
            return;
        }
    };
    let args = match spec.get("args") {
        None => {
            soft.push(format!(
                "mcpServers.{server}: no `args` key; the entrypoint is `{command}` with no \
                 arguments, which is only correct if that binary speaks MCP on stdio by default"
            ));
            Vec::new()
        }
        Some(serde_json::Value::Array(items)) => {
            let strings: Vec<String> = items
                .iter()
                .filter_map(|a| a.as_str().map(str::to_string))
                .collect();
            if strings.len() != items.len() {
                hard.push(format!(
                    "mcpServers.{server}: `args` contains a non-string element"
                ));
            }
            strings
        }
        Some(_) => {
            hard.push(format!("mcpServers.{server}: `args` must be an array"));
            Vec::new()
        }
    };
    agy_judge_pmat_entrypoint(server, command, &args, hard);
}

/// Liveness rules for pmat's own entrypoints. Silent no-op on any other server.
fn agy_judge_pmat_entrypoint(
    server: &str,
    command: &str,
    args: &[String],
    hard: &mut Vec<String>,
) {
    let base = command.rsplit('/').next().unwrap_or(command);
    let via_cargo = base == "cargo" && args.iter().any(|a| a == "pmat");
    if via_cargo {
        hard.push(format!(
            "mcpServers.{server}: starts pmat through `cargo run`, which needs a Cargo.toml \
             checkout — in any other directory it exits with \
             \"could not find `Cargo.toml`\", and where it does work it rebuilds the crate on \
             every server start. Use command `pmat` with args [\"--mode\", \"mcp\"]"
        ));
    }
    if base != "pmat" && !via_cargo {
        return; // not pmat: this binary has no source of truth for it
    }
    if args.iter().any(|a| a == "stdio") && args.iter().any(|a| a == "--transport") {
        hard.push(format!(
            "mcpServers.{server}: `--transport stdio` is rejected by pmat's own argument parser \
             (possible values: http, web-socket, http-sse, both, all); the process exits at parse \
             time and writes 0 bytes of MCP. Use command `pmat` with args [\"--mode\", \"mcp\"]"
        ));
        return;
    }
    let speaks_mcp = args.windows(2).any(|w| w[0] == "--mode" && w[1] == "mcp");
    if args.iter().any(|a| a == "serve") && !speaks_mcp {
        hard.push(format!(
            "mcpServers.{server}: `pmat serve` is the HTTP/WebSocket server and its own help says \
             [NOT IMPLEMENTED]; bare `pmat serve` refuses to start without PMAT_MCP_HTTP_TOKEN and \
             never speaks MCP on stdio. Use command `pmat` with args [\"--mode\", \"mcp\"]"
        ));
    }
}
