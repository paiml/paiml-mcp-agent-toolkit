# Implementation receipt — PMAT-669 (#1185: container jobs restore the runner's ownership of its _work tree)

Branch `fix/1185-runner-ownership`. Routing: direct (a workflow edit the orchestrator owns).

| check | result |
|---|---|
| RED | `grep -n chown .github/workflows/*.yml` → nothing: the two container jobs (`ci.yml` `mutants`, `mutation-diff.yml` `mutation-diff`) run as root in `localhost:5000/sovereign-ci:stable`, write into the runner's shared `_work`, and leave it root-owned; #1185 counts thousands of foreign-owned paths per runner and eight consecutive aprender jobs dead at "Set up runner" |
| GREEN | a final `if: always()` step in both container jobs restores `stat -c %u:%g "$GITHUB_WORKSPACE/.."` over the workspace parent and `$RUNNER_TEMP`, prints the residue count before and after, and **fails the step if anything is still foreign-owned after** — so the log carries the evidence and the step cannot pass vacuously |
| measured here | both workflows parse (`yaml.safe_load`); the step sits inside each container job's step list (`ci.yml` job `mutants`, `mutation-diff.yml` job `mutation-diff`) |
| measured in CI | the first master run of `ci.yml` after merge prints `runner ownership: N path(s) not owned by … before, 0 after` in the `mutants` job — recorded on the issue |
| out of scope | the fleet-wide `ACTIONS_RUNNER_HOOK_JOB_COMPLETED` hook #1185 names as the alternative belongs to the runner hosts, not to this repository |

Verdict: **DONE** when merged; the CI evidence is posted on #1185 after the first run.
