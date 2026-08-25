//! The dogfood skill must not tell its reader to run a flag pmat rejects.
//!
//! Issue #1062: `.claude/skills/dogfood/SKILL.md`, its `.agents/` mirror and
//! `run_audit2.sh` all told the reader to run `pmat serve --http --port 9977`.
//! There is no `--http` flag on `serve` and there never was one — `serve` has
//! carried `--transport <VALUE>` (with `http` as its DEFAULT) since v3.3.0, so
//! this was not a removed flag needing a compatibility alias, it was a spelling
//! invented by the doc. `git log -S'long = "http"'` over all history returns
//! nothing.
//!
//! What made it worth a permanent guard is not the typo but where it sat. The
//! stale flag was inside Gate 2g-iv, the check that HTTP **refuses to serve
//! without a bearer token**. clap exited 2 before any socket was bound, so both
//! probes reported `HTTP 000` (connection refused) and the gate's stated
//! failure conditions — "FAIL if it starts without a token, or if the
//! unauthenticated request is not rejected" — were both satisfied by a server
//! that did not exist. A security gate reported PASS for eight commits while
//! testing nothing. Measured with the real flag it is `401` unauthenticated and
//! `200` with the token, so the property does hold; nothing had checked.
//!
//! The check here is deliberately **flag existence**, not whole-line parsing.
//! Shell redirections, pipes, `$VAR`s and paths in a documented command line
//! make full parsing unreliable, but none of them produce a `--flag` token, so
//! this stays robust while still catching the drift that matters. Flag VALUES
//! are not checked, which is what lets the skill keep deliberate negatives like
//! `analyze complexity --format table` (P4b asserts `table` is rejected).

use super::on_big_stack;
use clap::CommandFactory;

/// Subcommands the skill invokes on purpose to prove they are rejected
/// (Gate 4 P5, "phantom subcommand"). Anything else unknown is real drift.
const DELIBERATE_PHANTOMS: &[&str] = &["definitely-not-a-real-command", "foo"];

/// The docs this guard covers, relative to the crate root.
const SKILL_DOCS: &[&str] = &[
    ".claude/skills/dogfood/SKILL.md",
    ".agents/skills/dogfood/SKILL.md",
];

fn read_doc(rel: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    std::fs::read_to_string(&path)
        .map_err(|e| format!("dogfood skill doc {} must be readable: {e}", path.display()))
        .expect("dogfood skill doc must be readable")
}

/// One documented invocation: the subcommand path, and the long/short flags
/// spelled on it.
#[derive(Debug, PartialEq, Eq, Clone)]
struct Site {
    subcommand: Vec<String>,
    flags: Vec<String>,
}

/// Pull `pmat …` invocations out of the fenced `bash` blocks of a markdown doc.
///
/// Prose is excluded by only reading fenced blocks — the skill says things like
/// "pmat ships three interfaces", which is not a command line.
fn extract_sites(md: &str) -> Vec<Site> {
    let mut sites = Vec::new();
    for block in bash_blocks(md) {
        for line in block.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            for rest in pmat_invocations(line) {
                if let Some(site) = parse_site(rest) {
                    sites.push(site);
                }
            }
        }
    }
    sites
}

/// The contents of every ```bash / ```sh fenced block.
fn bash_blocks(md: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current: Option<String> = None;
    for line in md.lines() {
        let t = line.trim_end();
        match current.as_mut() {
            None => {
                if t == "```bash" || t == "```sh" {
                    current = Some(String::new());
                }
            }
            Some(buf) => {
                if t == "```" {
                    blocks.push(std::mem::take(buf));
                    current = None;
                } else {
                    buf.push_str(line);
                    buf.push('\n');
                }
            }
        }
    }
    blocks
}

/// Everything following each `pmat ` token that begins a command, truncated at
/// the first shell metacharacter that ends the argument list.
fn pmat_invocations(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut from = 0usize;
    while let Some(idx) = line[from..].find("pmat ") {
        let start = from + idx;
        from = start + 5;
        // `pmat` must start a word — this rejects paths such as
        // `.pmat-cache/pmat dead-code-cache.json`.
        let preceded_ok = start == 0 || {
            let p = bytes[start - 1];
            !(p.is_ascii_alphanumeric() || p == b'-' || p == b'_' || p == b'/' || p == b'.')
        };
        if !preceded_ok {
            continue;
        }
        let rest = &line[start + 5..];
        let end = rest
            .find(['|', '>', ';', '&', '`', ')'])
            .unwrap_or(rest.len());
        out.push(&rest[..end]);
    }
    out
}

/// Split an argument list into leading subcommand words and the flags used.
fn parse_site(rest: &str) -> Option<Site> {
    let mut subcommand = Vec::new();
    let mut flags = Vec::new();
    // Stop treating words as subcommand names once a quoted positional appears
    // (`pmat query "error handling"`) — those are arguments, not commands.
    let mut words_closed = false;
    for raw in rest.split_whitespace() {
        if raw.starts_with('"') || raw.starts_with('\'') {
            words_closed = true;
        }
        let tok = raw.trim_matches(['"', '\'']);
        if tok == "-" || tok == "--" || tok.is_empty() {
            continue;
        }
        if tok.starts_with('-') {
            let name = tok.split('=').next().unwrap_or(tok);
            let name = name.trim_matches(['"', '\'']);
            if !flags.iter().any(|f| f == name) {
                flags.push(name.to_string());
            }
        } else if flags.is_empty()
            && !words_closed
            // A subcommand starts with a letter: this drops the `2` left behind
            // by a `2>&1` redirection, and file/path arguments.
            && tok.starts_with(|c: char| c.is_ascii_lowercase())
            && tok.chars().all(|c| c.is_ascii_lowercase() || c == '-')
        {
            subcommand.push(tok.to_string());
        }
    }
    if subcommand.is_empty() && flags.is_empty() {
        None
    } else {
        Some(Site { subcommand, flags })
    }
}

/// Walk `path` as far as the clap tree allows, returning how many words were
/// consumed as subcommands and the flags accepted at that depth.
///
/// The walk is greedy but tolerant: a word that is not a subcommand ends the
/// descent and is treated as a positional argument, because a documented line
/// like `pmat query error handling` passes a search phrase, not a command. Only
/// a FIRST word that resolves to nothing is real subcommand drift.
fn resolve_command(path: &[String]) -> (usize, Vec<String>) {
    let owned: Vec<String> = path.to_vec();
    on_big_stack(move || {
        let mut cmd = <crate::cli::Cli as CommandFactory>::command();
        let mut accepted = collect_flags(&cmd);
        let mut consumed = 0usize;
        for name in &owned {
            let next = cmd
                .get_subcommands()
                .find(|s| {
                    s.get_name() == name.as_str() || s.get_all_aliases().any(|a| a == name.as_str())
                })
                .cloned();
            let Some(next) = next else { break };
            cmd = next;
            consumed += 1;
            accepted.extend(collect_flags(&cmd));
        }
        // clap synthesises these at every level.
        accepted.push("--help".to_string());
        accepted.push("-h".to_string());
        accepted.push("--version".to_string());
        accepted.push("-V".to_string());
        (consumed, accepted)
    })
}

fn collect_flags(cmd: &clap::Command) -> Vec<String> {
    let mut out = Vec::new();
    for arg in cmd.get_arguments() {
        for long in arg.get_long_and_visible_aliases().unwrap_or_default() {
            out.push(format!("--{long}"));
        }
        if let Some(short) = arg.get_short() {
            out.push(format!("-{short}"));
        }
        for short in arg.get_short_and_visible_aliases().unwrap_or_default() {
            out.push(format!("-{short}"));
        }
    }
    out
}

/// Report every documented flag that clap would reject.
fn drifted_flags(md: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for site in extract_sites(md) {
        let label = if site.subcommand.is_empty() {
            "pmat".to_string()
        } else {
            format!("pmat {}", site.subcommand.join(" "))
        };
        if site
            .subcommand
            .first()
            .is_some_and(|s| DELIBERATE_PHANTOMS.contains(&s.as_str()))
        {
            continue;
        }
        let (consumed, accepted) = resolve_command(&site.subcommand);
        if consumed == 0 && !site.subcommand.is_empty() {
            problems.push(format!(
                "`{label}` — no such subcommand `{}`",
                site.subcommand[0]
            ));
            continue;
        }
        for flag in &site.flags {
            if !accepted.iter().any(|a| a == flag) {
                problems.push(format!("`{label}` — no such flag `{flag}`"));
            }
        }
    }
    problems
}

#[test]
fn every_flag_the_dogfood_skill_documents_exists_in_the_clap_tree() {
    for doc in SKILL_DOCS {
        let problems = drifted_flags(&read_doc(doc));
        assert!(
            problems.is_empty(),
            "{doc} documents commands pmat rejects — a reader following this \
             doc gets a clap error, and a GATE following it passes vacuously \
             (issue #1062):\n  {}",
            problems.join("\n  ")
        );
    }
}

#[test]
fn the_two_dogfood_skill_copies_stay_byte_identical() {
    let claude = read_doc(SKILL_DOCS[0]);
    let agents = read_doc(SKILL_DOCS[1]);
    assert_eq!(
        claude, agents,
        "{} and {} are mirrors; fixing one and not the other leaves the stale \
         copy live for whichever agent runtime reads it",
        SKILL_DOCS[0], SKILL_DOCS[1]
    );
}

/// Under-discovery guard. A broken extractor that finds nothing would make the
/// test above pass unconditionally — "we checked no commands" must never read
/// as "every command is fine".
#[test]
fn the_extractor_still_finds_the_documented_invocations() {
    let sites = extract_sites(&read_doc(SKILL_DOCS[0]));
    assert!(
        sites.len() >= 25,
        "extractor found only {} pmat invocations in the dogfood skill; it \
         found 32 when written, so it has almost certainly stopped matching \
         rather than the doc having shrunk",
        sites.len()
    );
    for expected in [
        vec!["serve".to_string()],
        vec!["analyze".to_string(), "complexity".to_string()],
        vec!["verify".to_string()],
    ] {
        assert!(
            sites.iter().any(|s| s.subcommand == expected),
            "extractor no longer sees `pmat {}`",
            expected.join(" ")
        );
    }
    // The serve site must carry its flags, or the #1062 check is inert there.
    let serve = sites
        .iter()
        .find(|s| s.subcommand == vec!["serve".to_string()])
        .expect("dogfood skill must exercise `pmat serve`");
    assert!(
        serve.flags.iter().any(|f| f == "--transport"),
        "expected `pmat serve --transport …`, got flags {:?}",
        serve.flags
    );
}

/// Counter-test: the guard must actually fire on the #1062 spelling. Without
/// this, a checker that accepts everything would look healthy above.
#[test]
fn the_guard_rejects_the_issue_1062_spelling() {
    let stale = "```bash\ntimeout 15 pmat serve --http --port 9977 2>&1 | head -3\n```\n";
    let problems = drifted_flags(stale);
    assert!(
        problems.iter().any(|p| p.contains("--http")),
        "guard must reject `pmat serve --http`, which is what #1062 reported; \
         got {problems:?}"
    );
}

/// Counter-test bounding the over-correction: the guard must NOT reject the
/// corrected spelling, nor the deliberate negatives the skill relies on.
#[test]
fn the_guard_accepts_the_corrected_spelling_and_deliberate_negatives() {
    let good = "```bash\ntimeout 15 pmat serve --transport http --port 9977 2>&1 | head -3\n\
                pmat analyze complexity --format table 2>&1\n\
                pmat definitely-not-a-real-command 2>&1\n```\n";
    assert_eq!(
        drifted_flags(good),
        Vec::<String>::new(),
        "`--transport http` is the real flag, `--format table` is a deliberate \
         bad VALUE (not a bad flag), and the phantom subcommand is Gate 4 P5"
    );
}

/// Prose must not be mistaken for a command line.
#[test]
fn prose_outside_fenced_blocks_is_not_treated_as_a_command() {
    let prose = "pmat ships **three** interfaces, and `.pmat-cache/pmat foo` is a path.\n";
    assert_eq!(extract_sites(prose), Vec::new());
}
