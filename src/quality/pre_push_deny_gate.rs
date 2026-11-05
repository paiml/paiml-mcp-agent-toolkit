//! The pre-push `cargo deny check advisories` gate (OUT-324, epic #1604).
//!
//! A push that changes `Cargo.lock` or `deny.toml` can turn the supply-chain check red
//! without any source file changing. CI finds out ten minutes later; this gate finds out
//! before the push leaves the machine. [`PRE_PUSH_DENY_GATE`] is one bash function that
//! every PMAT-written pre-push hook splices in, so the three writers (the
//! `pmat hooks install` hook, `GitHookManager::install_hooks`, and
//! `scripts/install-git-hooks.sh`) cannot drift apart.
//!
//! Contract of `pmat_deny_gate "$PUSH_INPUT"` (return 0 = push may proceed):
//!
//! * `cargo deny` runs only when a pushed range touches `Cargo.lock` or `deny.toml`;
//!   otherwise it is not run at all (the control arm).
//! * A failed run refuses. A run that could not be measured (no `cargo-deny`, a config
//!   it cannot parse, a missing advisory database) refuses too: the exit status is the
//!   only thing trusted, and a non-zero one is never read as "nothing wrong".
//! * Two states exit 0 and are still refused: `duplicate items detected` (a planted
//!   duplicate ignore entry) and an `advisory-not-detected` warning for an advisory id
//!   the pushed range ADDED to `deny.toml` (a planted unknown advisory). Ignore entries
//!   that were already there and no longer match are not this push's doing and are left
//!   to `deny.toml` housekeeping, so they do not hold up an unrelated lockfile bump.
//! * `PMAT_DENY_FLAGS` is appended after `check` (for example `--disable-fetch` on an
//!   offline host). cargo-deny reads `~/.cargo/advisory-dbs/<hash>/`, not
//!   `~/.cargo/advisory-db`.

/// Placeholder a hook template carries where the gate function is spliced in.
pub(crate) const PRE_PUSH_DENY_GATE_PLACEHOLDER: &str = "__PMAT_DENY_GATE__";

/// The bash function. Splice it into a hook, then call
/// `pmat_deny_gate "$PUSH_INPUT"` with `PUSH_INPUT=$(cat)` read from the hook's stdin.
pub(crate) const PRE_PUSH_DENY_GATE: &str = r##"# >>> pmat-deny-gate (OUT-324) >>>
# Run `cargo deny check advisories` when a pushed range changes Cargo.lock or deny.toml.
# stdin of a pre-push hook: "<local ref> <local sha> <remote ref> <remote sha>" per ref;
# the caller reads it once (PUSH_INPUT=$(cat)) and passes it as $1. Returns 1 to refuse.
pmat_deny_gate() {
    local input="$1" need=0 added="" lref lsha rref rsha base changed ids
    local empty_tree="4b825dc642cb6eb9a060e54bf8d69288fbee4904"
    while read -r lref lsha rref rsha; do
        [ -n "${lsha:-}" ] || continue
        case "$lsha" in *[!0]*) ;; *) continue ;; esac # deleting a ref pushes no files
        base=""
        case "${rsha:-0}" in *[!0]*) base="$rsha" ;; esac
        if [ -z "$base" ] || ! git cat-file -e "${base}^{commit}" 2>/dev/null; then
            # New branch (or a remote object we lack): everything no remote ref has.
            base=$(git rev-list "$lsha" --not --remotes 2>/dev/null | tail -n 1) || base=""
            if [ -z "$base" ]; then
                continue
            fi
            base=$(git rev-parse --verify -q "${base}^" 2>/dev/null) || base="$empty_tree"
        fi
        if ! changed=$(git diff --name-only "$base" "$lsha" -- ':(glob)**/Cargo.lock' ':(glob)**/deny.toml' 2>/dev/null); then
            need=1 # cannot tell what changed: measure rather than guess
            continue
        fi
        if [ -n "$changed" ]; then
            need=1
            ids=$(git diff -U0 "$base" "$lsha" -- ':(glob)**/deny.toml' 2>/dev/null | grep -E '^\+' | grep -oE 'RUSTSEC-[0-9]+-[0-9]+|GHSA-[0-9a-z-]+' | sort -u | tr '\n' ' ' || true)
            added="$added $ids"
        fi
    done <<< "$input"
    [ "$need" = 1 ] || return 0

    echo -n "  cargo deny advisories (Cargo.lock/deny.toml changed)... "
    if ! cargo deny --version > /dev/null 2>&1; then
        echo "❌"
        echo "   REFUSED: cargo-deny is not installed, so the advisories check cannot be measured."
        echo "   Install: cargo install cargo-deny --locked"
        return 1
    fi
    local out rc=0 id
    # shellcheck disable=SC2086
    out=$(cargo deny --format json check ${PMAT_DENY_FLAGS:-} advisories 2>&1) || rc=$?
    if [ "$rc" -ne 0 ]; then
        echo "❌"
        echo "   REFUSED: cargo deny check advisories failed (exit $rc). Last diagnostics:"
        printf '%s\n' "$out" | grep -E '"severity":"(error|warning)"|"message"' | tail -n 8 | cut -c1-300 | sed 's/^/     /' || true
        echo "   Reproduce: cargo deny check advisories"
        return 1
    fi
    if printf '%s\n' "$out" | grep -q 'duplicate items detected'; then
        echo "❌"
        echo "   REFUSED: cargo deny reports duplicate items in deny.toml (a repeated ignore entry)."
        return 1
    fi
    for id in $added; do
        if printf '%s\n' "$out" | grep 'advisory-not-detected' | grep -q "\"$id\""; then
            echo "❌"
            echo "   REFUSED: deny.toml ignores $id, which no advisory/crate in the graph matches (unknown advisory)."
            return 1
        fi
    done
    echo "✅"
    return 0
}
# <<< pmat-deny-gate (OUT-324) <<<"##;

#[cfg(test)]
pub(crate) mod harness {
    //! Drives a real hook script against a throw-away git repository with a `cargo` shim
    //! (or, for the live arm, the real cargo-deny).
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    pub(crate) const ZERO: &str = "0000000000000000000000000000000000000000";

    pub(crate) struct Repo {
        pub dir: tempfile::TempDir,
    }

    pub(crate) struct Outcome {
        pub code: i32,
        pub output: String,
        pub deny_calls: usize,
    }

    #[derive(Clone, Default)]
    pub(crate) struct Shim {
        /// `cargo deny --version` fails when true.
        pub no_deny: bool,
        pub deny_rc: i32,
        pub deny_out: String,
    }

    fn git(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .output()
            .expect("run git");
        assert!(
            out.status.success(),
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout)
            .expect("utf8")
            .trim()
            .to_string()
    }

    impl Repo {
        pub(crate) fn new(deny_toml: &str) -> Self {
            let dir = tempfile::tempdir().expect("tempdir");
            let p = dir.path();
            git(p, &["init", "-q", "-b", "main"]);
            std::fs::create_dir_all(p.join("src")).expect("src");
            std::fs::write(
                p.join("Cargo.toml"),
                "[package]\nname = \"fx\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            )
            .expect("Cargo.toml");
            std::fs::write(p.join("src/main.rs"), "fn main() {}\n").expect("main.rs");
            std::fs::write(
                p.join("Cargo.lock"),
                "version = 4\n\n[[package]]\nname = \"fx\"\nversion = \"0.1.0\"\n",
            )
            .expect("Cargo.lock");
            std::fs::write(p.join("deny.toml"), deny_toml).expect("deny.toml");
            std::fs::write(p.join("README.md"), "one\n").expect("README");
            git(p, &["add", "."]);
            git(p, &["commit", "-q", "-m", "base"]);
            Self { dir }
        }

        pub(crate) fn path(&self) -> &Path {
            self.dir.path()
        }

        pub(crate) fn head(&self) -> String {
            git(self.path(), &["rev-parse", "HEAD"])
        }

        /// Overwrite `file` with `content` and commit it.
        pub(crate) fn commit_file(&self, file: &str, content: &str) -> String {
            std::fs::write(self.path().join(file), content).expect("write");
            git(self.path(), &["add", "."]);
            git(
                self.path(),
                &["commit", "-q", "-m", &format!("edit {file}")],
            );
            self.head()
        }
    }

    /// Run `hook` with one ref-update line on stdin. `shim = None` uses the real cargo.
    pub(crate) fn run_hook(
        hook: &Path,
        repo: &Repo,
        remote_sha: &str,
        local_sha: &str,
        shim: Option<&Shim>,
        extra_env: &[(&str, &str)],
    ) -> Outcome {
        let work = tempfile::tempdir().expect("shim dir");
        let log: PathBuf = work.path().join("cargo.log");
        std::fs::write(&log, "").expect("log");
        let mut path_var = std::env::var("PATH").unwrap_or_default();
        if shim.is_some() {
            let script = work.path().join("cargo");
            std::fs::write(
                &script,
                "#!/usr/bin/env bash\necho \"$*\" >> \"$SHIM_LOG\"\ncase \"$1\" in\n  fmt) exit 0;;\n  deny)\n    if [ \"${2:-}\" = \"--version\" ]; then [ \"${SHIM_NO_DENY:-0}\" = 1 ] && exit 101; exit 0; fi\n    printf '%s' \"${SHIM_DENY_OUT:-}\"; exit \"${SHIM_DENY_RC:-0}\";;\nesac\nexit 0\n",
            )
            .expect("shim");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
                    .expect("chmod");
            }
            path_var = format!("{}:{}", work.path().display(), path_var);
        }
        let mut cmd = Command::new("bash");
        cmd.arg(hook)
            .current_dir(repo.path())
            .env("PATH", path_var)
            .env("PMAT_BOOK_DIR", "/nonexistent-pmat-book")
            .env("SHIM_LOG", &log)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(s) = shim {
            cmd.env("SHIM_NO_DENY", if s.no_deny { "1" } else { "0" })
                .env("SHIM_DENY_RC", s.deny_rc.to_string())
                .env("SHIM_DENY_OUT", &s.deny_out);
        }
        for (k, v) in extra_env {
            cmd.env(k, v);
        }
        let mut child = cmd.spawn().expect("spawn hook");
        {
            use std::io::Write;
            let mut stdin = child.stdin.take().expect("stdin");
            let _ = writeln!(
                stdin,
                "refs/heads/work {local_sha} refs/heads/work {remote_sha}"
            );
        }
        let out = child.wait_with_output().expect("wait");
        let calls = std::fs::read_to_string(&log).unwrap_or_default();
        Outcome {
            code: out.status.code().unwrap_or(-1),
            output: format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            ),
            deny_calls: calls
                .lines()
                .filter(|l| l.starts_with("deny --format"))
                .count(),
        }
    }

    const CLEAN_DENY: &str = "[advisories]\nignore = []\n";
    const LOCK_V2: &str = "version = 4\n\n[[package]]\nname = \"fx\"\nversion = \"0.1.1\"\n";

    fn vulnerability() -> Shim {
        Shim {
            deny_rc: 1,
            deny_out: "{\"fields\":{\"code\":\"vulnerability\",\"message\":\"x\",\"severity\":\"error\"},\"type\":\"diagnostic\"}\n".into(),
            ..Shim::default()
        }
    }

    /// Every arm of the contract, against one hook script.
    pub(crate) fn assert_deny_gate_contract(hook: &Path) {
        // RED arm: Cargo.lock changed, deny fails -> push refused, deny ran.
        let r = Repo::new(CLEAN_DENY);
        let base = r.head();
        let head = r.commit_file("Cargo.lock", LOCK_V2);
        let o = run_hook(hook, &r, &base, &head, Some(&vulnerability()), &[]);
        assert_ne!(
            o.code, 0,
            "a lockfile bump that fails deny must be refused:\n{}",
            o.output
        );
        assert_eq!(o.deny_calls, 1, "deny must run once:\n{}", o.output);

        // RED arm: deny.toml changed, deny fails -> refused.
        let r = Repo::new(CLEAN_DENY);
        let base = r.head();
        let head = r.commit_file("deny.toml", "[advisories]\nignore = []\n# touched\n");
        let o = run_hook(hook, &r, &base, &head, Some(&vulnerability()), &[]);
        assert_ne!(
            o.code, 0,
            "a deny.toml change that fails deny must be refused:\n{}",
            o.output
        );

        // CONTROL: nothing in Cargo.lock/deny.toml changed -> deny is NOT run, push passes
        // even though a deny run would have failed.
        let r = Repo::new(CLEAN_DENY);
        let base = r.head();
        let head = r.commit_file("README.md", "two\n");
        let o = run_hook(hook, &r, &base, &head, Some(&vulnerability()), &[]);
        assert_eq!(o.code, 0, "an unrelated push must pass:\n{}", o.output);
        assert_eq!(
            o.deny_calls, 0,
            "deny must not run when neither file changed:\n{}",
            o.output
        );

        // CLEAN control: lockfile changed, deny clean -> passes, deny ran.
        let r = Repo::new(CLEAN_DENY);
        let base = r.head();
        let head = r.commit_file("Cargo.lock", LOCK_V2);
        let o = run_hook(hook, &r, &base, &head, Some(&Shim::default()), &[]);
        assert_eq!(o.code, 0, "a clean deny run must pass:\n{}", o.output);
        assert_eq!(
            o.deny_calls, 1,
            "deny must have been measured:\n{}",
            o.output
        );

        // Planted duplicate: exit 0 but `duplicate items detected` -> refused.
        let r = Repo::new(CLEAN_DENY);
        let base = r.head();
        let head = r.commit_file(
            "deny.toml",
            "[advisories]\nignore = [{ id = \"RUSTSEC-2024-0436\" }, { id = \"RUSTSEC-2024-0436\" }]\n",
        );
        let dup = Shim {
            deny_out: "{\"fields\":{\"labels\":[],\"message\":\"duplicate items detected\",\"severity\":\"warning\"},\"type\":\"diagnostic\"}\n".into(),
            ..Shim::default()
        };
        let o = run_hook(hook, &r, &base, &head, Some(&dup), &[]);
        assert_ne!(
            o.code, 0,
            "a planted duplicate must be refused:\n{}",
            o.output
        );

        // Planted unknown advisory: the id this push ADDED is reported not-detected.
        let r = Repo::new(CLEAN_DENY);
        let base = r.head();
        let head = r.commit_file(
            "deny.toml",
            "[advisories]\nignore = [{ id = \"RUSTSEC-2024-9999\", reason = \"planted\" }]\n",
        );
        let nd = |id: &str| {
            Shim {
            deny_out: format!("{{\"fields\":{{\"code\":\"advisory-not-detected\",\"labels\":[{{\"message\":\"no crate matched advisory criteria\",\"span\":\"{id}\"}}],\"message\":\"advisory was not encountered\",\"severity\":\"warning\"}},\"type\":\"diagnostic\"}}\n"),
            ..Shim::default()
        }
        };
        let o = run_hook(hook, &r, &base, &head, Some(&nd("RUSTSEC-2024-9999")), &[]);
        assert_ne!(
            o.code, 0,
            "a planted unknown advisory must be refused:\n{}",
            o.output
        );
        assert!(
            o.output.contains("RUSTSEC-2024-9999"),
            "name the id:\n{}",
            o.output
        );

        // CONTROL: the same warning for an id this push did not add passes.
        let o = run_hook(hook, &r, &base, &head, Some(&nd("RUSTSEC-2023-0001")), &[]);
        assert_eq!(
            o.code, 0,
            "a pre-existing not-detected ignore is not this push's:\n{}",
            o.output
        );

        // Unmeasurable: cargo-deny is not installed -> refused, never passed.
        let r = Repo::new(CLEAN_DENY);
        let base = r.head();
        let head = r.commit_file("Cargo.lock", LOCK_V2);
        let missing = Shim {
            no_deny: true,
            ..Shim::default()
        };
        let o = run_hook(hook, &r, &base, &head, Some(&missing), &[]);
        assert_ne!(
            o.code, 0,
            "no cargo-deny must refuse, not pass:\n{}",
            o.output
        );

        // Unmeasurable: deny exits 2 with no diagnostics -> refused.
        let broken = Shim {
            deny_rc: 2,
            deny_out: String::new(),
            ..Shim::default()
        };
        let o = run_hook(hook, &r, &base, &head, Some(&broken), &[]);
        assert_ne!(
            o.code, 0,
            "an unmeasurable deny run must refuse:\n{}",
            o.output
        );

        // Deleting a remote ref is not a lockfile change.
        let o = run_hook(hook, &r, &base, ZERO, Some(&vulnerability()), &[]);
        assert_eq!(o.code, 0, "a ref deletion carries no files:\n{}", o.output);
        assert_eq!(o.deny_calls, 0);

        // New branch (remote sha all zeros, no remotes): the range cannot be narrowed, so
        // deny is measured rather than skipped.
        let o = run_hook(hook, &r, ZERO, &head, Some(&vulnerability()), &[]);
        assert_ne!(
            o.code, 0,
            "a new branch must be measured, and this one fails:\n{}",
            o.output
        );
        assert_eq!(o.deny_calls, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::harness::{assert_deny_gate_contract, run_hook, Repo};
    use super::*;
    use std::path::PathBuf;

    fn write_hook(dir: &std::path::Path, name: &str, text: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, text).expect("write hook");
        p
    }

    #[test]
    fn the_gate_function_is_spliced_in_and_nonempty() {
        assert!(PRE_PUSH_DENY_GATE.contains("pmat_deny_gate()"));
        assert!(PRE_PUSH_DENY_GATE.contains("cargo deny"));
    }

    #[test]
    fn git_hook_manager_pre_push_hook_honours_the_contract() {
        let dir = tempfile::tempdir().expect("tempdir");
        let hooks = dir.path().join(".git/hooks");
        std::fs::create_dir_all(&hooks).expect("hooks");
        crate::quality::git_hooks::GitHookManager::new(dir.path())
            .install_hooks()
            .expect("install");
        assert_deny_gate_contract(&hooks.join("pre-push"));
    }

    #[test]
    fn install_git_hooks_sh_pre_push_hook_honours_the_contract() {
        let script = include_str!("../../scripts/install-git-hooks.sh");
        let start = script
            .find("<<'HOOK_EOF'\n#!/bin/bash\n# Pre-push hook")
            .expect("pre-push heredoc");
        let body_start = script[start..].find('\n').expect("nl") + start + 1;
        let end = script[body_start..].find("\nHOOK_EOF\n").expect("end") + body_start;
        let dir = tempfile::tempdir().expect("tempdir");
        let hook = write_hook(
            dir.path(),
            "pre-push",
            &format!("{}\n", &script[body_start..end]),
        );
        assert!(
            script.contains(PRE_PUSH_DENY_GATE.trim_end()),
            "scripts/install-git-hooks.sh must carry the shared gate verbatim"
        );
        assert_deny_gate_contract(&hook);
    }

    /// Live arm: the real cargo-deny against real fixtures. Skipped (loudly) only when
    /// cargo-deny or its advisory database is absent, unless PMAT_REQUIRE_CARGO_DENY=1.
    #[test]
    fn live_cargo_deny_refuses_planted_duplicate_and_unknown_advisory() {
        let have_deny = std::process::Command::new("cargo")
            .args(["deny", "--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        let home = std::env::var("HOME").unwrap_or_default();
        let have_db = std::fs::read_dir(format!("{home}/.cargo/advisory-dbs"))
            .map(|mut d| d.any(|e| e.map(|e| e.path().is_dir()).unwrap_or(false)))
            .unwrap_or(false);
        if !(have_deny && have_db) {
            assert!(
                std::env::var("PMAT_REQUIRE_CARGO_DENY").is_err(),
                "PMAT_REQUIRE_CARGO_DENY is set but cargo-deny/advisory-db is missing"
            );
            eprintln!("SKIPPED live cargo-deny arm: cargo-deny or ~/.cargo/advisory-dbs absent");
            return;
        }
        let dir = tempfile::tempdir().expect("tempdir");
        let hooks = dir.path().join(".git/hooks");
        std::fs::create_dir_all(&hooks).expect("hooks");
        crate::quality::git_hooks::GitHookManager::new(dir.path())
            .install_hooks()
            .expect("install");
        let hook = hooks.join("pre-push");
        let env = [("PMAT_DENY_FLAGS", "--disable-fetch")];
        let clean = "[advisories]\nignore = []\n";
        let lock2 = "version = 4\n\n[[package]]\nname = \"fx\"\nversion = \"0.1.1\"\n";

        // clean control: passes
        let r = Repo::new(clean);
        let base = r.head();
        let head = r.commit_file("Cargo.lock", lock2);
        let o = run_hook(&hook, &r, &base, &head, None, &env);
        assert_eq!(o.code, 0, "clean control:\n{}", o.output);

        // no-change control: a broken deny.toml in the tree is NOT consulted
        let r = Repo::new("[advisories]\nignore = [{ id = \"RUSTSEC-2999-0001\" }]\n");
        let base = r.head();
        let head = r.commit_file("README.md", "two\n");
        let o = run_hook(&hook, &r, &base, &head, None, &env);
        assert_eq!(o.code, 0, "deny must not run:\n{}", o.output);

        // unparsable id (cargo-deny exits 1)
        let r = Repo::new(clean);
        let base = r.head();
        let head = r.commit_file(
            "deny.toml",
            "[advisories]\nignore = [{ id = \"RUSTSEC-2999-0001\", reason = \"x\" }]\n",
        );
        let o = run_hook(&hook, &r, &base, &head, None, &env);
        assert_ne!(o.code, 0, "unparsable advisory id:\n{}", o.output);

        // well-formed id no advisory carries (cargo-deny exits 0 with a warning)
        let r = Repo::new(clean);
        let base = r.head();
        let head = r.commit_file(
            "deny.toml",
            "[advisories]\nignore = [{ id = \"RUSTSEC-2024-9999\", reason = \"x\" }]\n",
        );
        let o = run_hook(&hook, &r, &base, &head, None, &env);
        assert_ne!(o.code, 0, "unknown advisory id:\n{}", o.output);

        // duplicate ignore entry (cargo-deny exits 0 with a warning)
        let r = Repo::new(clean);
        let base = r.head();
        let head = r.commit_file(
            "deny.toml",
            "[advisories]\nignore = [{ id = \"RUSTSEC-2024-0436\", reason = \"x\" }, { id = \"RUSTSEC-2024-0436\", reason = \"x\" }]\n",
        );
        let o = run_hook(&hook, &r, &base, &head, None, &env);
        assert_ne!(o.code, 0, "duplicate ignore:\n{}", o.output);
    }
}
