# Zero Branching Enforcement — RETIRED

> **This doctrine is retired and its mechanisms are not in force.** The document
> is kept because three other files pointed at it and because the reasoning for
> retiring it is worth having on record.

## Why it was retired

It contradicts the workflow this project requires and uses. `master` is
protected with required status checks (`ci / gate`, `feature-gate`,
`docs build (docs.rs environment)`, `pmat score`), so **direct pushes to master
are blocked**. Work lands as: feature branch → PR → required checks → merge.

A pre-commit hook that "verifies the commit is on master and blocks otherwise"
therefore forbids the only path by which anything can land. The two rules cannot
both hold.

## What was actually true, as of 2026-08-14

- **No committed hook checks the branch.** `grep -c -i branch .git/hooks/pre-commit`
  returns 0. The installer, the `pmat hooks` generator and every template are
  clean.
- The only enforcer on disk is `.git/hooks/pre-commit-branch-enforcer.disabled-r6`,
  renamed aside in April 2026 and not executable by that name.
- `templates/hooks/pre-commit-tdg.sh` still contained
  `[ -x "$SCRIPT_DIR/pre-commit-branch-enforcer" ] && run it` — a **latent
  re-arm**: restoring the name in any repository scaffolded from that template
  would silently reinstate the block. That delegation has been removed.
- This document's own status line read *"✅ All branches deleted (only master
  exists locally and remotely)"*. `git branch -r | wc -l` returns **71**.

That last point is the reason this file is corrected rather than quietly
deleted: a status line asserting a fact that one command refutes is the defect
class this repository spends most of its effort removing, and it sat at the top
of the document defining a doctrine.

## If you want branch discipline

Use the protected-branch settings and required checks, which are enforced
server-side by GitHub and cannot be bypassed with `--no-verify`. A pre-commit
hook is local, advisory and skippable; it is feedback, not a gate — the same
distinction that applies to the agent hooks in `.agents/hooks/README.md`.
