#!/usr/bin/env bash
# ont11-audit — ONT-001 row ONT-11 (R-21, F-31; pmat#1422) acceptance, run against the binary:
#   (a) CB-1201 reads pv's exit code: 2 (Unknown) and 3 (error) are Fail carrying pv's own
#       `decline:` / `error:` line, pv absent under contracts/ is Fail `decline: pv not found`,
#       and `pmat comply check` exits 1; a judged pass stays Pass; no contracts/ stays Skip.
#   (b) `pmat comply check --list --format json` is a JSON array of {id, group, name} that holds
#       CB-1201 and a rule named exactly `contracts-armed-gates-monotone`.
#   (c) CB-2118 fails a branch that drops an armed gate or shape since its merge-base, naming it;
#       passes a branch that keeps them; skips with `decline: no contracts/`.
#   F-31: no source file under src/ uses pv's library.
#   PMAT=<path to the binary under test> scripts/ont11-audit.sh
# Exit 0 = every arm green; 1 = an arm red; 2 = harness failure.
set -uo pipefail
# --self-test: the audit must be able to fail. Run it against a pmat that does nothing and
# require RED; a checker that passes on a no-op binary checks nothing.
if [ "${1:-}" = "--self-test" ]; then
  S=$(mktemp -d); printf '#!/bin/sh\nexit 0\n' > "$S/pmat"; chmod +x "$S/pmat"
  if PMAT="$S/pmat" bash "$0" >/dev/null 2>&1; then echo "self-test FAILED: the audit passed against a pmat that does nothing"; rm -rf "$S"; exit 1; fi
  rm -rf "$S"; echo "self-test OK: the audit is RED against a pmat that does nothing"; exit 0
fi
PMAT=${PMAT:-}
if [ -z "$PMAT" ] || ! command -v "$PMAT" >/dev/null 2>&1; then echo "PMAT='$PMAT' is not an executable" >&2; exit 2; fi
case "$PMAT" in */*) ;; *) echo "PMAT must be a path, not a bare name" >&2; exit 2;; esac
command -v jq >/dev/null || { echo "jq is required" >&2; exit 2; }
REPO=$(git -C "$(dirname "$0")" rev-parse --show-toplevel 2>/dev/null) || exit 2
red=0; arm(){ if [ "$2" = 0 ]; then echo "  ✓ $1"; else echo "  ✗ $1"; red=1; fi; }
echo "ont11-audit (#1422) — $($PMAT --version 2>/dev/null | head -1)"
T=$(mktemp -d); [ -n "$T" ] && [ "$T" != "/" ] || exit 2
[[ "$T" == /* && "$T" != *..* ]] || exit 2
cleanup() { if [ -n "${T:-}" ] && [ "$T" != "/" ] && [ -d "$T" ]; then find "$T" -depth -delete 2>/dev/null; fi; }
trap cleanup EXIT
unset GITHUB_BASE_REF

# ── (a) CB-1201 against a stub pv ───────────────────────────────────────────
mkdir -p "$T/r/contracts" "$T/empty" "$T/bare"
# A project with no contracts/: a manifest, so comply has a project to judge.
printf '[package]\nname = "bare"\nversion = "0.1.0"\nedition = "2021"\n' > "$T/bare/Cargo.toml"
printf 'metadata: {version: "1.0.0"}\n' > "$T/r/contracts/x.yaml"
stub() { mkdir -p "$T/pv$1"; printf '#!/bin/sh\n%s\nexit %s\n' "$2" "$1" > "$T/pv$1/pv"; chmod +x "$T/pv$1/pv"; }
stub 2 'echo "note: shape unknown" >&2; echo "decline: NotArmed" >&2; echo "{\"passed\":true}"'
stub 3 'echo "error: contracts/x.yaml: bad yaml" >&2'
stub 0 'echo "{\"passed\":true,\"findings\":[]}"'
cb1201() { # <PATH> <dir> → rc in $rc, the CB-1201 row in $T/row
  PATH="$1" "$PMAT" comply check --path "$2" --checks CB-1201 --format json > "$T/out.json" 2>/dev/null; rc=$?
  jq -c '[.checks[]|select(.name|startswith("CB-1201:"))]' "$T/out.json" > "$T/row" 2>/dev/null || echo '[]' > "$T/row"
}
row_is() { jq -e --arg s "$1" --arg m "$2" 'length==1 and .[0].status==$s and (.[0].message|contains($m))' "$T/row" >/dev/null; }

cb1201 "$T/pv2:$PATH" "$T/r"
[ "$rc" = 1 ] && row_is Fail "decline: NotArmed"
arm "1  pv exit 2 is Fail with pv's decline: line verbatim, even beside passed:true; exit 1 (exit $rc)" $?
cb1201 "$T/pv3:$PATH" "$T/r"
[ "$rc" = 1 ] && row_is Fail "error: contracts/x.yaml: bad yaml"
arm "2  pv exit 3 is Fail with pv's error: line; exit 1 (exit $rc)" $?
cb1201 "$T/empty:/usr/bin:/bin" "$T/r"
[ "$rc" = 1 ] && row_is Fail "decline: pv not found"
arm "3  pv absent while contracts/ exists is Fail 'decline: pv not found'; exit 1 (exit $rc)" $?
cb1201 "$T/pv0:$PATH" "$T/r"
row_is Pass ""
arm "4  control: pv exit 0 with passed:true stays Pass (exit $rc)" $?
cb1201 "$T/pv2:$PATH" "$T/bare"
row_is Skip ""
arm "5  control: no contracts/ stays Skip, pv is not consulted (exit $rc)" $?

# ── (b) the rule list ───────────────────────────────────────────────────────
"$PMAT" comply check --list --format json > "$T/rules.json" 2>/dev/null; rc=$?
# -s slurps: `length==1` says stdout is exactly ONE JSON document, and `.[0]` is that
# document, which must be a flat array of rule objects.
[ "$rc" = 0 ] && jq -se 'length==1 and (.[0]|type=="array" and length>100
    and all(.[]; (keys|sort)==["group","id","name"])
    and any(.[]; .id=="CB-1201" and .name=="PV Lint")
    and any(.[]; (.id|startswith("CB-")) and .name=="contracts-armed-gates-monotone"))' "$T/rules.json" >/dev/null
arm "6  --list --format json is [{id,group,name}] holding CB-1201 'PV Lint' and contracts-armed-gates-monotone (exit $rc)" $?
id=$(jq -r '.[]|select(.name=="contracts-armed-gates-monotone").id' "$T/rules.json" 2>/dev/null)
"$PMAT" comply check --path "$T/bare" --checks "${id:-CB-NONE}" --format json > "$T/out.json" 2>/dev/null; rc=$?
jq -e --arg id "$id" '[.checks[]|select(.name|startswith($id+":"))]|length==1' "$T/out.json" >/dev/null
arm "7  the listed id ${id:-<none>} is one --checks accepts (exit $rc)" $?

# ── (c) CB-2118 on a git fixture ────────────────────────────────────────────
G="$T/g"; mkdir -p "$G/contracts"
g() { git -C "$G" -c user.email=a@b -c user.name=a -c core.hooksPath=/dev/null "$@" >/dev/null 2>&1; }
printf '{"armed_gates":["validate","audit"],"armed_shapes":["ont-shapes-v1"]}\n' > "$G/contracts/lint-baseline.json"
g init -q -b master && g add . && g commit -qm base && g checkout -qb topic || exit 2
cb2118() { # <dir>
  "$PMAT" comply check --path "$1" --checks "${id:-CB-NONE}" --format json > "$T/out.json" 2>/dev/null; rc=$?
  jq -c --arg id "$id" '[.checks[]|select(.name|startswith($id+":"))]' "$T/out.json" > "$T/row" 2>/dev/null || echo '[]' > "$T/row"
}
cb2118 "$G"
[ "$rc" = 0 ] && row_is Pass ""
arm "8  a branch that keeps its arming passes (exit $rc)" $?
printf '{"armed_gates":["validate","audit","score"]}\n' > "$G/contracts/lint-baseline.json"
cb2118 "$G"
[ "$rc" = 0 ] && row_is Pass ""
arm "9  adding a gate, or removing armed_shapes (= every shape armed), passes (exit $rc)" $?
printf '{"armed_gates":["validate"],"armed_shapes":[]}\n' > "$G/contracts/lint-baseline.json"
cb2118 "$G"
[ "$rc" = 1 ] && row_is Fail "armed_gates: audit" && row_is Fail "armed_shapes: ont-shapes-v1"
arm "10 dropping a gate and a shape fails naming both; exit 1 (exit $rc)" $?
rm -f "${G:?}/contracts/lint-baseline.json"
cb2118 "$G"
[ "$rc" = 1 ] && row_is Fail "armed_gates: validate" && row_is Fail "armed_shapes: ont-shapes-v1"
arm "10b deleting the baseline drops every gate and shape, naming each; exit 1 (exit $rc)" $?
cb2118 "$T/bare"
row_is Skip "decline: no contracts/"
arm "11 no contracts/ is Skip 'decline: no contracts/' (exit $rc)" $?
N="$T/n"; mkdir -p "$N/contracts"; git -C "$N" init -q -b master >/dev/null 2>&1
git -C "$N" -c user.email=a@b -c user.name=a -c core.hooksPath=/dev/null commit -q --allow-empty -m n >/dev/null 2>&1
cb2118 "$N"
row_is Pass "never armed"
arm "12 a repo that never armed passes — the rule does not force arming (exit $rc)" $?

# ── F-31 ────────────────────────────────────────────────────────────────────
needle="aprender""_contracts::"
! git -C "$REPO" grep -q -F "$needle" -- 'src/*.rs'
arm "13 F-31: no src/ file uses pv's library" $?

[ "$red" = 0 ] && echo "ont11-audit: all arms green" || echo "ont11-audit: RED"
exit "$red"
