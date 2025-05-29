# GitHub Actions Release Workflow - FIXED

**Status**: Fixed  
**Resolution Date**: 2025-05-28  
**Solution**: Created simple-release.yml as replacement

## Solution Summary

Replaced the complex, self-triggering automated-release.yml with a simple, manual-only workflow that:
1. Runs only on manual trigger (workflow_dispatch)
2. Executes all steps in a single job
3. Eliminates race conditions and self-triggering issues
4. Actually creates GitHub releases with binaries

## What Changed

1. **Created new workflow**: `.github/workflows/simple-release.yml`
   - Manual trigger only
   - Single job execution
   - Simple version bumping logic
   - Direct release creation

2. **Disabled old workflow**: Modified `automated-release.yml` to remove automatic triggers
   - Added "(Disabled)" to name
   - Removed push triggers
   - Added comment explaining why it's disabled

## How to Use

```bash
# Trigger a release manually from GitHub Actions UI:
# 1. Go to Actions tab
# 2. Select "Simple Release" workflow
# 3. Click "Run workflow"
# 4. Choose version bump type (patch/minor/major)
# 5. Click "Run workflow"

# Or use GitHub CLI:
gh workflow run simple-release.yml -f version_bump=patch
```

## Why This Works

1. **No self-triggering**: Manual-only means no recursive workflow runs
2. **Single job**: Everything runs sequentially in one job - no complex dependencies
3. **Simple logic**: No skip conditions or complex state management
4. **Proven pattern**: Uses standard GitHub Actions for releases (softprops/action-gh-release)

## Migration Steps

1. All future releases should use `simple-release.yml`
2. The old `automated-release.yml` remains but is disabled
3. Once confident, the old workflow can be deleted entirely

## Lessons Learned

- Complex workflows with self-triggering behavior are inherently fragile
- Manual releases give better control and predictability
- Simple, linear workflows are easier to debug and maintain
- Start simple, add complexity only when proven necessary