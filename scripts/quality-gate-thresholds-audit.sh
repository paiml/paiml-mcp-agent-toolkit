#!/usr/bin/env bash
# quality-gate-thresholds-audit — AD-05 acceptance: `pmat quality-gate --checks file-size,churn,lint`
# enforce the three thresholds the delivery diagram names. Every leg has a control that must PASS.
#   PMAT=<binary> scripts/quality-gate-thresholds-audit.sh
# Exit 0 = every leg green; 1 = a leg red (the check is missing, or it cannot fail, or it fails the control).
set -euo pipefail
PMAT=${PMAT:-pmat}
red=0; leg(){ if [ "$2" = 0 ]; then echo "  ✓ $1"; else echo "  ✗ $1"; red=1; fi; }
echo "quality-gate-thresholds-audit (AD-05) — $($PMAT --version 2>/dev/null | head -1)"
T=$(mktemp -d); [ -n "$T" ] && [ "$T" != "/" ] || exit 2
cleanup() { if [ -n "${T:-}" ] && [ "$T" != "/" ] && [ -d "$T" ]; then find "$T" -type f -delete; find "$T" -depth -type d -empty -delete; fi; }
trap cleanup EXIT
mk_repo(){ # $1 dir; a minimal Rust crate in a git repository
  mkdir -p "$1/src"; ( cd "$1" && git init -q --template= && git config user.email t@t && git config user.name t && git config core.hooksPath /dev/null )
  printf '[package]\nname = "fx"\nversion = "0.1.0"\nedition = "2021"\n' > "$1/Cargo.toml"
  printf 'pub fn one() -> u32 { 1 }\n' > "$1/src/lib.rs"
  ( cd "$1" && git add . && git commit -qm init )
}
gate(){ ( cd "$1" && shift && "$PMAT" quality-gate --fail-on-violation --format summary "$@" >/dev/null 2>&1 ); echo $?; }
# ---- file-size: a 501-line file fails, a 499-line file passes (default max_file_lines = 500)
R="$T/size"; mk_repo "$R"
{ echo 'pub fn big() -> u32 {'; for i in $(seq 1 499); do echo "    let _v$i = $i;"; done; echo '    0'; echo '}'; } > "$R/src/big.rs"   # 502 lines
echo 'pub mod big;' >> "$R/src/lib.rs"; ( cd "$R" && git add . && git commit -qm big )
rc=$(gate "$R" --checks file-size); if [ "$rc" = 1 ]; then leg "file-size: a $(wc -l < "$R/src/big.rs")-line file fails" 0; else leg "file-size: a $(wc -l < "$R/src/big.rs")-line file fails (rc=$rc)" 1; fi
{ echo 'pub fn big() -> u32 {'; for i in $(seq 1 496); do echo "    let _v$i = $i;"; done; echo '    0'; echo '}'; } > "$R/src/big.rs"   # 499 lines
rc=$(gate "$R" --checks file-size); if [ "$rc" = 0 ]; then leg "file-size control: a $(wc -l < "$R/src/big.rs")-line file passes" 0; else leg "file-size control: a $(wc -l < "$R/src/big.rs")-line file passes (rc=$rc)" 1; fi
rc=$(gate "$R" --checks file-size --max-file-lines 400); if [ "$rc" = 1 ]; then leg "file-size: --max-file-lines 400 fails the 499-line file" 0; else leg "file-size: --max-file-lines 400 fails the 499-line file (rc=$rc)" 1; fi
# ---- churn: a file with more commits in 90 days than the threshold fails; below it passes
R="$T/churn"; mk_repo "$R"
for i in $(seq 1 6); do echo "// rev $i" >> "$R/src/lib.rs"; ( cd "$R" && git commit -qam "rev $i" ); done   # 7 commits touch lib.rs
rc=$(gate "$R" --checks churn --max-churn-commits 5); if [ "$rc" = 1 ]; then leg "churn: 7 commits to one file in 90 days fails --max-churn-commits 5" 0; else leg "churn: 7 commits fails --max-churn-commits 5 (rc=$rc)" 1; fi
rc=$(gate "$R" --checks churn --max-churn-commits 10); if [ "$rc" = 0 ]; then leg "churn control: the same file passes --max-churn-commits 10" 0; else leg "churn control: passes --max-churn-commits 10 (rc=$rc)" 1; fi
# ---- lint: one clippy warning fails; a clean crate passes
R="$T/lint"; mk_repo "$R"
printf 'pub fn one() -> u32 {\n    let x = 1;\n    return x;\n}\n' > "$R/src/lib.rs"; ( cd "$R" && git commit -qam warn )   # clippy::needless_return
rc=$(gate "$R" --checks lint); if [ "$rc" = 1 ]; then leg "lint: one clippy warning fails" 0; else leg "lint: one clippy warning fails (rc=$rc)" 1; fi
printf 'pub fn one() -> u32 {\n    1\n}\n' > "$R/src/lib.rs"; ( cd "$R" && git commit -qam clean )
rc=$(gate "$R" --checks lint); if [ "$rc" = 0 ]; then leg "lint control: a clean crate passes" 0; else leg "lint control: a clean crate passes (rc=$rc)" 1; fi
[ "$red" = 0 ] && echo "GREEN" || { echo "RED"; exit 1; }
