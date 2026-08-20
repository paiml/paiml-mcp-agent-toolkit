//! Tests for `pmat init`.
//!
//! The generator is a pure function, so most of these need no filesystem at
//! all; the ones that do use a tempdir and assert on bytes, not on log lines.
//!
//! Two of these tests exist specifically because the thing they check was
//! wrong in the tree before this command existed:
//!
//! * `mcp_config_declares_the_invocation_that_works` — the committed
//!   `.agents/mcp_config.json` named `cargo run --bin pmat -- serve
//!   --transport stdio`, which is a clap parse error (`invalid value 'stdio'`)
//!   preceded by a `cargo run` that cannot even start outside a Cargo
//!   workspace. It had never been executed.
//! * `hand_written_file_is_never_clobbered_without_force` — a bootstrap that
//!   overwrites a hooks manifest someone tuned by hand is data loss, and the
//!   only way to know it does not is to write a file and read it back.

use super::*;

fn root() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn paths(target: Target) -> Vec<&'static str> {
    plan(target).artifacts.iter().map(|a| a.path).collect()
}

fn read(dir: &Path, rel: &str) -> String {
    std::fs::read_to_string(dir.join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn outcome_of(report: &Report, rel: &str) -> Outcome {
    report
        .applied
        .iter()
        .find(|a| a.path == rel)
        .unwrap_or_else(|| panic!("{rel} not in report"))
        .outcome
}

// ── #1030 / PMAT-INIT-001: the four falsifiable claims ─────────────────────

/// Claim 2: "Generates `pmat-quality-feedback.sh` in the workspace."
/// Claim 4: "Writes a root rules file (AGENTS.md / GEMINI.md)."
///
/// Asserted for every target, because a bootstrap that only bootstraps one
/// client is a bootstrap that silently does nothing for the other two.
#[test]
fn every_target_emits_the_hook_script_and_a_root_rules_file() {
    for target in [Target::Agy, Target::Claude, Target::Ultracode] {
        let p = paths(target);
        assert!(
            p.contains(&".agents/hooks/pmat-quality-feedback.sh"),
            "{target}: no hook script in {p:?}"
        );
        assert!(p.contains(&"AGENTS.md"), "{target}: no root rules file");
    }
}

/// The emitted hook is the script this repository already exercises, not a
/// fork of it: `include_str!` binds them at compile time.
#[test]
fn emitted_hook_is_a_runnable_shell_script_that_calls_the_gate() {
    let hook = templates::QUALITY_FEEDBACK_HOOK;
    assert!(
        hook.starts_with("#!/bin/sh"),
        "needs a shebang to be exec'd"
    );
    assert!(
        hook.contains("pmat quality-gate"),
        "the hook must actually invoke the gate"
    );
    // Both clients treat a crash as an approval; the script must therefore
    // handle a missing pmat itself rather than dying.
    assert!(
        hook.contains("command -v pmat"),
        "hook must fail open deliberately, not accidentally"
    );
}

/// Claim 3: "Automatically installs PMAT as an MCP server."
///
/// This is the bug the ticket is about. The assertion is deliberately in two
/// halves: the invocation that works must be present, AND the invocation that
/// does not work must be absent — checking only the first would still pass if
/// someone re-added the broken one alongside it.
#[test]
fn mcp_config_declares_the_invocation_that_works_and_not_the_broken_one() {
    for target in [Target::Agy, Target::Claude, Target::Ultracode] {
        let plan = plan(target);
        let cfg = plan
            .artifacts
            .iter()
            .find(|a| a.path.ends_with("mcp_config.json") || a.path == ".mcp.json")
            .unwrap_or_else(|| panic!("{target}: no MCP registration emitted"));

        let json: serde_json::Value =
            serde_json::from_str(&cfg.contents).expect("MCP config must be valid JSON");
        let server = &json["mcpServers"]["pmat"];
        assert_eq!(
            server["command"], "pmat",
            "{target}: must name the installed binary, not a build tool"
        );
        assert_eq!(
            server["args"],
            serde_json::json!(["--mode", "mcp"]),
            "{target}: `pmat --mode mcp` is the measured-working stdio entrypoint"
        );

        for broken in ["cargo", "serve", "--transport", "stdio", "run"] {
            assert!(
                !cfg.contents.contains(broken),
                "{target}: emitted config still mentions `{broken}` — \
                 `pmat serve --transport stdio` exits 2 at clap parse with zero bytes of \
                 output, and `cargo run` cannot start outside a Cargo workspace"
            );
        }
    }
}

// ── the plan is pure ───────────────────────────────────────────────────────

#[test]
fn plan_touches_no_filesystem_and_is_deterministic() {
    let a = plan(Target::Agy);
    let b = plan(Target::Agy);
    let names_a: Vec<_> = a.artifacts.iter().map(|x| x.path).collect();
    let names_b: Vec<_> = b.artifacts.iter().map(|x| x.path).collect();
    assert_eq!(names_a, names_b);
    for (x, y) in a.artifacts.iter().zip(b.artifacts.iter()) {
        assert_eq!(x.contents, y.contents, "{} differs between plans", x.path);
    }
}

#[test]
fn agy_target_writes_the_agents_layout() {
    let p = paths(Target::Agy);
    for expected in [
        ".agents/hooks.json",
        ".agents/mcp_config.json",
        ".agents/skills/pmat-quality/SKILL.md",
    ] {
        assert!(p.contains(&expected), "missing {expected} in {p:?}");
    }
}

/// PMAT-INIT-002 claim 1 names the `PreToolUse` schema explicitly; the shape
/// committed at HEAD (`{"hooks":[{"event":…,"handler":…}]}`) is the other one.
#[test]
fn agy_hooks_manifest_uses_the_pretooluse_schema() {
    let json: serde_json::Value =
        serde_json::from_str(templates::AGY_HOOKS_JSON).expect("hooks.json must parse");
    let entry = &json["pmat-quality-feedback"]["PreToolUse"][0];
    assert_eq!(entry["matcher"], "write_file|code_execution");
    let cmd = entry["hooks"][0]["command"]
        .as_str()
        .expect("PreToolUse[0].hooks[0].command");
    assert!(
        cmd.ends_with("pmat-quality-feedback.sh antigravity"),
        "must invoke the shared entrypoint in antigravity mode, got {cmd}"
    );
}

/// Claude Code's manifest must not carry the relative-path hazard: a relative
/// command resolves against the client's cwd and, because a hook that fails to
/// launch is an *approval*, a wrong cwd is a silent no-op rather than an error.
#[test]
fn claude_hook_command_is_project_dir_anchored() {
    let json: serde_json::Value =
        serde_json::from_str(templates::CLAUDE_SETTINGS_JSON).expect("settings.json must parse");
    let cmd = json["hooks"]["PreToolUse"][0]["hooks"][0]["command"]
        .as_str()
        .expect("claude command");
    assert!(
        cmd.starts_with("$CLAUDE_PROJECT_DIR/"),
        "must be anchored, got {cmd}"
    );
    assert!(cmd.ends_with("pmat-quality-feedback.sh claude"));
}

/// CB-1650 accepts `low|medium|high|xhigh` and rejects the session-only values
/// `max` and `ultracode` by design. A skill this command emits must be one the
/// repo's own lint would accept.
#[test]
fn skill_frontmatter_uses_the_documented_keys_and_a_lintable_effort() {
    let md = templates::SKILL_MD;
    let front = md
        .strip_prefix("---\n")
        .and_then(|rest| rest.split("\n---\n").next())
        .expect("SKILL.md must open with YAML frontmatter");
    for key in ["effort:", "allowed-tools:", "description:"] {
        assert!(front.contains(key), "frontmatter missing {key}: {front}");
    }
    let effort = front
        .lines()
        .find_map(|l| l.strip_prefix("effort:"))
        .expect("effort key")
        .trim();
    assert!(
        ["low", "medium", "high", "xhigh"].contains(&effort),
        "effort `{effort}` is outside the set CB-1650 accepts; `max` and `ultracode` \
         are session-only values and are rejected there by design"
    );
}

// ── refusals: never invent a schema ────────────────────────────────────────

#[test]
fn agy_refuses_plugins_json_instead_of_inventing_it() {
    let p = plan(Target::Agy);
    assert!(
        !p.artifacts.iter().any(|a| a.path.contains("plugins.json")),
        "no plugins.json schema is defined anywhere; writing one would be a guess"
    );
    let r = p
        .refusals
        .iter()
        .find(|r| r.artifact.contains("plugins.json"))
        .expect("the omission must be reported, not silent");
    assert!(
        r.reason.contains("no plugins.json schema exists"),
        "the refusal must name the missing fact"
    );
    assert!(
        r.reason.contains("issues/1031"),
        "a refusal without a pointer is untriageable"
    );
}

#[test]
fn ultracode_refuses_to_invent_a_schema_but_still_writes_what_is_defined() {
    let p = plan(Target::Ultracode);
    let r = p
        .refusals
        .iter()
        .find(|r| r.artifact.contains("ultracode schema"))
        .expect("claim 1 of #1032 must be answered explicitly");
    assert!(
        r.reason.contains("session-only"),
        "the reason must be the actual one: ultracode is a harness setting, not a file format"
    );
    assert!(r.reason.contains("issues/1032"));

    // ...and the half that IS defined is still produced.
    assert!(
        paths(Target::Ultracode)
            .iter()
            .any(|p| p.ends_with(".ultracode.mjs")),
        "the committed judgment-workflow convention is defined and must be generated"
    );
}

/// Every refusal must be self-contained: what, why, and where it is tracked.
#[test]
fn every_refusal_is_actionable() {
    for target in [Target::Agy, Target::Claude, Target::Ultracode] {
        for r in plan(target).refusals {
            assert!(r.reason.len() > 120, "{}: reason is too thin", r.artifact);
            assert!(
                r.reason.contains("https://github.com/"),
                "{}: no tracking pointer",
                r.artifact
            );
        }
    }
}

// ── #1032: the ultracode judgment workflow ─────────────────────────────────

/// Strip `//` line comments so prose *about* a forbidden token cannot fail a
/// check about the code. Mirrors `qa_mcp_sweep::tests::code_of`, which guards
/// the committed workflow this one is modelled on.
fn code_of(src: &str) -> String {
    src.lines()
        .map(|l| match l.find("//") {
            Some(i) if !l[..i].contains('"') => &l[..i],
            _ => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn workflow() -> &'static str {
    templates::ULTRACODE_WORKFLOW_MJS
}

/// The generated workflow is held to exactly the invariants
/// `qa_mcp_sweep::tests::workflow_*` assert for the committed ground truth.
/// If the generator drifted away from the one committed example of this
/// convention, these fail.
#[test]
fn generated_workflow_matches_the_committed_conventions_invariants() {
    let w = workflow();
    assert!(w.len() > 500, "a stub cannot judge anything: {} B", w.len());
    assert!(
        w.contains("import "),
        "plain ESM so CI can `node --check` it"
    );
    assert!(w.contains("main()"), "must have an entry point");
    assert!(
        w.contains("main().catch("),
        "a failed run must exit non-zero, not resolve quietly"
    );
    assert!(w.contains("process.exit(1)"));
}

#[test]
fn generated_workflow_reads_only_the_deterministic_artifact() {
    let w = workflow();
    assert!(w.contains("artifacts/qa/mcp-sweep.json"));
    assert_eq!(
        w.matches("readFileSync(").count(),
        1,
        "exactly one read site keeps `reads only the artifact` structural"
    );
    for forbidden in ["qa-work mcp-sweep", "qa mcp-sweep", "--format json"] {
        assert!(
            !code_of(w).contains(forbidden),
            "the judgment layer must never re-run the deterministic sweep (found {forbidden})"
        );
    }
}

#[test]
fn generated_workflow_stamps_provenance_on_every_spawn() {
    let w = workflow();
    assert_eq!(
        w.matches("spawnSubagent(").count(),
        1,
        "more than one spawn site means `every spawn` stops being structural"
    );
    for var in [
        "PMAT_AGENT_HARNESS",
        "PMAT_AGENT_WORKFLOW_ID",
        "PMAT_AGENT_MODEL",
    ] {
        assert!(w.contains(var), "subagent env must carry {var}");
    }
    assert!(
        w.contains("\"ultracode-workflow\""),
        "PMAT_AGENT_HARNESS must be the value AgentHarness::UltracodeWorkflow parses"
    );
}

#[test]
fn generated_workflow_records_refusals_and_never_relies_on_resume() {
    let w = workflow();
    assert!(w.contains("work event --type refusal"));
    assert!(w.contains("catch"), "the refusal path must be reachable");
    assert!(
        !code_of(w).contains("resume"),
        "durable state is .pmat-work receipts, not session-bound continuation (spec E7)"
    );
}

/// Cheap syntax sanity for the generated ESM, so a truncated template cannot
/// reach a user's repo even where `node` is unavailable. `node --check` is the
/// authority and runs in `tests/init_workspace_t.rs`.
#[test]
fn generated_workflow_has_balanced_delimiters() {
    let code = code_of(workflow());
    for (open, close) in [('{', '}'), ('(', ')'), ('[', ']')] {
        assert_eq!(
            code.matches(open).count(),
            code.matches(close).count(),
            "unbalanced {open}{close}"
        );
    }
}

// ── applying to a real directory ───────────────────────────────────────────

#[test]
fn first_run_creates_every_artifact() {
    let dir = root();
    let p = plan(Target::Agy);
    let report = apply(&p, dir.path(), false).expect("apply");

    assert_eq!(report.written(), p.artifacts.len());
    assert_eq!(report.kept(), 0);
    for a in &p.artifacts {
        let on_disk = read(dir.path(), a.path);
        assert_eq!(on_disk, a.contents, "{} was not written verbatim", a.path);
    }
    // Nested paths must be created, not skipped.
    assert!(dir.path().join(".agents/skills/pmat-quality").is_dir());
}

/// Idempotence, stated as the ticket states it: running it twice must not
/// corrupt anything and the second run must SAY so.
#[test]
fn second_run_changes_nothing_and_reports_already_current() {
    let dir = root();
    let p = plan(Target::Agy);
    apply(&p, dir.path(), false).expect("first");

    let before: Vec<_> = p
        .artifacts
        .iter()
        .map(|a| std::fs::read(dir.path().join(a.path)).expect("read"))
        .collect();

    let second = apply(&p, dir.path(), false).expect("second");
    assert_eq!(second.written(), 0, "second run must write nothing");
    assert_eq!(second.already_current(), p.artifacts.len());
    assert_eq!(second.kept(), 0);

    let after: Vec<_> = p
        .artifacts
        .iter()
        .map(|a| std::fs::read(dir.path().join(a.path)).expect("read"))
        .collect();
    assert_eq!(before, after, "second run mutated bytes");

    let rendered = render_human(&second);
    assert!(
        rendered.contains("current"),
        "the second run must say the files are already current:\n{rendered}"
    );
}

/// The data-loss test. A hand-tuned manifest must survive, and the report must
/// say it was left alone and how to override.
#[test]
fn hand_written_file_is_never_clobbered_without_force() {
    let dir = root();
    let mine = "{\n  \"my-own-hook\": {\"PreToolUse\": []}\n}\n";
    std::fs::create_dir_all(dir.path().join(".agents")).expect("mkdir");
    std::fs::write(dir.path().join(".agents/hooks.json"), mine).expect("seed");

    let p = plan(Target::Agy);
    let report = apply(&p, dir.path(), false).expect("apply");

    assert_eq!(
        read(dir.path(), ".agents/hooks.json"),
        mine,
        "the user's manifest was overwritten — this is the data-loss bug"
    );
    assert_eq!(
        outcome_of(&report, ".agents/hooks.json"),
        Outcome::KeptYours
    );
    assert_eq!(report.kept(), 1);

    let rendered = render_human(&report);
    assert!(
        rendered.contains("--force"),
        "the report must say what it skipped AND how to override:\n{rendered}"
    );

    // Everything else still got written: one collision does not abort the run.
    assert!(dir.path().join(".agents/mcp_config.json").is_file());
    assert!(dir.path().join("AGENTS.md").is_file());
}

#[test]
fn force_replaces_only_the_files_that_differ() {
    let dir = root();
    let p = plan(Target::Agy);
    apply(&p, dir.path(), false).expect("first");
    std::fs::write(dir.path().join(".agents/hooks.json"), "{}\n").expect("edit");

    let report = apply(&p, dir.path(), true).expect("forced");
    assert_eq!(
        outcome_of(&report, ".agents/hooks.json"),
        Outcome::Overwritten
    );
    assert_eq!(report.written(), 1, "--force must not churn matching files");
    assert_eq!(report.already_current(), p.artifacts.len() - 1);
    assert_eq!(
        read(dir.path(), ".agents/hooks.json"),
        templates::AGY_HOOKS_JSON
    );
}

#[cfg(unix)]
#[test]
fn hook_script_is_executable_and_a_stripped_mode_is_repaired() {
    use std::os::unix::fs::PermissionsExt;
    let dir = root();
    let p = plan(Target::Claude);
    apply(&p, dir.path(), false).expect("apply");

    let hook = dir.path().join(".agents/hooks/pmat-quality-feedback.sh");
    let mode = std::fs::metadata(&hook).expect("stat").permissions().mode();
    assert_eq!(
        mode & 0o111,
        0o111,
        "a hook without +x cannot launch, and a hook that cannot launch is an APPROVAL"
    );

    // Strip it the way an archive extraction or a `git checkout` on a
    // permission-blind filesystem would, then re-run: the file matches so
    // nothing is rewritten, but the mode must still be repaired.
    let mut perms = std::fs::metadata(&hook).expect("stat").permissions();
    perms.set_mode(0o644);
    std::fs::set_permissions(&hook, perms).expect("chmod");

    let second = apply(&p, dir.path(), false).expect("second");
    assert_eq!(
        outcome_of(&second, ".agents/hooks/pmat-quality-feedback.sh"),
        Outcome::AlreadyCurrent
    );
    let mode = std::fs::metadata(&hook).expect("stat").permissions().mode();
    assert_eq!(mode & 0o111, 0o111, "second run must repair the exec bit");
}

#[test]
fn ultracode_target_lands_the_workflow_where_the_convention_puts_it() {
    let dir = root();
    let p = plan(Target::Ultracode);
    apply(&p, dir.path(), false).expect("apply");
    let wf = dir
        .path()
        .join("contracts/workflows/pmat-quality-sweep.ultracode.mjs");
    assert!(wf.is_file(), "workflow not written");
    assert_eq!(
        std::fs::read_to_string(&wf).expect("read"),
        templates::ULTRACODE_WORKFLOW_MJS
    );
    // Ultracode is Claude Code at xhigh effort (spec E1), so the client wiring
    // is Claude's — including the MCP registration that answers claim 2.
    assert!(dir.path().join(".mcp.json").is_file());
    assert!(dir.path().join(".claude/settings.json").is_file());
}

// ── reporting ──────────────────────────────────────────────────────────────

#[test]
fn json_report_carries_outcomes_and_refusals() {
    let dir = root();
    let p = plan(Target::Agy);
    let report = apply(&p, dir.path(), false).expect("apply");
    let json = render_json(&report);

    assert_eq!(json["target"], "agy");
    assert_eq!(
        json["artifacts"].as_array().expect("artifacts").len(),
        p.artifacts.len()
    );
    assert_eq!(json["summary"]["written"], p.artifacts.len());
    assert_eq!(json["summary"]["refused"], 1);
    assert_eq!(json["refused"][0]["artifact"], ".agents/plugins.json");
    assert!(json["artifacts"]
        .as_array()
        .expect("artifacts")
        .iter()
        .all(|a| a["outcome"] == "created"));
}

#[test]
fn human_report_prints_what_it_refused_and_why() {
    let dir = root();
    let report = apply(&plan(Target::Agy), dir.path(), false).expect("apply");
    let text = render_human(&report);
    assert!(text.contains("refused"), "{text}");
    assert!(text.contains(".agents/plugins.json"), "{text}");
    assert!(text.contains("issues/1031"), "{text}");
    assert!(text.contains("1 refused"), "{text}");
}

#[test]
fn wrap_preserves_every_word_and_respects_the_width() {
    let text = "alpha bravo charlie delta echo foxtrot golf hotel india juliet";
    let wrapped = wrap(text, 20, "  ");
    let words: Vec<&str> = wrapped.split_whitespace().collect();
    assert_eq!(words, text.split_whitespace().collect::<Vec<_>>());
    for line in wrapped.lines() {
        assert!(line.trim_end().len() <= 22, "too wide: {line:?}");
    }
}

#[test]
fn target_names_round_trip() {
    for t in [Target::Agy, Target::Claude, Target::Ultracode] {
        assert_eq!(t.to_string(), t.as_str());
    }
    assert_eq!(Target::Ultracode.as_str(), "ultracode");
}

#[test]
fn outcome_labels_are_fixed_width_and_have_stable_machine_names() {
    let labels = [
        Outcome::Created,
        Outcome::AlreadyCurrent,
        Outcome::KeptYours,
        Outcome::Overwritten,
    ];
    let width = labels[0].label().len();
    for o in labels {
        assert_eq!(o.label().len(), width, "{o:?} misaligns the report");
        assert!(!o.as_str().ends_with(' '));
    }
    assert!(Outcome::KeptYours.preserved_user_content());
    assert!(!Outcome::Overwritten.preserved_user_content());
}
