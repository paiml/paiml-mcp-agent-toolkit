#!/usr/bin/env bash
# validate-pmat-book-selftest.sh — offline self-test for scripts/validate-pmat-book.sh (#1441).
#
#   fixture 1 (absent):  PMAT_BOOK_DIR names a directory that does not exist.
#                        The gate must exit non-zero and say NOTHING WAS
#                        VALIDATED. Before #1441 it exited 0 here, so a green
#                        'make validate-book' was no evidence of anything.
#   fixture 2 (present): PMAT_BOOK_DIR names an existing directory, so the gate
#                        gets PAST the absent-book check. It is an empty book,
#                        so what it does after that is not asserted here; only
#                        that it did not stop at the absent-book check.
#
# This never touches a real pmat-book checkout.
set -uo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
gate="${1:-${script_dir}/validate-pmat-book.sh}"
[ -f "${gate}" ] || { echo "validate-pmat-book-selftest: no gate at ${gate}" >&2; exit 2; }

work_dir="$(mktemp -d "${TMPDIR:-/tmp}/validate-pmat-book-selftest.XXXXXX")"
trap 'rm -rf "${work_dir:?}"' EXIT
fail=0

PMAT_BOOK_DIR="${work_dir}/absent" bash "${gate}" > "${work_dir}/absent.out" 2>&1
rc=$?
if [ "${rc}" -ne 0 ] && grep -q 'NOTHING WAS VALIDATED' "${work_dir}/absent.out"; then
  echo "ok    absent book -> exit ${rc}, NOTHING WAS VALIDATED"
else
  echo "FAIL  absent book -> exit ${rc} (want non-zero and NOTHING WAS VALIDATED)"
  fail=1
fi

mkdir -p "${work_dir}/present"
PMAT_BOOK_DIR="${work_dir}/present" PMAT_BOOK_TIMEOUT=5 bash "${gate}" > "${work_dir}/present.out" 2>&1
if grep -q 'pmat-book not found' "${work_dir}/present.out"; then
  echo "FAIL  present book -> stopped at the absent-book check"
  fail=1
else
  echo "ok    present book -> past the absent-book check"
fi

[ "${fail}" -eq 0 ] && echo "PASS  validate-pmat-book-selftest" || echo "FAIL  validate-pmat-book-selftest"
exit "${fail}"
