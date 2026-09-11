---
epic: null
status: active
vendors: []
---

# Goal mode — `pmat comply` as a closed loop

## 0. Provenance

Written 2026-09-09 against master `d3eef0198`, `pmat` 3.40.0, latest tag `v3.40.0`.

Every count in §5 and §14 was measured at that commit and moves as work lands; the
COMMANDS are the durable part, and each is given so a reader can re-measure rather than
trust the number. A count in this document that no longer reproduces is this document
being stale, which is the failure mode §5 is about.

Produced by the pipeline this specification itself mandates for a spec (§6):
Claude `Plan` → `agy /grillme` → `agy /teamwork` → assimilation. Every number below
carries the command that produced it, and the three findings that changed the design
were made by *disagreeing* with the reviews, not by adopting them:

- the grill's two lanes were one lane plus a retry, and the delegate said so rather
  than reporting a consensus;
- the teamwork lane silently swapped this specification's primary key (§4.1) and the
  delegate caught it;
- the load-bearing fact in §1.2 was found by neither, and is verified here.

Predecessor: `docs/specifications/agentic-delivery-pmat.md`. This document completes
its AD-07 (spec↔work traceability) and AD-10, and adds B/D/F, which it never modelled.
Its house rule is adopted verbatim:

> **A gate that cannot fail is theater.**

## 1. Executive summary

### 1.1 The ask

`pmat comply` should be a CLOSED LOOP. Seven invariants, from the operator:

| | invariant |
|---|---|
| **A** | No work done without a ticket linked to the roadmap. |
| **B** | All work grouped by a tagged release. |
| **C** | A commit message must contain the ticket for the work. |
| **D** | GitHub and the roadmap must EQUALLY reflect reality. Neither may be stale. |
| **E** | `docs/specifications/*` MUST be linked to a GitHub epic (and tickets) — period. |
| **E.1** | A specification must be reviewed by Claude plan + a quorum spanning quality, architecture, security, CRUX (competitive research), vendor docs where relevant, and adversarial review. |
| **F** | Nothing may remain untagged to a version. It is either closed, or assigned. |
| **G** | `pmat goal <driver> [<spec>]` works continuously, cutting tagged releases. |

### 1.2 The finding that makes this cheap

`pmat comply ledger` reported **157 CB rules, 0 ENFORCED, 157 NEUTERED** when this was
written; at `f5eccdfb3` it reports **163 rules, 2 ENFORCED, 161 NEUTERED** — CB-2113 and
CB-2115, landed by steps 2 and 4 of §11. It names one
cause: `continue-on-error` on the `Ladder gate — pmat comply` step. Read on its own that
says the only way to enforce anything is to unblock a step that is *currently red on four
pre-existing failures*, and whose own comment documents a five-step sequence to unwedge it.

It is computed over an incomplete root set. There are **six** required contexts, not five:

```console
$ gh api repos/paiml/paiml-mcp-agent-toolkit/rules/branches/master \
    --jq '[.[]|select(.type=="required_status_checks")|.parameters.required_status_checks[].context]'
["gate"]

$ gh api repos/paiml/paiml-mcp-agent-toolkit/rulesets --jq '.[]|{id,name,enforcement}'
{"id":13878864,"name":"Green Main — unified gate enforcement","enforcement":"active"}

$ gh api repos/paiml/paiml-mcp-agent-toolkit/branches/master/protection \
    --jq '.required_status_checks.contexts'
["ci / gate","feature-gate","docs build (docs.rs environment)","pmat score","provable ladder"]

$ gh pr view 1243 --json statusCheckRollup \
    | jq -r '[.statusCheckRollup[]|(.name//.context)]|map(select(.=="gate" or .=="ci / gate"))'
["ci / gate","gate"]
```

An **active ruleset** requires the unprefixed context `gate`. That is the top-level
`gate:` job in `.github/workflows/ci.yml` — in this repository, readable, already
carrying `roadmap-validate`'s verdict, and distinct from `ci / gate`, which lives inside
the opaque reusable workflow. Both appear on a real PR.

`src/services/gate_effect/required.rs` reads only branch protection
(`grep -c "rules/branches"` → **0**), so the ledger cannot see the sixth root. And
`.github/required-status-checks.txt` not only omits it, it asserts the opposite:

> the unrequired top-level job in ci.yml whose display name is `gate`

That sentence is **false** as of ruleset 13878864. Correcting it is prerequisite **P2**,
and it is the smallest change in this document with the largest consequence: it converts
"no rule in this repository can be enforced without first unwedging a red step" into
"a rule becomes enforced by joining one `needs:` list".

### 1.3 What this specification adds

Eight rules, one roadmap field, three commands, and a driver loop — sequenced so that
each lands **green**, and so that the loop (G) is built last, because *a loop that
automates a broken invariant automates the breakage*.

## 2. Doctrine

1. **A gate that cannot fail is theater.** Every rule here ships with a named falsifier
   and a control, and the control runs in CI *before* the rule is trusted.
2. **`not_measured` is not a pass.** A rule that could not reach its input FAILS. But it
   is distinct from `not_applicable`, which is a structural statement about where the
   rule is running (§3.3). Conflating them is what makes a gate hostile.
3. **The fixer ships before the refusal.** A refusal with no way to comply is a wall.
4. **No CB rule may depend on `pmat goal`, on a skill, or on Claude or agy being
   installed.** Every gate must be checkable from a clean clone with
   `git`, `jq`, `cargo` and `gh`. `pmat goal` is the single surface permitted a driver.
5. **One number, one place.** Where a fact could live on three surfaces, name one
   authority and make the others projections (§4.1). A second source of truth is a
   second number to disagree — the defect `pmat comply numeric-claims` exists to find.
6. **Land green.** A rule that is red on the day it lands gets disabled. Where existing
   data violates a new rule, the data is fixed under its own ticket first.

## 3. The loop

### 3.1 States

A **ticket** is the unit. A spec and a release are contexts a ticket sits in.

```
              E, E.1                  A                     B/F
 SPEC-DRAFT ─────────► SPEC-REVIEWED ────► EPIC-OPEN ─────────► TICKET-OPEN
                                                                     │ B/F
                                                                     ▼
                                                            TICKET-SCHEDULED
                                                                     │ A
                                                                     ▼
                                                              IN-PROGRESS
                                                                     │ C
                                                                     ▼
                                                               COMMITTED
                                                                     │ D
                                                                     ▼
                                                                  MERGED
                                                                     │ F
                                                                     ▼
                                                                  TAGGED
                                                                     │ G
                                                                     ▼
                                                                 RELEASED
```

| state | predicate | where the fact lives |
|---|---|---|
| `SPEC-DRAFT` | a tracked `docs/specifications/**.md` | git |
| `SPEC-REVIEWED` | `docs/audits/spec-<slug>-review.json` exists, `spec_sha256` matches, every required role PASS | repo |
| `EPIC-OPEN` | an open issue labelled `epic` with the spec's tickets as sub-issues (§4.3) | GitHub |
| `TICKET-OPEN` | roadmap item with `github_issue: N`, issue `N` open, `N` a sub-issue of the epic | roadmap + GitHub |
| `TICKET-SCHEDULED` | roadmap `release: X.Y.Z`; issue on milestone `X.Y.Z` | roadmap + GitHub |
| `IN-PROGRESS` | roadmap `status: inprogress` | roadmap |
| `COMMITTED` | ≥1 commit carries `Pmat-Ticket: <id>` | git trailers |
| `MERGED` | those commits are ancestors of master; item `completed`; issue closed | git + both |
| `TAGGED` | tag `vX.Y.Z` contains every merged commit of every ticket with `release: X.Y.Z` | git |
| `RELEASED` | tag + GitHub release + crates.io version | `scripts/release-check.sh` |

### 3.2 Illegal transitions

| # | illegal | forbidden by | refused by |
|---|---|---|---|
| T1 | spec worked with no review | E.1 | CB-2111 |
| T2 | active spec with no epic | E | CB-2110 |
| T3 | edit with no ticket | A | W-006 hook (point of action) + CB-2112 (CI) |
| T4 | `IN-PROGRESS` skipping `TICKET-SCHEDULED` | B, F | CB-2114 |
| T5 | commit with no trailer | C | `commit-msg` hook + CB-2113 |
| T6 | trailer naming a ticket not in the roadmap | A + C | CB-2113 leg 2 |
| T7 | roadmap and GitHub disagreeing | D | CB-2115 |
| T8 | a tag containing a commit whose ticket names another release | B | CB-2116 |
| T9 | master accumulating untagged merged commits without a cut | F2 | CB-2116 leg 2 |
| T10 | tag with no release/crates.io version | — | **already closed** — AD-01 `release-check` |

### 3.3 `not_measured` vs `not_applicable` — the distinction that keeps this usable

The grill's strongest objection was that "`not_measured` fails" makes the repository
hostile to outside contributors: a fork PR has no write-scoped token, four of the eight
rules need GitHub, so every fork PR would be permanently red.

The objection is right about the consequence and wrong about the cause. Two different
events were being given one name.

| verdict | meaning | on a red |
|---|---|---|
| `measured` | the rule read its input and judged | pass or fail on the merits |
| `not_measured` | the rule SHOULD have read its input and could not — no token where one was expected, an unparsable roadmap, an API error | **FAIL** |
| `not_applicable` | the rule structurally cannot apply in this context, and the context is named | **PASS**, and the reason is printed |

A **fork PR** is `not_applicable` for the four GitHub-side rules (CB-2110, 2112, 2114,
2115), for a stated reason: *a fork's PR cannot change this repository's issues,
milestones or epics, so there is nothing about them for this PR to have broken.* Those
four are re-checked on the push to master, which is where the merge actually lands and
where a token exists — with two carve-outs that exist for the same reason.

**On a push to master, CB-2110, CB-2112 and CB-2114 judge only the half they can see in the
tree**: front-matter present and parsable, `github_issue` present and well-formed,
`release:` present on an `inprogress` item. Every assertion about live GitHub state — the
epic is open, the issue exists and is open, the milestone exists and carries the issue — is
deferred to the scheduled run. Closing an epic in the GitHub UI must not redden a commit that
was green when it merged, and the file half is exactly the half a commit can break. (Decided
by quorum 2026-09-11, 4 of 5 seats; the dissent would take all four rules off the master push
entirely, as P4 does for CB-2115 — §11.2 Q16.)

**CB-2115 does not run on a master push at all.** It
reads GitHub live, so its verdict on master changes while master does not: an issue opened
by hand turns a green commit red with no diff to blame, which is what happened on
`5af9a0f0d` at 06:08Z on 2026-09-11. CB-2115 runs on pull requests, where a human can fix
what the diff caused, and on a schedule, where a disagreement **opens a ticket instead of
failing a build**. (Decided by quorum 2026-09-11, 3 of 4 seats — §11.2; PMAT-1309.)

**Except when the fork PR edits the records those rules judge.** A fork cannot change
issues, milestones or epics, but it can edit `docs/roadmaps/roadmap.yaml` and
`docs/specifications/`, which are the other half of every one of those four rules. A fork
PR that touches either is `not_measured` — it fails closed — because the alternative is a
pull request that passes here and turns master red on the merge, which is the shape
doctrine 6 exists to refuse. A fork PR that touches neither keeps the `not_applicable`
pass above. Implemented by PMAT-1310.

The four offline rules (CB-2111, 2113, 2116, 2117) run everywhere, forks included.

`not_applicable` is only legitimate when the *context* is what makes it inapplicable,
never when the *input* is missing. A missing `GH_TOKEN` on a same-repo PR is
`not_measured` and fails, because a token was expected there.

## 4. Data model

### 4.1 One key, three surfaces

> **The release identity is a bare semver string (`3.41.0`, never `v3.41.0`). It is the
> primary key on all three surfaces. No surface may mint one.**

| surface | holds | authority |
|---|---|---|
| git tag `vX.Y.Z` | the immutable boundary and its commit set | **membership** — what shipped is what the tag contains |
| GitHub milestone titled `X.Y.Z` | the intent: which issues are in scope | **scope**, while the release is open |
| roadmap `release: "X.Y.Z"` | an offline-readable cache | **never** — a projection; `pmat work sync` is its only writer |

**RR-RELEASE.** A roadmap item's `release` is valid iff a milestone with that exact
title exists. A milestone may close iff tag `v<title>` exists. A tag may be cut iff its
milestone has zero open issues. The string is never translated, normalised or prefixed
except in the tag, where the `v` is added at exactly one place in the code.

Strict order — tag ⊃ milestone ⊃ roadmap — so there is no bidirectional merge and no
conflict resolution. A surface can only *fail to reflect* the one above it.

**The title is the intent, and the cut refuses rather than overrule it.** §10.3 derives the
version from the labels of the tickets in the release, so a milestone titled `3.41.0` that
collects a `breaking-change` ticket is titled wrongly, not scheduled wrongly. The cut does
**not** rename it. It stops, prints both numbers, and a human either retitles the milestone
or moves the ticket out of it. An automatic rename would let the cut satisfy RR-RELEASE by
editing the very surface CB-2114 compares against — the rule would pass because the evidence
moved, and master would go red afterwards on drift no commit introduced. Refusing is the loud
failure; renaming is the quiet one. A rename by anyone else, at any other time, is the drift
CB-2114 exists to catch. (Decided by quorum 2026-09-11, 3 of 4 seats; the dissent would fix
the title at creation and refuse the ticket at assignment instead — §11.2.)

**The key is the string, not the milestone's numeric id.** The teamwork lane proposed
tracking the id so a rename would be invisible; that is rejected. Renaming a milestone
IS renaming the release, and CB-2114 going red and naming the rename is the correct
behaviour, not a defect to design around.

### 4.2 Schema change — one field

`src/models/roadmap_types.rs`:

```rust
/// The release this ticket ships in: a bare semver string ("3.41.0"), never
/// "v3.41.0". A PROJECTION of the GitHub milestone of the same title
/// (RR-RELEASE); `pmat work sync` is the only writer. `None` on a completed or
/// cancelled item means the item predates release tracking; `None` on a
/// `planned` item means NOT YET SCHEDULED, which §11.2 P1 makes the normal state
/// of the backlog; `None` on an `inprogress` item is a CB-2114 violation.
#[serde(default, skip_serializing_if = "Option::is_none")]
pub release: Option<String>,
```

`serde(default)` + `skip_serializing_if` so all 273 existing entries round-trip
unchanged — a schema change that reformats 273 entries buries the diff that matters.

The 211 `completed` and 2 `cancelled` items are **not** backfilled; backfilling would
invent facts. CB-2114 scopes itself to `status ∈ {planned, inprogress}` and says so.

Deliberately **not** added: `epic:`, `milestone:`, `tag:`, `commits:`. Each is derivable
(epic from the issue's parent, milestone *is* `release`, tag is `v` + `release`, commits
from trailers) and each would be a fourth number to disagree.

### 4.3 The spec ↔ epic edge

YAML front-matter on the spec itself, so a check can read it with no network:

```yaml
---
epic: 1234         # an open GitHub issue labelled `epic`
status: active     # active | superseded | historical
vendors: [cuda]    # optional; adds a vendor: role to E.1's required set
---
```

`status: historical` exempts a spec from CB-2110's epic leg, not its parse leg. An
escape hatch nobody can see is a hole, so `pmat comply report` lists every `historical`
spec **by name** on every run.

`status` IS the exemption, and **nothing bounds who changes it**. That is stated here rather
than papered over: the diff is the only control. A `CODEOWNERS` entry on
`docs/specifications/` and a review artifact for the flip were both considered and both
rejected — each is a gate whose own bypass is another line in the same pull request, and a
rule that can be switched off by an unreviewable edit is better honest about it than dressed
in ceremony. What makes the hatch visible is the naming above: every `historical` and
`superseded` spec is listed by name on every `pmat comply report` run, so a flip shows up in
the next report whoever made it. (Decided by quorum 2026-09-11, 4 of 4 seats — §11.2.)

Epic membership is read from GitHub's native **sub-issue** relation, and only from it
(operator decision, 2026-09-09). An `Epic: #N` line in an issue body is **not** accepted:
a body line is prose that drifts, a sub-issue is a structural edge GitHub maintains, and
accepting both would mean two answers to one question — doctrine 5.

### 4.4 Stress cases

| case | what happens |
|---|---|
| **hotfix cut from an old tag** while a newer milestone is open | Legal. The order in §4.1 is **per release**, not global: `3.40.1` gets its own milestone and its own tag, and neither is compared against `3.42.0`. |
| **issue moved between milestones after the tag** | Refused. Once `v3.41.0` exists its milestone is closed and its membership is immutable; moving an issue out of a closed milestone is the CB-2116 violation, and the message says so. |
| **milestone renamed** | CB-2114 goes red naming the rename. The rename *is* a release rename; if intended, rename the tag and the roadmap entries in the same change. |
| **a ticket whose work spans two releases** | Forbidden. A ticket belongs to exactly one release; work that spans is two tickets, and `pmat work` will say so. This is a modelling decision, stated so nobody discovers it as a bug. |

## 5. Staleness (D)

"Neither may be stale" is a symmetry claim. It cannot hold for *fields* — two systems
with independent clocks always disagree transiently. It can hold for *membership*.

### 5.1 Set predicate — tolerance ZERO

Let **R** = roadmap items with `status ∈ {planned, inprogress}`, and **G** = open issues
minus those labelled `no-roadmap`.

> `item ↦ item.github_issue` is a bijection R → G.

Three finding classes, reported separately because they have different fixes:

| class | today | fix |
|---|---|---|
| `ORPHAN-ROADMAP` — open item with no issue, or naming a closed/absent one | 48 | `work sync --yaml-to-github` |
| `ORPHAN-GITHUB` — open unlabelled issue with no item | ≈53 | `work sync --github-to-yaml` |
| `COLLISION` — two items naming one issue | **0 today**; 13 items named #612 until PMAT-721 resolved them in #1252 | **never auto-fixed** — a human |

### 5.2 Field predicate — tolerance TIME

For each matched pair, compare `title` and the state word. A disagreement is tolerated only
while `now − max(item.updated, issue.updated_at) < staleness_grace_minutes` (default 60). A
time bound, not a count: "up to 3 may disagree" means three chosen items may be wrong forever
and nobody chooses which three.

**Any edit to either side restarts that window, and that is accepted.** Measuring instead
from when a sync first SAW the disagreement was considered and rejected: it needs a
first-seen record that must be written, committed, read back and kept honest across every
clone and every CI runner, and a gate whose verdict depends on a state file that can be
absent, stale or wrong is a gate that cannot be judged from a clean clone. A timestamp bump
is also some evidence that a human touched the disagreement. Neither side of this trade is
free — the cost taken here is a window a touch can restart, instead of a new source of truth
that can lie. (Decided by quorum 2026-09-11, 4 of 5 seats; the dissent held that a window any
edit resets never expires — §11.2.)

The window covers every leg, not only a field disagreement. An issue opened less than
`staleness_grace_minutes` ago is **tolerated** — the rule PASSES and prints the count, the
same verdict a matched pair inside the window gets. It is emphatically not `not_measured`,
which §3.3 maps to FAIL: the sync that would mint its item cannot have run yet, and a rule
that turns master red the instant anyone opens an issue is a rule someone disables. The same holds for an item added without its
issue. Past the window it is a finding, and `pmat work sync` is the fixer. (Decided by
quorum 2026-09-11, 5 of 5 seats — one of the two unanimous decisions of the sixteen.) The code
covers the field-disagreement leg only; extending it to ORPHAN and MISSING is PMAT-1309.

### 5.3 `RR-COHERENCE` — which side wins

> **GitHub is authoritative for existence and open/closed state. The roadmap is
> authoritative for plan — `spec`, `acceptance_criteria`, `phases`, `priority`.
> `release` is held on the roadmap but decided by the milestone (§4.1, RR-RELEASE):
> `pmat work sync` projects it and is its only writer.**

Disjoint field sets, so there is never a merge conflict and never a prompt.

### 5.4 Landing green

CB-2115 could not land while 13 items collided on #612 — a gate red on arrival is a gate
someone disables. PMAT-721 resolved them in #1252 and 0 collide today; the sequence that
got there:

1. `pmat work sync` becomes real, `--check-only` first. It **reports**.
2. The 13 collisions are resolved **by a human under their own ticket**. Which of the 13
   is really #612 is a judgement, and a sync that guesses is how the 13 arose. This is a
   supervised one-off migration, not the gate-time auto-fix §5.1 forbids — the
   distinction the teamwork lane collapsed.
3. Only then does CB-2115 become a gate, and it lands green.

## 6. Spec review (E.1)

### 6.1 The artifact

The gate reads a **file**. It never invokes a model.

`docs/audits/spec-<slug>-review.json`:

```json
{
  "spec": "docs/specifications/goal-mode.md",
  "spec_sha256": "<sha256 of the file at review time>",
  "plan": { "tool": "claude-plan", "ref": "<path or conversation id>", "sha256": "…" },
  "lanes": [
    {"role": "quality",      "executor": "agy", "verdict": "PASS", "summary": "…"},
    {"role": "architecture", "executor": "agy", "verdict": "PASS", "summary": "…"},
    {"role": "security",     "executor": "agy", "verdict": "PASS", "summary": "…"},
    {"role": "crux",         "executor": "agy", "verdict": "PASS", "summary": "…"},
    {"role": "adversarial",  "executor": "agy", "verdict": "PASS", "summary": "…"}
  ],
  "agreed": true,
  "partial": false
}
```

`role` is a **closed set**: `quality | architecture | security | crux | adversarial |
vendor:<name>`. An unrecognised role is an error, not an extra lane — otherwise "six
reviewers" decays into six copies of one. For the same reason a closed-set role the spec
does not require (a vendor its front-matter does not name) and a second lane of one role
are errors too.

### 6.2 What CB-2111 checks, and what it buys

Checks, all offline: the file parses; `spec_sha256` equals the file's hash **now**;
`plan` is present with a `sha256` of 64 hex digits; every required role is present; every
lane is PASS; `agreed: false` and `partial: true` are red.

**What it buys, exactly:** it converts a skipped review from an *omission nobody can
see* into an *auditable lie somebody wrote down*. **What it does not buy:** any evidence
the review happened, or that the lanes were independent minds. `agentic-delivery-pmat.md
§5.3` already records a lane reporting four existing files as absent and labelling that
finding `measured`. This document's own §0 records a grill whose "two lanes" were one
lane and a retry.

That is a real but bounded purchase, and it ships on those terms.

### 6.3 Who produces it

`pmat spec review --record <json>` validates and stages. It does **not** produce.
Production is the `pmat-implement` bundle's job (a change tracked in *that* repo), or a
human's — it is JSON, and a hand-written six-role review with real findings is a
legitimate quorum. That is what "pmat does not depend on Claude or agy" means concretely.

## 7. Micro-enforcement matrix

| id | inv. | statement | falsifier (plant this → RED) | verdict reaches |
|---|---|---|---|---|
| **CB-2110** | E | every `active` spec names an open epic issue with ≥1 sub-issue | delete the `epic:` line | `traceability` → `gate` |
| **CB-2111** | E.1 | every `active` spec has a review artifact whose hash matches and whose roles all PASS | **append one space to the spec** | `traceability` → `gate` |
| **CB-2112** | A | every open item has a `github_issue`, open, whose number is the item's numeric tail | null one `github_issue` | `traceability` → `gate` |
| **CB-2113** | C | on a PR: every non-merge commit the PR adds carries `Pmat-Ticket: <id>` naming a real, NON-TERMINAL item; its `release` is compared with the open milestone and a LATER one is reported, not failed, until §11.2's milestone scheme exists (then it fails); on master: every non-merge commit in `v<latest>..HEAD` names a real item, any status | `git commit --allow-empty -m 'no trailer' --no-verify` | `traceability` → `gate` |
| **CB-2114** | B, F1 | every **`inprogress`** item has `release:`, its milestone exists, its issue is on it; a `planned` item with no `release:` is unscheduled, not a violation (§11.2 Q15) | remove the `release:` from an `inprogress` item | `traceability` → `gate` |
| **CB-2115** | D | the open sets are in bijection, and no matched pair has disagreed past the grace window | close one linked issue, leave the item open | `traceability` → `gate` |
| **CB-2116** | B, F2 | (a) every non-merge commit in a tag's range carries a trailer for a ticket of that release; (b) `count(v<latest>..master) ≤ max_untagged_commits + untagged_ci_slack` | move a merged ticket's `release` | `traceability` → `gate` |
| **CB-2117** | G | `docs/status/goal-ledger.md` matches what the generator computes now | hand-edit a row | `traceability` → `gate` |

All eight are `CheckSeverity::Error`. Anything less reports and never fails — the mistake
CB-2101/2102's own comments name.

### 7.1 The enforcement path

**Not** behind `Ladder gate — pmat comply`: that step is `continue-on-error`, is
currently red on four pre-existing failures, and unwedging it is a five-step sequence
documented in `.github/workflows/quality-gate.yml` that would redden a required context on master and every
open PR. Routing around it is not forum-shopping — it is refusing to make eight new rules
hostage to a deadlock they did not cause. The other 161 stay behind it and this document does
**not** claim to fix them (§8.11).

A new job in `.github/workflows/ci.yml`, wired exactly as `roadmap-validate` is:

```yaml
  traceability:
    name: traceability
    runs-on: ubuntu-latest
    timeout-minutes: 45
    steps:
      - uses: actions/checkout@v7
        with: { fetch-depth: 0, fetch-tags: true }
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --bin pmat --locked
      # CONTROL FIRST, as roadmap-validate does: prove the gate can fail.
      - name: control — a fixture with no trailer must be refused (exit 1)
        run: bash scripts/traceability-control.sh
      - name: the closed loop holds
        env: { GH_TOKEN: "${{ github.token }}" }
        run: ./target/debug/pmat comply check --checks CB-2110,CB-2111,CB-2112,CB-2113,CB-2114,CB-2115,CB-2116,CB-2117
```

added to **both** `gate`'s `needs:` and its result loop (the `gate` job in
`.github/workflows/ci.yml`). Both: the
comment above `roadmap-validate` is explicit that the loop is what fails.

`The result loop's `if [ "$result" != "success" ]` treats a `skipped` or `cancelled` job as a failure, so it also
fails the gate — the classic `if: always()` + `needs: skipped` hole is already closed
here, verified.

**`merge_group` is added to `.github/workflows/ci.yml`'s `on:` in the same change.** A merge queue is not
known to be enabled, but a gate that is not evaluated on the queue is a gate the queue
bypasses, and adding the trigger costs nothing.

### 7.2 Prerequisites

- **P1 — `comply check --checks <CSV>`.** Does not exist; AD-07's own acceptance test
  already names it. An id not in the list is `Skip(not selected)`, never absent — an
  absent check and a skipped check must not look alike. **An unknown id is an error**,
  because that is how a typo'd gate becomes a green gate.
- **P2 — `required::resolve` must union rulesets with branch protection**, and
  `.github/required-status-checks.txt` gains `gate` and loses its false sentence, in the
  same commit. A test asserts the union.

### 7.3 The PR lane, the pre-release lane, and what measuring them cost

A gate a human waits forty minutes for is a gate that gets bypassed, and every rule in §7
is paid for in that wait. This section records what was measured when that wait was
attacked — including the change that did not work, because the number that refuted it is
worth more than the one that suggested it.

**The suite, measured two ways.** 21,536 lib tests, one machine, one binary:

| runner | wall-clock | where |
|---|---|---|
| `cargo test --lib` | 1227.83s | what `ci / test` runs today |
| `cargo nextest run --lib` | **100.51s** | a 12.2x that is real — locally |

**And the switch that made CI slower.** `sovereign-ci.yml` sets `NEXTEST_TEST_THREADS=4`
in a container given `--cpus 8`; `cargo test` defaults to available parallelism, which is
8. Halving the workers while paying a process spawn per test loses more than isolation
wins. Measured on the pull request that tried it: **9,047 of 21,536 tests after 11 minutes
of running**, against 20.5 minutes for the entire suite under `cargo test`. The 12.2x is
unreachable here until that cap is raised, so `use_nextest` is `false` in `ci.yml`, with
the numbers beside it, and `scripts/pr-lane-control.sh` REFUSES a workflow that turns it
on while the cap stands. A speedup measured on a 32-core workstation is not a speedup;
it is a hypothesis about the runner.

**The win that needed no switch.** Two tests built an `AgentContextIndex` over the WHOLE
REPOSITORY, 160 seconds each. `test_make_cluster_item_basic` never read the result — its
only use was `is_err()`, and every assertion below it is on a struct literal. They now
build a one-file fixture, and all 24 `file_split` tests finish in **0.01s**: 320 seconds
off `ci / test` AND off `ci / coverage`, which runs the same tests serially.
`cargo test`'s thread pool had hidden them for as long as they existed; per-test
isolation made them visible in its first run, which is the argument for nextest restated
as the defect it found.

**Coverage, measured.** `cargo llvm-cov nextest --lib`: **85.25% of lines** (85.56% of
regions, 297,741 lines, 43,919 uncovered), in **469 seconds** — against the ~29 minutes
`ci / coverage` spends on `cargo llvm-cov test`. Two facts follow. The first is that on a
pull request that job's ONLY enforcement step is skipped (`Enforce coverage floor (OPT-IN
ratchet)`), there is no `.pmat/coverage-baseline.txt` in this repository, and `coverage_min`
is unset — so 29 minutes of every pull request measure something nothing reads. The second
is that the operator's standing target is 95%, and 85.25% is 29,000 lines short of it: a
gate set at 95% today would be red on arrival, which §5.4 forbids. The floor therefore
lands at the measured value as a ratchet that may only rise, and 95% is a programme with
tickets, not a number written into a config file.

**Two levers are upstream, in `paiml/.github`, and both are named in PMAT-1315:** the
thread cap above, and `skip_coverage` — an input documented as "Skip coverage job" that
fails the build, because the workflow's own gate requires `success` from `coverage` and
refuses `skipped` by design. The gate is right; the input cannot be used by any caller.

## 8. Honest limits

1. **The `ci / gate` hole.** It resolves into `paiml/.github`, unreadable here. This
   document routes around it; closing it is upstream.
2. **`enforce_admins: false`.** An admin merge bypasses every required context, `gate`
   included. Caught by nothing.
3. **Tags are mutable.** `git tag -d && git push --force` rewrites a boundary and
   CB-2116 would then measure the new one and pass. Needs org-level tag protection.
4. **Merge commits only.** `allowed_merge_methods` is restricted to `merge` (operator
   decision, 2026-09-09), so a branch's `Pmat-Ticket:` trailers survive onto master
   instead of being collapsed into one squashed message. This is what lets CB-2113
   measure **master**, not merely the PR — see §7. Setting it is a ruleset change on
   `13878864`; until it is made, CB-2113 measures the PR's commits only and says so in
   its output rather than pretending to cover master.
5. **Four rules describe more than the code does today**, each with its ticket: CB-2113's
   master leg reports `not_applicable` there for now and its release leg is unwritten
   (PMAT-1308), CB-2115's grace window covers the field-disagreement leg only and the rule
   still runs on a master push (PMAT-1309), the fork carve-out above is not yet implemented
   (PMAT-1310), and CB-2114 still binds every open item rather than the `inprogress` ones
   while CB-2110/2112/2114 still assert live state on a master push (PMAT-1312), and the
   cut's sweep-and-move of §10.3 is a description of `pmat goal`, which does not exist
   (step 9). Each is stated here rather than left for a reader to discover by running it.
6. **A trailer proves a claim, not the work.** `Pmat-Ticket: PMAT-999` on an unrelated
   diff passes CB-2113. Only a quorum reading the diff against the ticket defends this,
   and that is a skill, not a gate.
7. **Lane independence is unverifiable from a file** (§6.2), demonstrated twice in this
   document's own production.
8. **Publishing needs a token pmat does not hold.** `pmat goal` cuts tags and opens
   release PRs; it never publishes.
9. **Four rules need network.** On a fork they are `not_applicable` (§3.3); on a
   same-repo PR without a token they are `not_measured` and fail.
10. **The `gate` requirement lives in a ruleset this repository does not own.** An org
   admin can disable ruleset 13878864 and every rule here silently becomes decoration.
   P2 makes that *visible* — the manifest/live comparison goes red — but cannot prevent it.
11. **This does not un-NEUTER the other 161 rules.** It builds a second, working path for
    eight. Saying otherwise would be the kind of claim `pmat comply numeric-claims` exists
    to catch.
12. **The thresholds are in a file the pull request can edit.** `max_untagged_commits`,
    `untagged_ci_slack` and `staleness_grace_minutes` live in `pmat.toml` (§10.2), which is
    in the tree like any other file: a PR that raises them passes CB-2116(b) and widens
    CB-2115's window without cutting a release or syncing anything. Nothing in this
    document closes that, and pretending a number in a writable file is a bound would be
    the same mistake as `.pmat-metrics.toml`, whose budgets nothing reads. What the design
    does buy is that the edit is **in the diff**, named in `pmat.toml`, on a line whose
    only purpose is that threshold. The ratchet's answer — run the command, never read the
    number — is the shape a later rule would need.

## 9. Ticket ↔ work linking model — the cases the invariants miss

| case | handling |
|---|---|
| **revert** | A revert is work. It gets its own ticket, its own trailer, and its `notes` name the reverted ticket. Not exempt. |
| **dependency bumps (Dependabot/Renovate)** | One standing ticket per release, `PMAT-<n>: dependency bumps for X.Y.Z`, whose trailer bot commits carry. If that is not wired, the bot's issues carry `no-roadmap` and the exemption is greppable. Never an author allowlist — an allowlist is an unbounded hole. |
| **closed `wontfix`** | Terminal and legal: the item becomes `cancelled`, the issue closes. Both sides agree; CB-2115 is satisfied. |
| **security embargo** | The linkage is **never** exempt; the *content* is. A private advisory gets a roadmap item with a redacted title and the `security` label. **There is no bypass token** — the teamwork lane proposed one and it is rejected, because a bypass token is exactly the shape that neuters a gate. |
| **abandoned branch** | Nothing merges, so nothing is enforced. But an `inprogress` item whose issue has not moved past the grace window is a CB-2115 field finding, and `pmat goal` will not select it. |
| **a human who never runs `pmat`** | The `commit-msg` hook only exists if installed. This is precisely why CB-2113 is also a CI rule: CI is the backstop for the uninstalled hook. |
| **work spanning two releases** | Forbidden by §4.4. Split the ticket. |

## 10. `pmat goal` (G)

### 10.1 Division of labour

| pmat | the driver (a skill) |
|---|---|
| choose the next ticket; enforce preconditions; invoke the driver with a timeout; **re-run its claimed gate**; decide the release boundary; cut the tag; open the release PR; write the ledger; stop the line | implement the ticket; write tests; produce a receipt |

The driver is a key in `pmat.toml`, **never a path or a literal** —
`src/cli/handlers/kaizen_handler/fixing.rs:207` hardcodes `Command::new("claude")` with a
fixed tool list, and that is the defect not to repeat.

```toml
[goal]
# A ticket COUNT is not the Rust idiom and this repository does not behave like one:
# 11 releases in 24 days, 17 to 208 commits each, median 59. What Rust projects
# actually key on is (a) semver semantics read from what is IN the release, and
# (b) how much is sitting unreleased. Both are below; the ticket cap is opt-in and
# unset by default.
max_untagged_commits      = 60   # a round number just above this repo's MEDIAN of 59 (§14)
untagged_ci_slack         = 10   # CB-2116(b) fails only above max + slack, so the cut fires BEFORE the gate
max_release_age_hours     = 72
max_tickets_per_release   = 0    # 0 = no cap. Set it only if you want one
staleness_grace_minutes   = 60

[goal.drivers.pmat-implement]
cmd             = "claude"
args            = ["-p", "/paiml-implement {ticket}"]
timeout_minutes = 90
gate_cmd        = "pmat verify --format json"
```

### 10.2 Surface

```
pmat goal <driver> [<spec-path>]   # the loop
pmat goal plan                     # the queue and the cut points. No side effects.
pmat goal status | stop [--now] | ledger [--write]
```

### 10.3 Release boundary — when 100 tickets become 4 releases

**The version number is derived, never chosen.** Semver semantics are read from the
labels of the tickets in the release: any `breaking-change` → major; else any
`enhancement`/`feature` → minor; else patch. That is the Rust idiom — the number
describes what is in the release, so it cannot be argued about. The derived number is
what the release is called: when it differs from the milestone's title, the cut REFUSES and
names both numbers rather than editing the surface the next gate reads (§4.1).

**Settling the milestone is two moves, and both are what a release train does.** Every
ticket merged since the last tag and not yet scheduled is **swept into the release being
cut** — merging is what schedules work, and CB-2116(a) becomes true by construction rather
than by a PR-time refusal that would cost what Q15 bought. And every issue still open on
the milestone is **moved to the next one** before the tag is cut, so a cut fired by volume,
age, the explicit label or the cap does not fail on its own first step: a "verify the
drain" that only trigger 1 could ever satisfy made triggers 2 to 5 dead as written. Both
moves are `pmat work sync`'s, both are printed, and neither is silent: a human sees their
unscheduled ticket acquire a release when the cut lands, and an open ticket change
milestone with the cut named as the reason. (Q17 decided 4 of 5 — the dissent would fail
the PR instead, so work is scheduled before it merges; Q18 decided 5 of 5. §11.2.)

**The boundary** is evaluated after each ticket reaches `MERGED`; first to fire cuts:

1. **milestone drain** (primary) — zero open issues on the current milestone. Deterministic,
   visible in GitHub's UI, and schedulable by a human;
2. **unreleased volume** — `count(v<latest>..master) ≥ max_untagged_commits`. This is the
   Rust-shaped trigger: it asks how much is sitting unreleased, not how many tickets were
   closed. Default 60 is a round number just above this repository's own median
   commits-per-release of 59 (§14), measured over
   `v3.30.0..v3.40.0`, not a number chosen for the document;
3. **age** — `> max_release_age_hours` with ≥1 ticket closed;
4. **explicit** — an item labelled `release-boundary` reaching `completed`;
5. **ticket cap** — `≥ max_tickets_per_release`, only when that is set to a non-zero value.
   Off by default: "100 tickets → 4 releases" was the operator's illustration of the
   *shape*, and turning an illustration into a threshold is how an invented number becomes
   a measurement nobody can trace.

A cut: **settle the milestone** → bump the version → regenerate `CHANGELOG.md` **from the
trailers** in `v<prev>..HEAD` (this is *why* C is enforced — the changelog becomes
derived rather than written) → tag → push → open the release PR → close the milestone →
open the next → **stop and wait for CI**. It does not merge its own release PR and does
not publish.

### 10.4 Stop-the-line, and the flake carve-out

Stop on the first red. The grill's objection — that one flaky test wedges a 100-ticket
run, and that this repository has documented flakes (#1202, a `$TMPDIR`-dependent suite)
— is answered without becoming "retry until green", which the repository's rules forbid:

> A red is retried **once** if and only if it matches a **pre-registered** flake: an open
> issue labelled `flake` whose body names the failing test. Any other red stops the line
> immediately. A retry that then passes is still **recorded as a flake occurrence** in the
> ledger and does not clear the issue.

Bounded at one, gated on a human having written the flake down first, and it leaves a
count that grows — so a flake nobody fixes becomes visible rather than invisible.

Every other stop is immediate: driver non-zero, driver's claimed gate disagreeing with
pmat's re-run, timeout, driver absent (exit 3, never a fallback), CI red, lock held.
`.pmat/goal-stop` is **sticky**: the loop refuses to start while it exists, so a stop
cannot be silently undone by a fresh invocation.

## 11. Backlog — EV order

| # | lands | why here | size |
|---|---|---|---|
| 0 | **P2** ruleset-aware `required::resolve`, `gate` in the manifest, false sentence deleted, ledger regenerated | until this lands **every rule below lands NEUTERED**; afterwards a rule is enforced by joining one `needs:` list | S |
| 1 | **P1** `comply check --checks`, unknown id is an error | the job cannot select its rules without it | S |
| 2 | **CB-2113** + `scripts/traceability-control.sh` + the job wired into `gate` + `merge_group` | cheapest rule, no data-model change, proves the whole path end to end on one rule before eight ride on it | M |
| 3 | `release:` field + **`pmat work sync` made real** + the 13-collision migration | the fixer before the refusal | L |
| 4 | **CB-2115** (D) | lands green only after step 3 | M |
| 5 | **CB-2112** (A) + **CB-2114** (B/F1) | need the field and the sync | M |
| 6 | **CB-2110** (E) + retire CB-148 + front-matter on 43 specs | needs epics to exist; largest human step | L |
| 7 | **CB-2111** (E.1) + the artifact schema + `pmat spec review --record` | needs step 6 | M |
| 8 | **CB-2116** (B/F2) | needs one release cut under the new model to have data | M |
| 9 | **`pmat goal`** + `docs/status/goal-ledger.md` + **CB-2117** | last, deliberately | L |

CB-148 is **retired, not patched**: three defects in 100 lines (wrong scope; a predicate
that ignores the line under test; a severity that cannot fail) and it currently renders
as a *pass*, which is a confident wrong answer. `pmat comply check` prints
`CB-148: RETIRED — superseded by CB-2110` for one minor version so the id does not
vanish silently.

### 11.1 Artifacts this document proposes and that do not exist yet

`docs/status/goal-ledger.md` is a DELIVERABLE of step 9, not an existing file. Likewise
`pmat goal` does not parse today:

```console
$ pmat goal --help
error: unrecognized subcommand 'goal'
```

They are named here so a reader can grep for them and find nothing. Four entries have
left this list since it was written — `scripts/traceability-control.sh` (step 2, #1247),
`pmat work sync --check-only` (step 3, #1248), `scripts/roadmap-coherence-control.sh`
(step 4, PMAT-722) and `pmat spec review --record` (step 7, PMAT-1299) — each removed by
the step that delivered it, never before. A specification that
cites its own outputs as though they already existed is exactly the drift this document
is about — and every OTHER path and command in this file resolves, checked with the two
greps `CLAUDE.md` prescribes for this file class.

### 11.2 Decisions taken by quorum, 2026-09-11

Fourteen questions were open when this document failed its own five-role review (PMAT-1303).
None was settled by the author. Each was put to five independent seats that read a read-only
copy of this tree, chose from options they were given without being told which one was
drafted, and answered alone; the majority is the decision and every minority answer is kept.
The artifact is `docs/audits/quorum-PMAT-1307.json`. Where a decision changed a passage and
the reasoning is not obvious from the text, the passage carries the vote inline; the table in
this section is the complete record, and it is the one to trust.

The first round was **contaminated by the author**: the copy carried a roadmap file stating
the drafted answers as fact, and one seat cited it for nine of its fourteen votes. That seat
was discarded, the file stripped, and the three questions its removal left undecided were
re-run on a clean copy. Both rounds are in the artifact. The lesson is recorded here because
it is the same failure this document is about: evidence that says what you wanted it to say
is not evidence.

Four of the fourteen are programme decisions rather than text:

| | decision | seats | what it changes |
|---|---|---|---|
| **P1** | Exactly one open milestone at a time — the next release. Work not scheduled for it carries no milestone, and the rules read "no milestone" as "not yet scheduled". | 3 of 4 | PMAT-725's re-mint, and D3's release leg above |
| **P2** | Classify the 44 specs BEFORE minting epics: most are finished or superseded work and belong in `historical`/`superseded`; only the live set gets an epic. | 4 of 4 | PMAT-729, step 6 |
| **P3** | CB-2111 flips after that classification, over the live set only — not after reviewing all 44. | 4 of 4 | PMAT-1300, step 7 |
| **P4** | CB-2115 comes off the master push (§3.3). | 3 of 4 | PMAT-1309 |

Two more were opened by the five-role review of this revision, which failed 4 of 5 lanes on
contradictions the fourteen decisions had introduced, and were decided the same way:

| | decision | seats | what it changes |
|---|---|---|---|
| **Q15** | CB-2114's release leg binds `inprogress` items only. A `planned` item with no `release:` is unscheduled, which P1 makes the normal state of the backlog — not a violation. | **5 of 5** | §4.2, §7's CB-2114 row |
| **Q16** | On a master push, CB-2110, CB-2112 and CB-2114 judge only their file half; every live-state assertion is deferred to the scheduled run. | 4 of 5 | §3.3 |
| **Q17** | The cut SWEEPS: every ticket merged since the last tag and not yet scheduled is assigned to the release being cut, so CB-2116(a) is true by construction and Q15's unscheduled merge cannot wedge the tag. | 4 of 5 | §10.3 |
| **Q18** | The cut MOVES the remainder: open issues left on the milestone go to the next one before the tag, so triggers 2–5 are live instead of dead on "verify the drain". | **5 of 5** | §10.3 |

Q17 and Q18 came from the second five-role review, which failed the text 3 lanes to 2 on
them: two lanes found independently that an unscheduled merge — normal under Q15 — could
never satisfy CB-2116(a) at the tag, and one found that four of the five release triggers
fire exactly when the cut's first step must fail. Q15 is what P1 cost: with 97 open items and one open milestone, CB-2114 as written was red on
arrival for nearly the whole backlog, and neither the seats that decided P1 nor the author
saw it. Two review lanes found it independently. Q16 is the other half of P4 — the hole P4
closed for CB-2115 was open in three sibling rules, and closing it for one rule while three
others keep it would have been a fix in name only.

**P2 and P3 shrink the two largest human steps in §11** from "44 epics and 44 reviews" to
"classify, then epic and review what is left". That is the difference between a backlog a
person can finish and one that is quietly abandoned — which is what step 6 and step 7 have
been since 2026-09-10.

## 12. Do-not-do

- Do not put these rules behind the ladder step "for consistency". Consistency with a
  neutered gate is neutered.
- Do not add `epic:`, `milestone:` or `tag:` to the roadmap. §4.2.
- Do not let `not_applicable` absorb a missing token on a same-repo PR. §3.3.
- Do not auto-fix a `COLLISION`. §5.4.
- Do not add a security bypass token. §9.
- Do not let `pmat goal` merge its own release PR or publish. §10.3.
- Do not make any CB rule depend on a driver, a skill, or a model. §2.4.

## 13. Definition of done

1. `pmat comply ledger` shows the eight rules **ENFORCED**, and names `gate` among the
   roots.
2. Each of the eight has run RED against its §7 falsifier in CI at least once, and the
   run is linked from the receipt.
3. `scripts/traceability-control.sh` fails the job when the control defect is present and
   passes when it is not — both observed.
4. The 13 `#612` collisions are zero, by measurement.
5. `pmat work sync --check-only` reports 0 `ORPHAN-*` on master.
6. This specification carries its own `epic:` and its own review artifact — it must satisfy
   its own invariants, or it is a document arguing for a rule it does not follow.

## 14. Verification ledger

| claim | command | measured at `d3eef0198`, when this was written | now, at `f5eccdfb3` |
|---|---|---|---|
| ruleset requires unprefixed `gate` | `gh api …/rules/branches/master --jq …` | `["gate"]`, ruleset 13878864 active | unchanged |
| branch protection requires 5 others | `gh api …/branches/master/protection --jq …` | `ci / gate`, `feature-gate`, docs.rs, `pmat score`, `provable ladder` | unchanged |
| both are distinct checks on a PR | `gh pr view 1243 --json statusCheckRollup` | `["ci / gate","gate"]` | unchanged |
| `src/services/gate_effect/required.rs` cannot see rulesets | `grep -c "rules/branches" src/services/gate_effect/required.rs` | `0` | **3** — step 0 landed |
| the manifest omits `gate` | `cat .github/required-status-checks.txt` | 5 entries, none is `gate` | **6**, `gate` among them (PMAT-717) |
| rules and how many are enforced | `pmat comply ledger` | `rules: 157 / ENFORCED: 0 / NEUTERED: 157` | **163 / 2 / 161** — CB-2113, CB-2115 |
| a skipped `needs` fails the gate | the result loop in the `gate` job | `if [ "$result" != "success" ]` | unchanged |
| `merge_group` absent | `grep -rn merge_group .github/` | no matches | **present** — step 2 landed |
| items colliding on one issue | roadmap parse | 13 of 61 open items carry an issue; **1 distinct** | **0 open collide**; 14 items name #612, all completed (PMAT-721) |
| trailer coverage | `git log v<latest>..master --no-merges --format='%(trailers:key=Pmat-Ticket,valueonly)'` | 30 of 81 (37%) since `v3.39.0` | **168 of 179 (94%)** since `v3.40.0` |
| untagged commits | `git rev-list --count v<latest>..master` | 44 | **202** — above `max_untagged_commits`, and §10 is not built yet |
| release cadence: 11 tags in 24 days, median 59 commits | `git tag --sort=creatordate` + `git rev-list --count <prev>..<tag>` | 17 min, 208 max, 59 median over `v3.30.0..v3.40.0` | unchanged, the same window |
| CB-148 cannot fail | `src/models/comply_config_impls.rs:127` | `unconfigured` → `Warning` → `should_fail(...)` = `false` | **retired**, superseded by CB-2110 (PMAT-728) |

Every `now` value was re-measured by PMAT-1307 against `f5eccdfb3`. A row that moved is
a claim this document used to make and no longer does; a row marked unchanged was checked,
not assumed.

GOAL-MODE-SPEC-END
