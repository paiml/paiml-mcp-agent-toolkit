// Hook installation methods for GitHookManager
// included from git_hooks.rs — no `use` imports or `#!` attributes

impl GitHookManager {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// Install hooks.
    pub fn install_hooks(&self) -> Result<()> {
        let hooks_dir = self.repo_path.join(".git/hooks");

        if !hooks_dir.exists() {
            fs::create_dir_all(&hooks_dir).context("Failed to create hooks directory")?;
        }

        // Install pre-commit hook
        self.install_pre_commit_hook(&hooks_dir)?;

        // Install commit-msg hook
        self.install_commit_msg_hook(&hooks_dir)?;

        // Install pre-push hook
        self.install_pre_push_hook(&hooks_dir)?;

        println!("✅ Git hooks installed successfully");
        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn install_pre_commit_hook(&self, hooks_dir: &Path) -> Result<()> {
        let hook_path = hooks_dir.join("pre-commit");

        let hook_content = r#"#!/usr/bin/env bash
# PMAT Quality Gate Pre-Commit Hook

set -e

echo "🔍 Running PMAT quality gates..."

# Get staged Rust files
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(rs)$' || true)

if [ -z "$STAGED_FILES" ]; then
    echo "No Rust files to check"
    exit 0
fi

# Run quality checks on each file
for FILE in $STAGED_FILES; do
    echo "Checking: $FILE"

    # Run PMAT quality analysis
    pmat quality-gate "$FILE" || {
        echo "❌ Quality gate failed for $FILE"
        echo "Fix quality violations before committing."
        exit 1
    }
done

# Run tests
echo "Running tests..."
cargo test --quiet || {
    echo "❌ Tests failed"
    exit 1
}

# Check for SATD
SATD_COUNT=$(grep -r "TODO\|FIXME\|HACK\|XXX" --include="*.rs" . | wc -l)
if [ "$SATD_COUNT" -gt 0 ]; then
    echo "❌ Found $SATD_COUNT SATD markers (TODO, FIXME, HACK, XXX)"
    echo "Zero tolerance for technical debt!"
    exit 1
fi

echo "✅ All quality gates passed!"
"#;

        let mut file = fs::File::create(&hook_path).context("Failed to create pre-commit hook")?;

        file.write_all(hook_content.as_bytes())
            .context("Failed to write pre-commit hook")?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn install_commit_msg_hook(&self, hooks_dir: &Path) -> Result<()> {
        let hook_path = hooks_dir.join("commit-msg");

        let hook_content = r#"#!/usr/bin/env bash
# PMAT Commit Message Quality Hook

COMMIT_MSG_FILE=$1
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

# Check commit message format
if ! echo "$COMMIT_MSG" | grep -qE "^(PMAT-[0-9]+:|feat:|fix:|docs:|style:|refactor:|test:|chore:)"; then
    echo "❌ Invalid commit message format"
    echo "Commit message must start with:"
    echo "  - PMAT-XXX: for ticket work"
    echo "  - feat: for new features"
    echo "  - fix: for bug fixes"
    echo "  - docs: for documentation"
    echo "  - test: for tests"
    echo "  - refactor: for refactoring"
    exit 1
fi

# Check minimum length
if [ ${#COMMIT_MSG} -lt 10 ]; then
    echo "❌ Commit message too short"
    exit 1
fi

echo "✅ Commit message format valid"
"#;

        let mut file = fs::File::create(&hook_path)?;
        file.write_all(hook_content.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        Ok(())
    }

    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    fn install_pre_push_hook(&self, hooks_dir: &Path) -> Result<()> {
        let hook_path = hooks_dir.join("pre-push");

        let hook_content = r#"#!/usr/bin/env bash
# PMAT Pre-Push Quality Gate
# auto-managed by PMAT — DO NOT EDIT
#
# Fast local gate (~1-3 min) to catch issues before push.
# Full clean-room verification runs remotely after push.
# Bypass with: git push --no-verify

set -euo pipefail

# Skip for non-Rust repos
if [ ! -f Cargo.toml ]; then
    exit 0
fi

echo "🔍 PMAT Pre-Push Quality Gate"
echo "=============================="

FAILED=0

# 1. Format check (fastest, ~2s)
echo -n "  Format check... "
if cargo fmt --all -- --check > /dev/null 2>&1; then
    echo "✅"
else
    echo "❌"
    echo "   Run: cargo fmt --all"
    FAILED=1
fi

# 2. Compilation check (~10-30s)
echo -n "  Cargo check... "
if cargo check --all-targets 2> /dev/null; then
    echo "✅"
else
    echo "❌"
    FAILED=1
fi

# 3. Clippy (~10-30s, incremental)
echo -n "  Clippy... "
if cargo clippy --all-targets -- -D warnings 2> /dev/null; then
    echo "✅"
else
    echo "❌"
    FAILED=1
fi

# 4. Unit tests (~30-60s)
echo -n "  Unit tests... "
if cargo test --lib --quiet 2> /dev/null; then
    echo "✅"
else
    echo "❌"
    FAILED=1
fi

if [ "$FAILED" -ne 0 ]; then
    echo ""
    echo "❌ Pre-push gate FAILED — fix issues before pushing"
    echo "   Bypass (emergency): git push --no-verify"
    exit 1
fi

echo ""
echo "✅ Pre-push gate passed"
"#;

        let mut file = fs::File::create(&hook_path)?;
        file.write_all(hook_content.as_bytes())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        Ok(())
    }
}
