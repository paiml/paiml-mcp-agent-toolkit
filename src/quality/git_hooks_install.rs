// Hook installation methods for GitHookManager
// included from git_hooks.rs — no `use` imports or `#!` attributes

impl GitHookManager {
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

    fn install_pre_push_hook(&self, hooks_dir: &Path) -> Result<()> {
        let hook_path = hooks_dir.join("pre-push");

        let hook_content = r#"#!/usr/bin/env bash
# PMAT Pre-Push Quality Verification

echo "🚀 Running pre-push quality verification..."

# Build in release mode
cargo build --release || {
    echo "❌ Release build failed"
    exit 1
}

# Run all tests
cargo test --all-features || {
    echo "❌ Tests failed"
    exit 1
}

# Run clippy
cargo clippy -- -D warnings || {
    echo "❌ Clippy warnings found"
    exit 1
}

# Check format
cargo fmt -- --check || {
    echo "❌ Code not formatted"
    echo "Run 'cargo fmt' to fix"
    exit 1
}

# Verify coverage
if command -v cargo-llvm-cov &> /dev/null; then
    COVERAGE=$(cargo llvm-cov report --summary-only | grep "TOTAL" | awk '{print $10}' | sed 's/%//')
    if (( $(echo "$COVERAGE < 95.0" | bc -l) )); then
        echo "❌ Coverage $COVERAGE% is below 95%"
        exit 1
    fi
fi

echo "✅ All pre-push checks passed!"
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
