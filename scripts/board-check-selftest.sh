#!/usr/bin/env bash
# board-check-selftest.sh — offline self-test for scripts/board-check.sh (PMAT-693).
#
# Builds two fixtures in a temp dir: a roadmap.yaml with 3 tickets plus a
# matching dispositions ledger.
#   Fixture 1 (gap):     one ticket is planned and absent from the ledger.
#                         board-check.sh --offline must exit 1 and name
#                         exactly that ticket id, nothing else.
#   Fixture 2 (covered):  every non-completed ticket has an enactable
#                         disposition in the ledger. board-check.sh --offline
#                         must exit 0.
#
# This script never touches the real repo's roadmap.yaml or ledger, and never
# calls gh (--offline skips the PR/issue legs entirely).
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
board_check="${script_dir}/board-check.sh"

if [ ! -x "${board_check}" ]; then
  echo "board-check-selftest: ${board_check} not found or not executable" >&2
  exit 1
fi

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/board-check-selftest.XXXXXX")"
trap 'rm -rf "${work_dir}"' EXIT

write_roadmap() {
  local path="$1"
  cat >"${path}" <<'EOF'
roadmap_version: '1.0'
github_enabled: true
github_repo: paiml/paiml-mcp-agent-toolkit
roadmap:
- id: PMAT-9901
  github_issue: null
  item_type: task
  title: 'fixture: completed ticket'
  status: completed
  priority: medium
  assigned_to: null
  created: 2026-01-01T00:00:00Z
  updated: 2026-01-01T00:00:00Z
  spec: null
  acceptance_criteria: []
  phases: []
  subtasks: []
  estimated_effort: null
  labels: []
  notes: null
- id: PMAT-9902
  github_issue: null
  item_type: task
  title: 'fixture: planned ticket, covered by the ledger'
  status: planned
  priority: medium
  assigned_to: null
  created: 2026-01-01T00:00:00Z
  updated: 2026-01-01T00:00:00Z
  spec: null
  acceptance_criteria: []
  phases: []
  subtasks: []
  estimated_effort: null
  labels: []
  notes: null
- id: PMAT-9903
  github_issue: null
  item_type: task
  title: 'fixture: planned ticket, planted gap (absent from the ledger)'
  status: planned
  priority: medium
  assigned_to: null
  created: 2026-01-01T00:00:00Z
  updated: 2026-01-01T00:00:00Z
  spec: null
  acceptance_criteria: []
  phases: []
  subtasks: []
  estimated_effort: null
  labels: []
  notes: null
EOF
}

gap_roadmap="${work_dir}/roadmap-gap.yaml"
gap_ledger="${work_dir}/dispositions-gap.json"
write_roadmap "${gap_roadmap}"
cat >"${gap_ledger}" <<'EOF'
[
  {
    "kind": "ticket",
    "id": "PMAT-9902",
    "title": "fixture: planned ticket, covered by the ledger",
    "disposition": "defer(3.41.0)",
    "rationale": "fixture row",
    "hrq": false,
    "links": []
  }
]
EOF

covered_roadmap="${work_dir}/roadmap-covered.yaml"
covered_ledger="${work_dir}/dispositions-covered.json"
write_roadmap "${covered_roadmap}"
cat >"${covered_ledger}" <<'EOF'
[
  {
    "kind": "ticket",
    "id": "PMAT-9902",
    "title": "fixture: planned ticket, covered by the ledger",
    "disposition": "defer(3.41.0)",
    "rationale": "fixture row",
    "hrq": false,
    "links": []
  },
  {
    "kind": "ticket",
    "id": "PMAT-9903",
    "title": "fixture: planned ticket, planted gap (absent from the ledger)",
    "disposition": "defer(3.41.0)",
    "rationale": "fixture row",
    "hrq": false,
    "links": []
  }
]
EOF

fail=0

echo "board-check-selftest: fixture 1 (planted gap: PMAT-9903 not in ledger)"
gap_out="$(mktemp "${TMPDIR:-/tmp}/board-check-selftest-gap-out.XXXXXX")"
gap_exit=0
"${board_check}" --offline --ledger "${gap_ledger}" --roadmap "${gap_roadmap}" >"${gap_out}" 2>&1 || gap_exit=$?
cat "${gap_out}"

if [ "${gap_exit}" -ne 1 ]; then
  echo "board-check-selftest: FAIL — expected exit 1 on the gap fixture, got ${gap_exit}" >&2
  fail=1
fi

gap_lines="$(grep -c '^GAP ' "${gap_out}" || true)"
if [ "${gap_lines}" -ne 1 ]; then
  echo "board-check-selftest: FAIL — expected exactly 1 GAP line on the gap fixture, got ${gap_lines}" >&2
  fail=1
fi

if ! grep -qF 'GAP ticket PMAT-9903 not in ledger' "${gap_out}"; then
  echo "board-check-selftest: FAIL — expected 'GAP ticket PMAT-9903 not in ledger', not found" >&2
  fail=1
fi

if grep -q 'PMAT-9901\|PMAT-9902' "${gap_out}"; then
  echo "board-check-selftest: FAIL — output named a ticket other than the planted gap" >&2
  fail=1
fi

rm -f "${gap_out}"

echo "board-check-selftest: fixture 2 (fully covered)"
covered_out="$(mktemp "${TMPDIR:-/tmp}/board-check-selftest-covered-out.XXXXXX")"
covered_exit=0
"${board_check}" --offline --ledger "${covered_ledger}" --roadmap "${covered_roadmap}" >"${covered_out}" 2>&1 || covered_exit=$?
cat "${covered_out}"

if [ "${covered_exit}" -ne 0 ]; then
  echo "board-check-selftest: FAIL — expected exit 0 on the covered fixture, got ${covered_exit}" >&2
  fail=1
fi

if grep -q '^GAP ' "${covered_out}"; then
  echo "board-check-selftest: FAIL — covered fixture reported a GAP" >&2
  fail=1
fi

rm -f "${covered_out}"

if [ "${fail}" -ne 0 ]; then
  echo "board-check-selftest: FAILED"
  exit 1
fi

echo "board-check-selftest: PASSED"
