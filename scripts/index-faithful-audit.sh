#!/usr/bin/env bash
# index-faithful-audit — CRUX-07 acceptance (pmat-architecture-crux-audit.md §8.7): the index is a faithful,
# reproducible view of the tree. Five legs, each with a control that must pass on the same binary.
#   PMAT=<binary> scripts/index-faithful-audit.sh [--only a,b,c,d,e]
# Exit 0 = every leg green; 1 = a leg red.
set -uo pipefail
PMAT=${PMAT:-pmat}; ONLY=${2:-a,b,c,d,e}; [ "${1:-}" = "--only" ] && ONLY=$2
red=0; leg(){ if [ "$2" = 0 ]; then echo "  ✓ $1"; else echo "  ✗ $1"; red=1; fi; }
want(){ case ",$ONLY," in *",$1,"*) return 0;; *) return 1;; esac; }
echo "index-faithful-audit (CRUX-07) — $($PMAT --version 2>/dev/null | head -1)"
T=$(mktemp -d); [ -n "$T" ] && [ "$T" != "/" ] || exit 2
[[ "$T" == /* && "$T" != *..* ]] || exit 2
cleanup() { if [ -n "${T:-}" ] && [ "$T" != "/" ] && [ -d "$T" ]; then chmod -R u+w "$T" 2>/dev/null; find "$T" -depth -delete 2>/dev/null; fi; }
trap cleanup EXIT
mk_crate(){ # $1 dir: a one-file crate in a git repository
  mkdir -p "$1/src"; printf '[package]\nname = "fx"\nversion = "0.1.0"\nedition = "2021"\n' > "$1/Cargo.toml"
  ( cd "$1" && git init -q --template= && git config user.email t@t && git config user.name t && git config core.hooksPath /dev/null )
}
built_at_epoch(){ python3 -c 'import json,sys,datetime; m=json.load(open(sys.argv[1])); s=m["built_at"]; s=s[:26]+"+00:00" if "." in s else s; print(int(datetime.datetime.fromisoformat(s.replace("Z","+00:00")).timestamp()))' "$1"; }
# ---- (a) a content change whose mtime predates built_at must be seen; the fast path must survive
if want a; then
  A="$T/a"; mk_crate "$A"; printf 'pub fn alpha_only() -> u32 { 1 }\n' > "$A/src/lib.rs"; ( cd "$A" && git add . && git commit -qm one )
  ( cd "$A" && $PMAT query alpha_only --limit 3 >/dev/null 2>&1 )
  M="$A/.pmat/context.idx/manifest.json"; [ -f "$M" ] || { leg "a: the index was built (manifest present)" 1; }
  if [ -f "$M" ]; then
    BA=$(built_at_epoch "$M")
    printf 'pub fn beta_only() -> u32 { 2 }\n' > "$A/src/lib.rs"
    python3 -c 'import os,sys; t=int(sys.argv[2])-1; os.utime(sys.argv[1],(t,t))' "$A/src/lib.rs" "$BA"   # backdate RELATIVE to built_at
    names(){ ( cd "$A" && $PMAT query "$1" --limit 5 --format json 2>/dev/null ) | python3 -c 'import json,sys
try: d=json.load(sys.stdin)
except Exception: d=[]
print(" ".join(r.get("function_name","") for r in (d if isinstance(d,list) else [])))'; }
    echo "$(names beta_only)" | grep -qw 'beta_only' && leg "a: a rewrite whose mtime predates built_at is indexed (beta_only is a result)" 0 || leg "a: backdated rewrite indexed (beta_only is NOT a result)" 1
    echo "$(names alpha_only)" | grep -qw 'alpha_only' && leg "a: the deleted alpha_only is no longer served" 1 || leg "a: the deleted alpha_only is no longer served" 0
    # control: the fast path survives — a repeat query on the quiescent tree still skips by mtime
    ( cd "$A" && python3 -c 'import os,sys,time; t=int(sys.argv[2])-5; os.utime(sys.argv[1],(t,t))' src/lib.rs "$BA" ) # an older mtime than the (new) built_at
    ( cd "$A" && $PMAT query beta_only --limit 1 >/dev/null 2>&1 ); out3=$( cd "$A" && $PMAT query beta_only --limit 1 2>&1 )
    echo "$out3" | grep -qE 'Incremental update: [1-9][0-9]* mtime-skipped, [0-9]+ checksum-reused, 0 re-parsed' && leg "a control: the fast path survives (mtime-skipped > 0, 0 re-parsed on a quiescent tree)" 0 || leg "a control: fast path survives (got: $(echo "$out3" | grep -i incremental | head -1 | cut -c1-70))" 1
  fi
fi
# ---- (b) result order is the sorted (file_path, start_line) sequence and nothing is truncated
if want b; then
  B="$T/b"; mk_crate "$B"; : > "$B/src/lib.rs"
  for f in mm aa zz bb yy cc; do printf 'pub fn zz_%s() -> u32 { 1 }\n' "$f" > "$B/src/$f.rs"; echo "pub mod $f;" >> "$B/src/lib.rs"; done
  ( cd "$B" && git add . && git commit -qm six )
  J=$( cd "$B" && $PMAT query zz_ --limit 10 --format json 2>/dev/null )
  cat > "$T/legb.py" <<'LEGB'
import json,sys
d=json.loads(sys.argv[1]) if sys.argv[1].strip() else []
rs=d if isinstance(d,list) else (d.get("results") or [])
seq=[(r.get("file_path"), r.get("start_line") or 0) for r in rs]
assert len(seq)==6, f"expected 6 results, got {len(seq)}: {seq}"
assert seq==sorted(seq), f"unsorted: {seq}"
LEGB
  why=$(python3 "$T/legb.py" "$J" 2>&1); rc=$?
  [ "$rc" = 0 ] && leg "b: six tied results, all returned, in sorted (file_path, start_line) order" 0 || leg "b: six tied results in sorted order ($(echo "$why" | grep -E 'AssertionError|Error' | tail -1 | cut -c1-110))" 1
fi
# ---- (c) analyze churn JSON is byte-stable in raw key order (c1) and in sorted form (c2), with real content
if want c; then
  C="$T/c"; mk_crate "$C"; for a in ann bob cid; do for i in 1 2; do echo "$a $i" >> "$C/src/lib.rs"; ( cd "$C" && git add . && git -c "user.name=$a" -c "user.email=$a@x" commit -qm "$a $i" ); done; done
  raw=$(for i in $(seq 1 12); do ( cd "$C" && $PMAT analyze churn --format json 2>/dev/null ) | python3 -c 'import sys,json; d=json.load(sys.stdin); d.pop("generated_at",None); print(json.dumps(d))' | sha256sum; done | sort -u | wc -l)
  [ "$raw" -eq 1 ] && leg "c1: raw (order-preserving) churn JSON is stable over 12 runs" 0 || leg "c1: raw churn JSON stable ($raw distinct hashes in 12 runs)" 1
  srt=$(for i in $(seq 1 12); do ( cd "$C" && $PMAT analyze churn --format json 2>/dev/null ) | jq -S 'del(.generated_at)' | sha256sum; done | sort -u | wc -l)
  [ "$srt" -eq 1 ] && leg "c2 control: key-sorted churn JSON is stable (catches an added timestamp)" 0 || leg "c2: sorted churn JSON stable ($srt distinct)" 1
  ( cd "$C" && $PMAT analyze churn --format json 2>/dev/null ) | python3 -c '
import json,sys,subprocess
d=json.load(sys.stdin); ac=d["summary"]["author_contributions"]
sl={l.split("\t")[1].strip(): int(l.split("\t")[0]) for l in subprocess.run(["git","shortlog","-sn","--all"],capture_output=True,text=True,cwd=sys.argv[1]).stdout.strip().splitlines()}
assert len(ac)>=3, ac
assert set(ac)==set(sl), (ac, sl)
if ac!=sl: print("  note: author counts differ from git shortlog -sn:", ac, "vs", sl, "(reported, not judged here)")' "$C" && leg "c: author_contributions names exactly the git shortlog -sn authors (3)" 0 || leg "c: author_contributions names the shortlog authors" 1
fi
# ---- (d) a torn manifest is detected; a clean pair is not flagged
if want d; then
  D="$T/d"; mk_crate "$D"; printf 'pub fn delta() -> u32 { 4 }\n' > "$D/src/lib.rs"; ( cd "$D" && git add . && git commit -qm d )
  ( cd "$D" && $PMAT query delta --limit 1 >/dev/null 2>&1 ); M="$D/.pmat/context.idx/manifest.json"
  clean=$( cd "$D" && $PMAT query delta --limit 1 2>&1 ); echo "$clean" | grep -qiE 'torn|corrupt|manifest.*(older|stale|invalid)|rebuild' && leg "d control: a clean manifest/db pair is not flagged" 1 || leg "d control: a clean manifest/db pair is not flagged" 0
  python3 -c 'import sys; p=sys.argv[1]; s=open(p).read(); open(p,"w").write(s[:len(s)//2])' "$M"   # torn mid-object after a good db write
  torn=$( cd "$D" && $PMAT query delta --limit 1 2>&1 ); echo "$torn" | grep -qiE 'torn|corrupt|manifest.*(older|stale|invalid|unreadable)|rebuil' && leg "d: a torn manifest.json is detected and named" 0 || leg "d: torn manifest detected (got: $(echo "$torn" | head -2 | tr '\n' ' ' | cut -c1-70))" 1
fi
# ---- (e) a failed index save is reported; with .pmat writable the same command is silent about saving
if want e; then
  E="$T/e"; mk_crate "$E"; printf 'pub fn eps() -> u32 { 5 }\n' > "$E/src/lib.rs"; ( cd "$E" && git add . && git commit -qm e )
  ( cd "$E" && $PMAT query eps --limit 1 >/dev/null 2>&1 )
  printf 'pub fn eps2() -> u32 { 6 }\n' >> "$E/src/lib.rs"
  ok=$( cd "$E" && $PMAT query eps2 --limit 1 2>&1 ); echo "$ok" | grep -qiE 'could not save|failed to save|save.*fail|permission denied' && leg "e control: with .pmat writable nothing complains about saving" 1 || leg "e control: with .pmat writable nothing complains about saving" 0
  printf 'pub fn eps3() -> u32 { 7 }\n' >> "$E/src/lib.rs"; chmod 555 "$E/.pmat/context.idx"   # the incremental save cannot replace manifest.json; the db still opens
  ro=$( cd "$E" && $PMAT query eps3 --limit 1 2>&1 ); rc=$?
  if echo "$ro" | grep -qiE 'saving index'; then
    echo "$ro" | grep -qiE 'could not save|failed to save|save.*fail|permission denied' && leg "e: the incremental save that fails is reported, not announced as done" 0 || leg "e: a failed incremental save is announced as done (got: $(echo "$ro" | grep -iE 'saving|saved' | head -1 | cut -c1-60), no failure line)" 1
  else
    leg "e: the incremental save path ran (got: $(echo "$ro" | head -2 | tr '\n' ' ' | cut -c1-80))" 1
  fi
  chmod -R u+w "$E/.pmat"
fi
[ "$red" = 0 ] && echo "GREEN" || { echo "RED"; exit 1; }
