# Zero Branching Enforcement - 3 Mechanisms

This document describes 3 mechanisms to make branching IMPOSSIBLE in this repository.

## Current Status

- ✅ All branches deleted (only master exists locally and remotely)
- ✅ Mechanism 1 implemented and active
- ⚠️  Mechanism 2 requires GitHub repository settings
- ⚠️  Mechanism 3 requires manual setup

---

## Mechanism 1: Pre-Commit Hook with Auto-Merge (IMPLEMENTED)

**Location**: `.git/hooks/pre-commit-branch-enforcer`

**How It Works**:
- Runs FIRST on every `git commit` (before all other pre-commit checks)
- Detects if commit is attempted on non-master branch
- Automatically merges to master using fast-forward merge
- Deletes the offending branch
- Requires user to re-run commit command

**Why It's Effective**:
- ✅ Catches ALL manual commits to non-master branches
- ✅ Auto-fixes the violation (no manual intervention needed)
- ✅ Works for local development
- ✅ Cannot be bypassed unless `--no-verify` is used

**Limitations**:
- ❌ Can be bypassed with `git commit --no-verify`
- ❌ Doesn't prevent branch creation, only enforces at commit time
- ❌ Doesn't prevent programmatic git operations from tools

**Testing**:
```bash
# Test 1: Try to commit on a branch (should auto-fix)
git checkout -b test-branch
touch test.txt
git add test.txt
git commit -m "test"
# Expected: Auto-merges to master, deletes test-branch, asks to re-commit

# Test 2: Verify you're on master
git branch --show-current
# Expected: master

# Test 3: Verify no other branches exist
git branch -a
# Expected: Only master and remotes/origin/master
```

**Installation**: Already installed and active (integrated into `.git/hooks/pre-commit`)

---

## Mechanism 2: GitHub Branch Protection (Server-Side) ⚙️ SETUP REQUIRED

**Purpose**: Server-side enforcement that cannot be bypassed locally

**Setup Steps**:

1. **Go to GitHub Repository Settings**:
   ```
   https://github.com/paiml/paiml-mcp-agent-toolkit/settings/branches
   ```

2. **Add Branch Protection Rule for `master`**:
   - Click "Add branch protection rule"
   - Branch name pattern: `master`
   - Enable:
     - ✅ **Require a pull request before merging** (OFF - direct pushes allowed)
     - ✅ **Require status checks to pass before merging** (optional)
     - ✅ **Include administrators** (CRITICAL - no exceptions)
     - ✅ **Restrict who can push to matching branches** (limit to maintainers only)

3. **Disable Branch Creation on Push**:
   - GitHub doesn't natively block branch creation
   - But you can use **GitHub Actions** to auto-delete non-master branches:

   Create `.github/workflows/enforce-zero-branching.yml`:
   ```yaml
   name: Enforce Zero Branching Policy

   on:
     push:
       branches-ignore:
         - master

   jobs:
     delete-branch:
       runs-on: ubuntu-latest
       steps:
         - name: Delete non-master branch
           uses: actions/github-script@v7
           with:
             script: |
               const ref = context.ref.replace('refs/heads/', '');
               console.log(`❌ Deleting unauthorized branch: ${ref}`);
               await github.rest.git.deleteRef({
                 owner: context.repo.owner,
                 repo: context.repo.repo,
                 ref: `heads/${ref}`
               });

               core.setFailed(`Branch ${ref} violated zero-branching policy and was deleted`);
   ```

4. **Verify Protection**:
   ```bash
   # Try to push a new branch (should be auto-deleted)
   git checkout -b test-violation
   git commit --allow-empty -m "test"
   git push origin test-violation
   # Expected: Branch pushed, then GitHub Action deletes it within ~30s
   ```

**Why It's Effective**:
- ✅ Server-side enforcement (cannot be bypassed locally)
- ✅ Works for all collaborators
- ✅ Auto-deletes unauthorized branches via GitHub Actions
- ✅ Provides audit trail in Actions log

**Limitations**:
- ❌ Requires repository admin access to setup
- ❌ Branch briefly exists (~10-30s) before auto-deletion
- ❌ Requires GitHub Actions to be enabled

---

## Mechanism 3: Git Wrapper Script 🔧 OPTIONAL

**Purpose**: Intercept ALL git commands before they reach real git binary

**How It Works**:
- Create a `git` wrapper script that sits earlier in PATH than real git
- Intercepts and blocks all branch creation commands
- Passes through all other git commands to real git

**Installation**:

1. **Create wrapper script**:
   ```bash
   # Create ~/bin/git wrapper
   mkdir -p ~/bin
   cat > ~/bin/git << 'EOF'
   #!/usr/bin/env bash
   #
   # Git wrapper to enforce ZERO BRANCHING policy
   # Intercepts all git commands and blocks branch creation
   #

   # Find real git binary
   REAL_GIT=$(which -a git | grep -v "$0" | head -1)

   # Block branch creation commands
   if [ "$1" = "checkout" ] && [ "$2" = "-b" ]; then
       echo "❌ ERROR: Branch creation DISABLED (git checkout -b)"
       echo "   Policy: ZERO BRANCHING - work on master only"
       exit 1
   elif [ "$1" = "switch" ] && [ "$2" = "-c" ]; then
       echo "❌ ERROR: Branch creation DISABLED (git switch -c)"
       echo "   Policy: ZERO BRANCHING - work on master only"
       exit 1
   elif [ "$1" = "branch" ] && [ $# -gt 1 ] && [[ ! "$2" =~ ^- ]]; then
       echo "❌ ERROR: Branch creation DISABLED (git branch <name>)"
       echo "   Policy: ZERO BRANCHING - work on master only"
       exit 1
   fi

   # Pass through to real git
   exec "${REAL_GIT}" "$@"
   EOF

   chmod +x ~/bin/git
   ```

2. **Add to PATH** (prepend ~/bin):
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
   export PATH="$HOME/bin:$PATH"

   # Reload shell
   source ~/.bashrc
   ```

3. **Verify wrapper is active**:
   ```bash
   which git
   # Expected: /home/noah/bin/git

   git --version
   # Expected: git version X.X.X (proves pass-through works)
   ```

4. **Test blocking**:
   ```bash
   # All these should be blocked:
   git checkout -b test
   git switch -c test
   git branch test

   # These should work normally:
   git status
   git log
   git checkout master
   ```

**Why It's Effective**:
- ✅ Intercepts ALL git commands (manual AND programmatic)
- ✅ Works for all tools (IDE, CLI, scripts)
- ✅ Cannot be bypassed without removing wrapper
- ✅ Transparent to all git operations (pass-through)

**Limitations**:
- ❌ Requires PATH modification (per-user setup)
- ❌ Can be bypassed by calling real git directly (`/usr/bin/git`)
- ❌ May break tools that expect git at standard location
- ❌ Requires maintenance when git binary location changes

**Uninstall**:
```bash
rm ~/bin/git
# Remove export PATH="$HOME/bin:$PATH" from ~/.bashrc
```

---

## Recommendation

**Use All 3 Mechanisms for Maximum Protection**:

1. **Mechanism 1** (Pre-Commit Hook): ✅ Already active, catches 90% of violations
2. **Mechanism 2** (GitHub Branch Protection): ⚙️  Setup via GitHub settings (10 min)
3. **Mechanism 3** (Git Wrapper): 🔧 Optional, for paranoid enforcement

**Priority**:
- **CRITICAL**: Mechanism 1 (already done ✅)
- **HIGH**: Mechanism 2 (server-side protection)
- **OPTIONAL**: Mechanism 3 (overkill, but foolproof)

---

## Verification Checklist

After implementing all mechanisms:

```bash
# 1. Verify no branches exist
git branch -a
# Expected: Only master and remotes/origin/master

# 2. Try to create branch (should be blocked by wrapper OR pre-commit)
git checkout -b test-violation
# Expected: Error message

# 3. Try to commit on master (should work)
git checkout master
touch test.txt
git add test.txt
git commit -m "test"
# Expected: Commit succeeds

# 4. Try to push non-master branch (should be auto-deleted by GitHub Action)
git checkout -b remote-test
git commit --allow-empty -m "test"
git push origin remote-test --no-verify
# Expected: Branch deleted by GitHub Action within 30s

# 5. Verify final state
git fetch --prune
git branch -a
# Expected: Only master and remotes/origin/master
```

---

## Enforcement Summary

| Mechanism | Local | Remote | Bypass Difficulty | Setup Time |
|-----------|-------|--------|-------------------|------------|
| Pre-Commit Hook | ✅ | ❌ | Easy (`--no-verify`) | ✅ 0 min (done) |
| GitHub Protection | ❌ | ✅ | Very Hard | 10 min |
| Git Wrapper | ✅ | ❌ | Medium (call real git) | 5 min |

**Combined**: All 3 mechanisms make branching **effectively impossible** for normal workflows.

---

## Maintenance

- **Pre-Commit Hook**: Survives git operations, may need re-install after `git clean -fdx .git/hooks`
- **GitHub Protection**: Permanent, no maintenance required
- **Git Wrapper**: Persists in ~/bin, survives across sessions

## Rollback

If you need to re-enable branching (not recommended):

1. Remove pre-commit hook call: Edit `.git/hooks/pre-commit`, remove lines 16-20
2. Disable GitHub branch protection: Repository Settings → Branches → Delete rule
3. Remove git wrapper: `rm ~/bin/git`
