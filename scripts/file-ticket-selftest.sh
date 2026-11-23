#!/usr/bin/env bash
# file-ticket-selftest.sh — offline self-test for scripts/file-ticket.sh (GH-1808).
# A stub `gh` (FILE_TICKET_GH seam) logs every call and returns canned JSON.
#   arm 1  no --parent             -> exit 2, zero gh calls
#   arm 2  parent has no milestone -> exit 2, no create call
#   arm 3  complete parent         -> create carries milestone + `triaged`,
#                                      sub-issue link is made, exit 0
#   arm 4  link fails              -> exit 1 (a bare ticket is never reported as filed)
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tool="${script_dir}/file-ticket.sh"
[ -f "${tool}" ] || { echo "file-ticket-selftest: ${tool} missing" >&2; exit 1; }

work="$(mktemp -d "${TMPDIR:-/tmp}/file-ticket-selftest.XXXXXX")"
trap 'rm -rf "${work}"' EXIT
log="${work}/gh.log"

cat >"${work}/gh" <<'STUB'
#!/usr/bin/env bash
echo "$*" >>"${STUB_LOG}"
case "$*" in
  *"-X POST"*sub_issues*) [ "${STUB_LINK_FAIL:-0}" = 1 ] && exit 1; echo '{"number":900}' ;;
  *"-X POST"*) echo '{"id":424242,"number":901}' ;;
  *issues/700/sub_issues*) echo '[{"number":901}]' ;;
  *issues/700*) echo "{\"number\":700,\"milestone\":${STUB_MILESTONE}}" ;;
  *) echo '{}' ;;
esac
STUB
chmod +x "${work}/gh"
export FILE_TICKET_GH="${work}/gh" STUB_LOG="${log}"

fail=0
check() { # name want got
  if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1 (want exit $2, got $3)"; fail=1; fi
}
run() { : >"${log}"; set +e; "$@" >"${work}/out" 2>&1; rc=$?; set -e; }

good_ms='{"number":7,"title":"3.45.1"}'

STUB_MILESTONE="${good_ms}" run bash "${tool}" --repo o/r --title t --body b
check "arm1 no parent refuses" 2 "${rc}"
[ ! -s "${log}" ] || { echo "FAIL arm1 made gh calls"; fail=1; }

STUB_MILESTONE=null run bash "${tool}" --repo o/r --parent 700 --title t --body b
check "arm2 parent without milestone refuses" 2 "${rc}"
if grep -q -- '-X POST' "${log}"; then echo "FAIL arm2 created a ticket"; fail=1; fi

STUB_MILESTONE="${good_ms}" run bash "${tool}" --repo o/r --parent 700 --title t --body b --label kind:code
check "arm3 complete filing" 0 "${rc}"
grep -q 'milestone=7' "${log}" || { echo "FAIL arm3 no milestone on create"; fail=1; }
grep -q 'labels\[\]=triaged' "${log}" || { echo "FAIL arm3 no triaged label"; fail=1; }
grep -q 'labels\[\]=kind:code' "${log}" || { echo "FAIL arm3 extra label dropped"; fail=1; }
grep -q 'issues/700/sub_issues.*sub_issue_id=424242' "${log}" || { echo "FAIL arm3 no sub-issue link"; fail=1; }

STUB_LINK_FAIL=1 STUB_MILESTONE="${good_ms}" run bash "${tool}" --repo o/r --parent 700 --title t --body b
check "arm4 failed link is not success" 1 "${rc}"

exit "${fail}"
