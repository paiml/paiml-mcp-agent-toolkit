#!/usr/bin/env bash
# roadmap-writer-gate.sh — no raw write lands under docs/roadmaps/ without the repository
# lock (PMAT-1385, #1385). The command `make gate`'s row for this check calls.
#
#   scripts/roadmap-writer-gate.sh               run the suite and judge it
#   scripts/roadmap-writer-gate.sh --judge LOG   judge a saved `cargo test` log, run nothing
#   scripts/roadmap-writer-gate.sh --self-test   prove the judge can fail, then exit
#
# The gate itself is a `--lib` suite, so the required `ci / gate` check (sovereign-ci's
# `cargo test --lib`) runs it on every pull request; this script is how a human or
# `make gate` runs the same 14 tests without the other 21,000.
#
# Why a script and not the bare cargo command: `cargo test -- <filter>` that matches
# NOTHING exits 0 ("0 passed"). A renamed test, a module that stopped being compiled,
# or a typo in the filter would read as a green gate. The judge therefore requires every
# test in REQUIRED below to appear by name as `... ok`, and refuses a log with a failure,
# a missing test, or no `test result:` line at all.
#
#   REQUIRED[0]  the tree: every raw write reaching a roadmap path is a RoadmapWriteLock
#                method — 11 raw writes on 8915fe3e6, 1 (write_migration) once the
#                service writers took the token, 0 after the fix
#   REQUIRED[1]  the allow-list is live: each allowed writer is still reached by the taint
#   REQUIRED[2..] the planted mutants (renamed binding, helper parameter, const, field,
#                return, clap default, format!, every sink kind) are caught, and prose,
#                reads and test code are not
#   the work_migrate_* tests: `pmat work migrate` waits for the lock and never opens the
#                generated roadmap.yaml in a repository with entries/
#
# Exit: 0 every required test ran and passed · 1 a test failed or is missing · 2 usage,
# or a log with no `test result:` line (the suite did not run, which is not a pass).
set -uo pipefail

REQUIRED=(
  roadmap_writer_gate_every_roadmap_write_in_the_tree_goes_through_the_lock_token
  roadmap_writer_gate_every_allowed_writer_is_still_reached
  roadmap_writer_gate_catches_the_migrate_shape
  roadmap_writer_gate_a_renamed_binding_is_still_caught
  roadmap_writer_gate_follows_consts_returns_fields_and_clap_defaults
  roadmap_writer_gate_sees_every_sink_kind
  roadmap_writer_gate_is_silent_on_unrelated_reads_and_tests
  roadmap_writer_gate_crosses_files
  roadmap_writer_gate_source_predicate
  work_migrate_waits_for_the_repository_lock
  work_migrate_keeps_every_byte_it_does_not_normalise
  work_migrate_in_fragment_mode_never_opens_the_aggregate
  work_migrate_in_fragment_mode_refuses_before_writing_anything
  work_migrate_in_fragment_mode_refuses_to_carry_text_after_the_last_row
)

# judge LOG — the verdict on one `cargo test` log.
judge() {
  local log=$1 name missing=0
  if ! grep -q '^test result: ' "$log"; then
    echo "roadmap-writer-gate: no 'test result:' line in $log — the suite did not run, which is not a pass" >&2
    return 2
  fi
  if grep -qE '^test .* \.\.\. FAILED$' "$log" || grep -qE '^test result: FAILED' "$log"; then
    grep -E '^test .* \.\.\. FAILED$' "$log" >&2
    echo "roadmap-writer-gate: RED — a required test failed" >&2
    return 1
  fi
  for name in "${REQUIRED[@]}"; do
    if ! grep -qE "^test (.*::)?${name} \.\.\. ok$" "$log"; then
      echo "roadmap-writer-gate: RED — ${name} did not run (renamed, unregistered, or filtered out)" >&2
      missing=$((missing + 1))
    fi
  done
  if [ "$missing" -gt 0 ]; then
    return 1
  fi
  echo "roadmap-writer-gate: GREEN — ${#REQUIRED[@]} required tests ran and passed"
  return 0
}

self_test() {
  local dir rc fails=0 name
  dir=$(mktemp -d) || exit 2
  trap 'rm -rf "${dir:?}"' RETURN
  arm() {   # arm LABEL WANT LOGFILE
    judge "$3" >/dev/null 2>&1
    rc=$?
    if [ "$rc" -ne "$2" ]; then
      echo "self-test: $1 exited $rc, want $2" >&2
      fails=$((fails + 1))
    fi
  }
  : >"$dir/all-ok.log"
  for name in "${REQUIRED[@]}"; do echo "test cli::handlers::work_handlers::x::${name} ... ok" >>"$dir/all-ok.log"; done
  echo "test result: ok. ${#REQUIRED[@]} passed; 0 failed" >>"$dir/all-ok.log"
  arm "every required test ok (control)" 0 "$dir/all-ok.log"

  echo "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 21903 filtered out" >"$dir/vacuous.log"
  arm "a filter that matched nothing" 1 "$dir/vacuous.log"

  grep -v -e "${REQUIRED[0]}" "$dir/all-ok.log" >"$dir/missing.log"
  arm "the tree test missing" 1 "$dir/missing.log"

  sed "s/${REQUIRED[1]} \.\.\. ok/${REQUIRED[1]} ... FAILED/" "$dir/all-ok.log" >"$dir/failed.log"
  arm "a required test FAILED" 1 "$dir/failed.log"

  echo 'error: could not compile pmat (lib test)' >"$dir/no-result.log"
  arm "a build that never ran the suite" 2 "$dir/no-result.log"

  if [ "$fails" -ne 0 ]; then
    echo "roadmap-writer-gate: self-test FAILED ($fails arm(s)) — the judge cannot be trusted" >&2
    return 2
  fi
  echo "roadmap-writer-gate: self-test ok (5 arms)"
  return 0
}

case "${1:-}" in
  --self-test) self_test; exit $? ;;
  --judge)
    [ $# -eq 2 ] || { echo "usage: $0 [--judge LOG | --self-test]" >&2; exit 2; }
    judge "$2"; exit $? ;;
  "") ;;
  *) echo "usage: $0 [--judge LOG | --self-test]" >&2; exit 2 ;;
esac

self_test || exit $?
log=$(mktemp) || exit 2
trap 'rm -f "$log"' EXIT
cargo test --lib --locked -- roadmap_writer_gate work_migrate_ 2>&1 | tee "$log"
judge "$log"
