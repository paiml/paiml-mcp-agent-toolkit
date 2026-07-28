# Roadmap YAML Schema

Reference for `docs/roadmaps/roadmap.yaml`, the file read by `pmat work`.

Quick checks:

```bash
pmat work validate        # parse the roadmap and list every broken row
pmat work list-statuses   # print the status vocabulary and its aliases
pmat work migrate         # auto-fix common issues
```

Source of truth: `src/models/roadmap_types.rs` (structs, `ItemType`, `Priority`) and
`src/models/roadmap_status.rs` (`ItemStatus`).

## The one asymmetry that catches people

`status` is **lenient**: case-insensitive, hyphen/underscore-insensitive, and it accepts
a broad alias set. `item_type` and `priority` are **strict**: exact lowercase only, no
aliases, no case folding.

```yaml
status: In-Progress     # ok — normalizes to inprogress
status: WIP             # ok — alias
item_type: Task         # ERROR — must be lowercase `task`
priority: High          # ERROR — must be lowercase `high`
```

All three report a nearest-match hint on a typo (`item_type: bugg` suggests `bug`),
and enumerate their accepted values.

The parser is serde-based, so any single load stops at the **first** violation. Do not
conform a large roadmap that way: run `pmat work validate`, which re-checks each row
independently and lists every broken one, with its index and id, in a single pass.

## Top level

| Field | Type | Required | Default |
|---|---|---|---|
| `roadmap_version` | string | **yes** | — |
| `github_enabled` | bool | no | `true` |
| `github_repo` | string | no | none |
| `roadmap` | list of items | no | `[]` |

`github_enabled` accepts a native bool or the quoted strings `"true"` / `"false"`.

## Roadmap item

| Field | Type | Required | Default |
|---|---|---|---|
| `id` | string | **yes** | — |
| `title` | string | **yes** | — |
| `status` | status | **yes** | — |
| `item_type` | item type | no | `task` |
| `priority` | priority | no | `medium` |
| `github_issue` | integer | no | none |
| `assigned_to` | string | no | none |
| `created` / `updated` | ISO 8601 string | no | `1970-01-01T00:00:00Z` |
| `spec` | path | no | none |
| `estimated_effort` | string | no | none |
| `notes` | string | no | none |
| `acceptance_criteria` | list of strings | no | `[]` |
| `labels` | list of strings | no | `[]` |
| `phases` | list of phases | no | `[]` |
| `subtasks` | list of subtasks | no | `[]` |

Unknown fields are **silently ignored** at every level, so extra keys such as
`description` or `implementation` are safe for backward compatibility.

### Phase

`name` and `status` are required; `estimated_effort` is optional; `completion`
defaults to `0`.

### Subtask

`id`, `title`, and `status` are required; `github_issue` is optional; `completion`
defaults to `0`.

## `item_type` — closed vocabulary, exact lowercase

```
task  epic  bug  feature  enhancement  documentation  refactor
```

No aliases. A value outside this set is an error; `task` is the default if the field
is omitted entirely.

## `priority` — closed vocabulary, exact lowercase

```
low  medium  high  critical
```

No aliases. Defaults to `medium`.

## `status` — closed vocabulary with aliases

Input is lowercased and stripped of `-` and `_` before matching, so `In-Progress`,
`in_progress`, and `InProgress` all resolve to `inprogress`.

| Canonical | Accepted aliases |
|---|---|
| `planned` | `todo`, `open`, `pending`, `new` |
| `inprogress` | `wip`, `active`, `started`, `working` |
| `blocked` | `stuck`, `waiting`, `on-hold` |
| `review` | `reviewing`, `pr`, `pending-review` |
| `completed` | `done`, `finished`, `closed` |
| `cancelled` | `canceled`, `dropped`, `wontfix` |

Unknown values produce a "did you mean" suggestion via Levenshtein distance.

### Status transitions

`pmat work` enforces this adjacency matrix when changing status:

```
planned    → inprogress, cancelled
inprogress → blocked, review, completed
blocked    → inprogress
review     → inprogress, completed
completed  → (terminal)
cancelled  → (terminal)
```

## Minimal valid roadmap

```yaml
roadmap_version: "1.0"
github_enabled: false
roadmap:
  - id: "PROJ-1"
    title: "Do the thing"
    status: planned
```

## Fully populated item

```yaml
roadmap_version: "1.0"
github_enabled: true
github_repo: "owner/repo"
roadmap:
  - id: "EPIC-001"
    github_issue: 42
    item_type: epic
    title: "Ship the parser"
    status: inprogress
    priority: high
    assigned_to: "@someone"
    created: "2026-01-01T00:00:00Z"
    updated: "2026-01-02T00:00:00Z"
    spec: docs/specifications/parser.md
    estimated_effort: "3d"
    labels: [parser, v2]
    acceptance_criteria:
      - "Round-trips the corpus"
    phases:
      - name: "Design"
        status: completed
        completion: 100
    subtasks:
      - id: "EPIC-001-a"
        title: "Lexer"
        status: done
        completion: 100
    notes: |
      Free-form markdown.
```
