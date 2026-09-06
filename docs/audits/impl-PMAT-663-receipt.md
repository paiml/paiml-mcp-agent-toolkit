# Implementation receipt — PMAT-663 (AD-09: one orchestrator per repository per host)

Spec: `docs/specifications/agentic-delivery-pmat.md` §6 / §9.9. Epic #1153. Branch `PMAT-663-orchestrator-lock`.
Bundle side: paiml/paiml-implement#9 (`hooks/subagent-lock.sh`, `skills/paiml-implement/scripts/discover.sh`, `hooks/orchestrator-lock-selftest.sh`, `install.sh`, `verify.sh`). Routing: direct (bundle shell + docs).

## Verification (orchestrator runs)

| check | result |
|---|---|
| RED — the self-test against the hook as shipped on the bundle's main | `a second orchestrator (B) is refused while A holds the repo (exit 0, wanted 2)`: a second orchestrator was allowed |
| GREEN — `hooks/orchestrator-lock-selftest.sh` against the new hook | seven ✓: B refused while A holds the repo; the refusal names A; A may spawn; a lock whose session is gone is taken over (the lock then names B); a lock older than 12h is taken over; an `Edit` by B is not gated |
| bundle `verify.sh` | `orchestrator-lock` PASS among 27 rows |
| bashrs on the self-test | 0 errors (a fixed fixture timestamp; no bare-variable `rm -rf`) |
| the ≤1-subagent cap | unchanged; the new lock is a second, coarser gate in front of it |

## Not done, said plainly
- The spec's "xhigh assertion" cannot be enforced from a hook: the effort level is a session setting the hook does not receive. It stays `[U]` until the harness exposes it.
- The session id is derived from the hook's SessionStart registration for the cwd; when two sessions start in the same cwd within the same second the newest wins the guess — the lock then names the wrong session, and the refusal still fires for the third. `CLAUDE_SESSION_ID`, when exported, removes the guess.

Verdict: **DONE** once the PR merges green (docs only on the pmat side).
