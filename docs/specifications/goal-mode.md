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
- the teamwork lane silently swapped this specification's primary key (§2.2) and the
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

`pmat comply ledger` reports **157 CB rules, 0 ENFORCED, 157 NEUTERED**, and names one
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
   authority and make the others projections (§2.2). A second source of truth is a
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
| `EPIC-OPEN` | an open issue labelled `epic` whose body names the spec path | GitHub |
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
where a token exists.

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
/// cancelled item means the item predates release tracking; `None` on an open
/// item is a CB-2114 violation.
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

Epic membership is read from GitHub's native **sub-issue** relation, with an `Epic: #N`
body line as a fallback recorded as `grounding: "asserted"` — the same distinction the
quorum artifacts already draw.

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
| `COLLISION` — two items naming one issue | **13 items all name #612** | **never auto-fixed** — a human |

### 5.2 Field predicate — tolerance TIME

For each matched pair, compare `title` and the state word. A disagreement is tolerated
only while `now − max(item.updated, issue.updated_at) < staleness_grace_minutes`
(default 60). A time bound, not a count: "up to 3 may disagree" means three chosen
items may be wrong forever and nobody chooses which three.

### 5.3 `RR-COHERENCE` — which side wins

> **GitHub is authoritative for existence and open/closed state. The roadmap is
> authoritative for plan — `spec`, `acceptance_criteria`, `phases`, `priority`,
> `release`.**

Disjoint field sets, so there is never a merge conflict and never a prompt.

### 5.4 Landing green

CB-2115 cannot land while 13 items collide on #612 — a gate red on arrival is a gate
someone disables. Sequence:

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
reviewers" decays into six copies of one.

### 6.2 What CB-2111 checks, and what it buys

Checks, all offline: the file parses; `spec_sha256` equals the file's hash **now**;
`plan` is present with a non-empty `sha256`; every required role is present; every lane
is PASS; `partial: true` is red.

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
| **CB-2113** | C | every non-merge commit in the PR carries `Pmat-Ticket: <id>` naming a real, non-terminal item | `git commit --allow-empty -m 'no trailer' --no-verify` | `traceability` → `gate` |
| **CB-2114** | B, F1 | every open item has `release:`, its milestone exists, its issue is on it | remove one `release:` | `traceability` → `gate` |
| **CB-2115** | D | the open sets are in bijection, and no matched pair has disagreed past the grace window | close one linked issue, leave the item open | `traceability` → `gate` |
| **CB-2116** | B, F2 | (a) every commit in a tag's range carries a trailer for a ticket of that release; (b) `count(v<latest>..master) ≤ max_untagged_commits` | move a merged ticket's `release` | `traceability` → `gate` |
| **CB-2117** | G | `docs/status/goal-ledger.md` matches what the generator computes now | hand-edit a row | `traceability` → `gate` |

All eight are `CheckSeverity::Error`. Anything less reports and never fails — the mistake
CB-2101/2102's own comments name.

### 7.1 The enforcement path

**Not** behind `Ladder gate — pmat comply`: that step is `continue-on-error`, is
currently red on four pre-existing failures, and unwedging it is a five-step sequence
documented in `.github/workflows/quality-gate.yml` that would redden a required context on master and every
open PR. Routing around it is not forum-shopping — it is refusing to make eight new rules
hostage to a deadlock they did not cause. The 157 stay behind it and this document does
**not** claim to fix them (§8.10).

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

added to **both** `gate`'s `needs:` and its result loop (`.github/workflows/ci.yml:31-50`). Both: the
comment above `roadmap-validate` is explicit that the loop is what fails.

`.github/workflows/ci.yml:43` is `if [ "$result" != "success" ]`, so a `skipped` or `cancelled` job also
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

## 8. Honest limits

1. **The `ci / gate` hole.** It resolves into `paiml/.github`, unreadable here. This
   document routes around it; closing it is upstream.
2. **`enforce_admins: false`.** An admin merge bypasses every required context, `gate`
   included. Caught by nothing.
3. **Tags are mutable.** `git tag -d && git push --force` rewrites a boundary and
   CB-2116 would then measure the new one and pass. Needs org-level tag protection.
4. **Squash merges erase trailers.** CB-2113 therefore measures the **PR's own commits**,
   where they still exist and can still be fixed; CB-2116's tag-range leg is what would
   notice a squashed master commit with no trailer. The durable fix is restricting
   `allowed_merge_methods` to `merge`, which is an org action and is **recommended here,
   not assumed**.
5. **A trailer proves a claim, not the work.** `Pmat-Ticket: PMAT-999` on an unrelated
   diff passes CB-2113. Only a quorum reading the diff against the ticket defends this,
   and that is a skill, not a gate.
6. **Lane independence is unverifiable from a file** (§6.2), demonstrated twice in this
   document's own production.
7. **Publishing needs a token pmat does not hold.** `pmat goal` cuts tags and opens
   release PRs; it never publishes.
8. **Four rules need network.** On a fork they are `not_applicable` (§3.3); on a
   same-repo PR without a token they are `not_measured` and fail.
9. **The `gate` requirement lives in a ruleset this repository does not own.** An org
   admin can disable ruleset 13878864 and every rule here silently becomes decoration.
   P2 makes that *visible* — the manifest/live comparison goes red — but cannot prevent it.
10. **This does not un-NEUTER the other 157 rules.** It builds a second, working path for
    eight. Saying otherwise would be the kind of claim `pmat comply numeric-claims` exists
    to catch.

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
max_tickets_per_release   = 25   # policy from the operator's "100 tickets → 4 releases"
max_release_age_hours     = 72
max_untagged_commits      = 60   # CB-2116(b); 44 today
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

Evaluated after each ticket reaches `MERGED`; first to fire cuts:

1. **milestone drain** (primary) — zero open issues on the current milestone;
2. **ticket budget** — `≥ max_tickets_per_release` closed since the last tag;
3. **age** — `> max_release_age_hours` with ≥1 ticket closed;
4. **explicit** — an item labelled `release-boundary` reaching `completed`.

A cut: verify the drain → bump the version → regenerate `CHANGELOG.md` **from the
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

`scripts/traceability-control.sh` and `docs/status/goal-ledger.md` are DELIVERABLES of
steps 2 and 9, not existing files. Likewise `pmat goal`, `pmat spec review --record` and
`pmat work sync --check-only` do not parse today:

```console
$ pmat goal --help
error: unrecognized subcommand 'goal'
```

They are named here so a reader can grep for them and find nothing. A specification that
cites its own outputs as though they already existed is exactly the drift this document
is about — and every OTHER path and command in this file resolves, checked with the two
greps `CLAUDE.md` prescribes for this file class.

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

| claim | command | measured |
|---|---|---|
| ruleset requires unprefixed `gate` | `gh api …/rules/branches/master --jq …` | `["gate"]`, ruleset 13878864 active |
| branch protection requires 5 others | `gh api …/branches/master/protection --jq …` | `ci / gate`, `feature-gate`, docs.rs, `pmat score`, `provable ladder` |
| both are distinct checks on a PR | `gh pr view 1243 --json statusCheckRollup` | `["ci / gate","gate"]` |
| `src/services/gate_effect/required.rs` cannot see rulesets | `grep -c "rules/branches" src/services/gate_effect/required.rs` | `0` |
| the manifest omits `gate` | `cat .github/required-status-checks.txt` | 5 entries, none is `gate` |
| 157 rules, 0 enforced | `pmat comply ledger` | `rules: 157 / ENFORCED: 0 / NEUTERED: 157` |
| a skipped `needs` fails the gate | `sed -n '43p' .github/workflows/ci.yml` | `if [ "$result" != "success" ]` |
| `merge_group` absent | `grep -rn merge_group .github/` | no matches |
| 13 items collide on #612 | roadmap parse | 13 of 61 open items carry an issue; **1 distinct** |
| trailer coverage | `git log v3.39.0..master --no-merges --format='%(trailers:key=Pmat-Ticket,valueonly)'` | 30 of 81 (37%) |
| untagged commits | `git rev-list --count v3.40.0..master` | 44 |
| CB-148 cannot fail | `src/models/comply_config_impls.rs:127` | `unconfigured` → `Warning` → `should_fail(...)` = `false` |

GOAL-MODE-SPEC-END
