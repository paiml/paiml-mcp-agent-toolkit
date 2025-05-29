# GitHub Actions Release Workflow Bug

**Status**: Active  
**Discovered**: 2025-05-28  
**Severity**: High - Prevents releases from being created  
**Affected Versions**: v0.2.2, v0.2.3, v0.2.4, v0.3.0, v0.3.1  

## Summary

The automated release workflow creates Git tags but fails to create GitHub releases with binaries. This is due to a logic flaw in the workflow that causes it to skip release creation when triggered by its own version bump commits.

## Root Cause Analysis

### The Problem Flow

1. **Initial Trigger**: A legitimate commit (e.g., "feat: add complexity analysis") pushes to master
2. **Workflow Starts**: `automated-release.yml` workflow begins
3. **Version Bump**: Workflow creates commit "chore: release vX.X.X" and pushes it
4. **Second Workflow Triggered**: The push triggers ANOTHER instance of the workflow
5. **Skip Logic Activates**: The new instance sees "chore: release" in commit message and exits (line 45-46)
6. **Original Workflow Continues**: But may have issues or race conditions
7. **Result**: Tags are created but GitHub releases are never created

### Code Analysis

The problematic code in `.github/workflows/automated-release.yml`:

```yaml
# Lines 39-53
- name: Check if triggered by version bump
  id: check_skip
  run: |
    if [ "${{ github.event_name }}" = "push" ]; then
      LAST_COMMIT_MSG=$(git log -1 --pretty=%s)
      if echo "$LAST_COMMIT_MSG" | grep -q "^chore: bump version to\|^chore: release"; then
        echo "Skipping workflow - triggered by automated version bump"
        echo "skip=true" >> "$GITHUB_OUTPUT"
      else
        echo "skip=false" >> "$GITHUB_OUTPUT"
      fi
    else
      echo "skip=false" >> "$GITHUB_OUTPUT"
    fi
```

The workflow uses "chore: release vX.X.X" as the commit message (line 186), which matches the skip pattern.

## Evidence

1. **Missing Releases**:
   ```bash
   $ gh release list
   v0.2.1	Latest	v0.2.1	2025-05-28T20:33:41Z
   v0.2.0		v0.2.0	2025-05-28T18:40:47Z
   ```

2. **Existing Tags Without Releases**:
   ```bash
   $ git tag -l | sort -V | tail -10
   v0.2.0
   v0.2.1
   v0.2.2  # No release
   v0.2.3  # No release
   v0.2.4  # No release
   v0.3.0  # No release
   v0.3.1  # No release
   ```

3. **Workflow Logs Show Skip**:
   ```
   Tag v0.2.4 already exists, skipping release
   ```

## Impact

- Users cannot download pre-built binaries for recent versions
- Installation instructions reference non-existent releases
- CI/CD pipeline appears to work but silently fails to deliver artifacts
- Version tags exist without corresponding releases, causing confusion

## Temporary Workarounds

### 1. Manual Release Creation
```bash
# Trigger release workflow manually
gh workflow run automated-release.yml -f bump_version=patch
```

### 2. Create Release Without Binaries
```bash
# For historical purposes only
for tag in v0.2.2 v0.2.3 v0.2.4 v0.3.0 v0.3.1; do
  gh release create $tag --title $tag --notes "Retroactively created release."
done
```

## Permanent Fix Options

### Option 1: Change Commit Message Pattern
Modify line 186 to use a different commit message that doesn't match the skip pattern:
```yaml
git commit -m "chore: bump version to ${{ needs.test-and-check.outputs.new_version }}"
```

### Option 2: Use Actor-Based Skip Logic
Replace the commit message check with an actor check:
```yaml
- name: Check if triggered by version bump
  id: check_skip
  run: |
    if [ "${{ github.event_name }}" = "push" ]; then
      # Skip if triggered by GitHub Actions bot
      if [ "${{ github.actor }}" = "github-actions[bot]" ]; then
        echo "Skipping workflow - triggered by bot"
        echo "skip=true" >> "$GITHUB_OUTPUT"
      else
        echo "skip=false" >> "$GITHUB_OUTPUT"
      fi
    else
      echo "skip=false" >> "$GITHUB_OUTPUT"
    fi
```

### Option 3: Use Workflow Concurrency Control
Add concurrency control to prevent multiple instances:
```yaml
concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: false
```

## Testing Strategy

1. Create a test branch
2. Modify the workflow with the fix
3. Trigger a test release
4. Verify:
   - Version bump commit is created
   - Tag is created
   - GitHub release is created with all platform binaries
   - No duplicate workflow runs interfere

## Related Files

- `.github/workflows/automated-release.yml` - Main problematic workflow
- `.github/workflows/release.yml` - Cargo-dist based workflow (disabled for auto triggers)
- `scripts/create-release.ts` - Manual release creation script
- `scripts/update-version.ts` - Version update script

## Lessons Learned

1. **Workflow Recursion**: Be careful with workflows that trigger themselves
2. **Skip Logic**: Ensure skip conditions don't prevent legitimate operations
3. **Testing**: Test the full release pipeline, not just individual parts
4. **Monitoring**: Add alerts for missing releases after tag creation
5. **Documentation**: Document the expected workflow behavior clearly

## Action Items

- [ ] Implement permanent fix
- [ ] Test fix on a branch
- [ ] Create missing releases for affected versions
- [ ] Add monitoring for future release failures
- [ ] Update documentation on release process
- [ ] Consider adding release verification step