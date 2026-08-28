// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

/// One `command` string pulled out of a `hooks.json` rule, with where it came
/// from, so a violation can name the exact entry rather than "a hook".
struct AgyHookCommand {
    /// e.g. `pmat-quality-feedback.PreToolUse[0].hooks[0]`
    origin: String,
    command: String,
}

/// CB-1664: `.agents/hooks.json` PreToolUse correctness (PMAT-INIT-002 claim 5).
///
/// Shape enforced (the one this repo's working tree ships, which is what
/// PMAT-INIT-002 claim 1 calls "the PreToolUse schema"):
///
/// ```json
/// { "<hook-name>": { "PreToolUse": [ { "matcher": "write_file|code_execution",
///     "hooks": [ { "type": "command", "command": "./.agents/hooks/x.sh agy" } ] } ] } }
/// ```
///
/// Beyond the shape, the referenced script must EXIST and be EXECUTABLE. A hook
/// pointing at a missing script is not a broken hook — it is an absent one:
/// both clients fail open, so the quality gate the user believes they installed
/// simply never runs and never says so.
///
/// A relative `command` is reported as a WARNING for the same reason: it
/// resolves only when the client's cwd is the project root, and when it does
/// not resolve the failure is silent. `.claude/settings.json` solves this with
/// `$CLAUDE_PROJECT_DIR`; no AGY equivalent is documented, so this check names
/// the hazard rather than pretending it is settled.
pub(crate) fn check_agy_hooks_schema(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1664: AGY Hooks Schema";
    let agents_dir = project_path.join(AGY_DIR);
    if !agents_dir.is_dir() {
        return skip_check(
            name,
            "no .agents/ directory in this project — 0 hooks judged",
        );
    }
    let hooks_path = agents_dir.join("hooks.json");
    if !hooks_path.is_file() {
        return skip_check(
            name,
            ".agents/ is present but has no hooks.json — 0 hooks judged \
             (this project installs no AGY hook)",
        );
    }
    let Ok(text) = std::fs::read_to_string(&hooks_path) else {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: ".agents/hooks.json is unreadable".to_string(),
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
                    ".agents/hooks.json is not valid JSON (line {} column {}): {e}",
                    e.line(),
                    e.column()
                ),
                severity: Severity::Error,
            }
        }
    };

    let mut hard: Vec<String> = Vec::new();
    let mut soft: Vec<String> = Vec::new();
    let mut commands: Vec<AgyHookCommand> = Vec::new();

    let Some(entries) = value.as_object() else {
        return agy_hooks_not_an_object(name);
    };
    if let Some(legacy) = agy_legacy_hook_shape(name, entries) {
        return legacy;
    }
    if entries.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: ".agents/hooks.json is an empty object — it declares 0 hooks, \
                      so the file installs nothing"
                .to_string(),
            severity: Severity::Error,
        };
    }

    for (hook_name, body) in entries {
        agy_judge_hook_entry(hook_name, body, &mut hard, &mut commands);
    }
    for cmd in &commands {
        agy_judge_hook_command(project_path, cmd, &mut hard, &mut soft);
    }

    if commands.is_empty() && hard.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: format!(
                ".agents/hooks.json declares {} hook name(s) but 0 executable commands \
                 — nothing would ever run",
                entries.len()
            ),
            severity: Severity::Error,
        };
    }

    let summary = format!(
        "judged {} hook command(s) across {} hook name(s) in .agents/hooks.json",
        commands.len(),
        entries.len()
    );
    agy_verdict(name, &summary, &hard, &soft)
}

fn agy_hooks_not_an_object(name: &str) -> ComplianceCheck {
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Fail,
        message: ".agents/hooks.json must be a JSON object keyed by hook name \
                  ({\"<hook-name>\": {\"PreToolUse\": [...]}}) — found a non-object"
            .to_string(),
        severity: Severity::Error,
    }
}

/// Detect the older `{"hooks": [{"event": ..., "handler": {...}}]}` layout.
///
/// It is valid JSON and it looks deliberate, so without this it would fall
/// through the generic "no recognized event key" path with a message that does
/// not tell the reader what to do. It has a name and a migration, so say both.
///
/// WARN, NOT FAIL — and the distinction is the point. This check cannot cite an
/// authority for the PreToolUse layout, because there isn't one: issue #1031 /
/// PMAT-INIT-002 claim 1 is a single checkbox reading "pmat init --target agy
/// writes a compliant .agents/hooks.json with PreToolUse schema". It names the
/// schema and specifies no shape — no key nesting, no required fields, no
/// example. The nesting this repo now emits was derived from Claude Code's
/// documented hook format, which is a *different product*; no Anti-Gravity
/// schema is published anywhere this repository can reach.
///
/// An earlier revision hard-Failed on the flat layout — including the layout
/// this very repository shipped at HEAD. That made an invented schema a gating
/// error, which is precisely what `pmat agy sync` refuses to do (MACS-017,
/// #984) and what `pmat init` refuses for `plugins.json`. Enforcing a guess is
/// worse than not checking: a Fail asserts we know the right answer.
///
/// So: report the migration, do not gate on it. Promote to Fail only once an
/// Anti-Gravity hook schema is published and can be cited here.
fn agy_legacy_hook_shape(
    name: &str,
    entries: &serde_json::Map<String, serde_json::Value>,
) -> Option<ComplianceCheck> {
    let count = entries.get("hooks")?.as_array()?.len();
    Some(ComplianceCheck {
        name: name.to_string(),
        message: format!(
            ".agents/hooks.json uses the superseded flat layout \
             ({{\"hooks\": [{{\"event\": ..., \"handler\": ...}}]}}, {count} entry/entries). \
             pmat init emits the PreToolUse layout instead: \
             {{\"<hook-name>\": {{\"PreToolUse\": [{{\"matcher\": ..., \
             \"hooks\": [{{\"type\": \"command\", \"command\": ...}}]}}]}}}}. \
             Reported, not gated: no Anti-Gravity hook schema is published that \
             this check could cite, so the layout is a convention rather than a \
             requirement (see #1031)."
        ),
        status: CheckStatus::Warn,
        severity: Severity::Warning,
    })
}

/// Judge one `"<hook-name>": { "PreToolUse": [ ... ] }` entry, collecting the
/// command strings it reaches for the caller to resolve on disk.
fn agy_judge_hook_entry(
    hook_name: &str,
    body: &serde_json::Value,
    hard: &mut Vec<String>,
    commands: &mut Vec<AgyHookCommand>,
) {
    let Some(events) = body.as_object() else {
        hard.push(format!(
            "{hook_name}: value must be an object of event arrays (e.g. {{\"PreToolUse\": [...]}})"
        ));
        return;
    };
    let recognized: Vec<&String> = events
        .keys()
        .filter(|k| AGY_HOOK_EVENTS.contains(&k.as_str()))
        .collect();
    if recognized.is_empty() {
        hard.push(format!(
            "{hook_name}: no recognized event key (found {:?}, expected one of {:?})",
            events.keys().collect::<Vec<_>>(),
            AGY_HOOK_EVENTS
        ));
        return;
    }
    for event in recognized {
        agy_judge_event_rules(hook_name, event, &events[event], hard, commands);
    }
}

fn agy_judge_event_rules(
    hook_name: &str,
    event: &str,
    rules: &serde_json::Value,
    hard: &mut Vec<String>,
    commands: &mut Vec<AgyHookCommand>,
) {
    let Some(rules) = rules.as_array() else {
        hard.push(format!("{hook_name}.{event}: must be an array of rules"));
        return;
    };
    if rules.is_empty() {
        hard.push(format!(
            "{hook_name}.{event}: empty rule array — the event is declared but matches nothing"
        ));
        return;
    }
    for (i, rule) in rules.iter().enumerate() {
        let at = format!("{hook_name}.{event}[{i}]");
        match rule.get("matcher").and_then(|m| m.as_str()) {
            Some(m) if !m.trim().is_empty() => {}
            Some(_) => hard.push(format!("{at}: `matcher` is empty")),
            None => hard.push(format!("{at}: missing `matcher` (string)")),
        }
        agy_judge_rule_hooks(&at, rule.get("hooks"), hard, commands);
    }
}

fn agy_judge_rule_hooks(
    at: &str,
    hooks: Option<&serde_json::Value>,
    hard: &mut Vec<String>,
    commands: &mut Vec<AgyHookCommand>,
) {
    let Some(hooks) = hooks else {
        hard.push(format!("{at}: missing `hooks` array"));
        return;
    };
    let Some(hooks) = hooks.as_array() else {
        hard.push(format!("{at}: `hooks` must be an array"));
        return;
    };
    if hooks.is_empty() {
        hard.push(format!(
            "{at}: `hooks` is empty — the matcher fires and runs nothing"
        ));
        return;
    }
    for (j, hook) in hooks.iter().enumerate() {
        let origin = format!("{at}.hooks[{j}]");
        match hook.get("type").and_then(|t| t.as_str()) {
            Some("command") => {}
            Some(other) => hard.push(format!("{origin}: type `{other}` is not `command`")),
            None => hard.push(format!("{origin}: missing `type` (expected `command`)")),
        }
        match hook.get("command").and_then(|c| c.as_str()) {
            Some(c) if !c.trim().is_empty() => commands.push(AgyHookCommand {
                origin,
                command: c.to_string(),
            }),
            Some(_) => hard.push(format!("{origin}: `command` is empty")),
            None => hard.push(format!("{origin}: missing `command` (string)")),
        }
    }
}

/// Resolve a hook `command` against the project tree.
///
/// Only the first whitespace-separated token is the program; the rest are the
/// hook's own arguments (`./.agents/hooks/x.sh antigravity`). A token with no
/// `/` is a PATH lookup this check cannot resolve, and a token containing `$`
/// is expanded by the client — both are reported as unverified rather than
/// silently counted as fine.
fn agy_judge_hook_command(
    project_path: &Path,
    cmd: &AgyHookCommand,
    hard: &mut Vec<String>,
    soft: &mut Vec<String>,
) {
    let AgyHookCommand { origin, command } = cmd;
    let Some(program) = command.split_whitespace().next() else {
        hard.push(format!("{origin}: `command` has no program token"));
        return;
    };
    if program.contains('$') {
        soft.push(format!(
            "{origin}: program `{program}` contains a client-expanded variable, \
             so its existence could not be verified here"
        ));
        return;
    }
    if !program.contains('/') {
        soft.push(format!(
            "{origin}: program `{program}` is resolved from PATH by the client, \
             so its existence could not be verified here"
        ));
        return;
    }
    let relative = program.strip_prefix("./").unwrap_or(program);
    let resolved = if program.starts_with('/') {
        std::path::PathBuf::from(program)
    } else {
        project_path.join(relative)
    };
    if !resolved.is_file() {
        hard.push(format!(
            "{origin}: script `{program}` does not exist (looked at {}). \
             Both clients fail open, so this hook is a silent no-op",
            resolved.display()
        ));
        return;
    }
    if !agy_is_executable(&resolved) {
        hard.push(format!(
            "{origin}: script `{program}` exists but is not executable — \
             exec fails at run time and the client fails open (chmod +x it)"
        ));
        return;
    }
    if !program.starts_with('/') {
        soft.push(format!(
            "{origin}: program `{program}` is relative, so it resolves only when the \
             client's cwd is the project root; from anywhere else it exits 127 and \
             the client fails open (silently). \
             `.claude/settings.json` uses `$CLAUDE_PROJECT_DIR/...`; AGY documents no equivalent"
        ));
    }
}
