//! The `commit-msg` hook (AD-03, PMAT-655) and the strictness helpers: split
//! out of `hooks_command.rs` so that file stays under the 500-line cap
//! (PMAT-666). Same `impl HooksCommand`, same behaviour; the pre-commit
//! generation stays in `hooks_command.rs`.

use super::hooks_command::HooksCommand;
use anyhow::Result;
use std::fs;

impl HooksCommand {
    /// Install pre-push hook (fast local quality gate).
    /// Write the `commit-msg` hook (AD-03). Under `strict` a message without
    /// a `Pmat-Ticket:` trailer — or, failing that, a ticket id / issue
    /// reference matching the configured pattern anywhere in the message —
    /// is refused; otherwise the hook warns and lets the commit through.
    ///
    /// `strict` is the `--strict` flag OR `[hooks] strict = true`; the
    /// pattern comes from `[hooks] ticket_pattern`.
    /// `[hooks] strict` from the configuration, or false when there is no configuration.
    pub(crate) fn configured_strict() -> bool {
        crate::services::configuration_service::configuration()
            .get_config()
            .map(|c| c.hooks.strict)
            .unwrap_or(false)
    }

    /// Whether the pre-commit hook on disk was installed strict. `hooks verify --fix`,
    /// `hooks update` and the comply auto-install re-generate the hook from this, so a
    /// `--strict` given once is not silently dropped by the next automatic rewrite.
    pub(crate) fn installed_strict(&self) -> bool {
        let installed = fs::read_to_string(self.hooks_dir.join("pre-commit")).unwrap_or_default();
        installed.contains("export PMAT_HOOKS_STRICT=1") || Self::configured_strict()
    }

    pub(crate) fn install_commit_msg_hook(&self, strict: bool) -> Result<()> {
        let pattern = crate::services::configuration_service::configuration()
            .get_config()
            .map(|c| c.hooks.ticket_pattern)
            .unwrap_or_else(|_| "PMAT-[0-9]+|#[0-9]+".to_string());
        let hook_path = self.hooks_dir.join("commit-msg");
        let content = Self::generate_commit_msg_hook(strict, &pattern);
        fs::create_dir_all(&self.hooks_dir)?;
        fs::write(&hook_path, content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }
        Ok(())
    }

    /// The `commit-msg` hook text. `$1` is the message file. The trailer is
    /// read with `git interpret-trailers --parse`, so `Pmat-Ticket: PMAT-655`
    /// counts wherever git counts it; a bare id or `#NNN` in the body is the
    /// fallback for repositories without pmat work.
    pub(crate) fn generate_commit_msg_hook(strict: bool, pattern: &str) -> String {
        format!(
            r#"#!/usr/bin/env bash
# PMAT commit-msg hook — ticket linking (AD-03)
# auto-managed by PMAT — DO NOT EDIT
# A commit must name the work it belongs to: a `Pmat-Ticket: PMAT-NNN` trailer
# (git-native: `git log --format='%(trailers:key=Pmat-Ticket,valueonly)'`),
# or a ticket id / issue reference matching the configured pattern.
# strict={strict}: 1 = refuse (exit 1), 0 = warn and continue.
# Bypass (emergency, audited by the comply check over the branch): git commit --no-verify
set -u
MSG_FILE="$1"
PATTERN='{pattern}'
STRICT={strict}
trailer=$(git interpret-trailers --parse < "$MSG_FILE" 2>/dev/null | grep -iE '^Pmat-Ticket:' | head -1 | sed 's/^[^:]*:[[:space:]]*//')
if [ -n "$trailer" ]; then
    exit 0
fi
# Comment lines are not part of the message.
if grep -vE '^[[:space:]]*#' "$MSG_FILE" | grep -qE "$PATTERN"; then
    exit 0
fi
echo "PMAT commit-msg: no Pmat-Ticket trailer and no ticket reference matching '$PATTERN' in the message." >&2
echo "  add a trailer:  git commit -m '<subject>' -m 'Pmat-Ticket: PMAT-NNN'" >&2
if [ "$STRICT" = "1" ]; then
    echo "  [hooks] strict is on: commit refused." >&2
    exit 1
fi
echo "  (warning only: set [hooks] strict = true, or pmat hooks install --strict, to refuse)" >&2
exit 0
"#,
            strict = u8::from(strict),
            pattern = pattern
        )
    }

    /// Remove the `commit-msg` hook if it is ours.
    pub(crate) fn remove_commit_msg_hook(&self) -> Result<bool> {
        let hook_path = self.hooks_dir.join("commit-msg");
        if hook_path.exists() && self.is_pmat_managed(&hook_path)? {
            fs::remove_file(&hook_path)?;
            return Ok(true);
        }
        Ok(false)
    }
}
