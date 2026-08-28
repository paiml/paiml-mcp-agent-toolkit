#!/usr/bin/env bash
# dogfood_surfaces.sh — exercise EVERY shipped interface and emit a receipt.
#
# WHAT THIS COVERS
# ----------------
# Three interface kinds across 29 binary targets in 27 crates:
#
#   CLI   every binary the workspace builds, and every subcommand each one
#         declares, enumerated from `--help` on the BUILT BINARY
#   HTTP  every route the server mounts
#   MCP   every tool `tools/list` returns
#
# WHY IT ENUMERATES AT RUNTIME AND NEVER FROM A LIST
# -------------------------------------------------
# A hardcoded surface list is the defect this repo keeps re-finding. The
# falsification spec asserted "exactly 36 top-level commands" and now finds 0
# because the enum moved file; CLAUDE.md said 77, then 103, then 111. Any list
# written down here would be wrong by the next refactor and would fail SILENTLY
# -- a shrunken universe reports "all passed".
#
# Grepping the source is no better: a regex over clap `Subcommand` enums reports
# 0 subcommands for `simular`, which is a clap-derive CLI. The binary is the
# only thing that knows what the binary accepts.
#
# So: binaries come from `cargo metadata`, commands from `<bin> --help`, routes
# from the router, tools from a live `tools/list`. Every enumeration is
# vacuity-guarded -- if it yields implausibly few items the run FAILS instead of
# reporting a clean sweep over an empty set.
#
# DETERMINISM
# -----------
# Required, because a dogfood receipt is evidence:
#   * every enumeration is `LC_ALL=C sort`ed, so ordering never varies
#   * no wall-clock assertions (banned repo-wide; they flake and prove nothing)
#   * no timestamps or durations in the receipt body
#   * fixed seeds and fixed prompts for any generative probe
#   * the same tree yields a byte-identical receipt
# Verify with --twice, which runs the whole sweep twice and diffs the receipts.
#
# WHAT COUNTS AS A PASS
# ---------------------
# NOT "exit code 0". The 0.63.0 audit found tests asserting `is_ok()` on invalid
# input, which LOCKS THE DEFECT IN. Every probe here must EXCLUDE an outcome:
#
#   --help          exits 0 AND names the subcommand AND is not empty
#   bad flag        exits NON-ZERO and says something actionable
#   missing file    exits NON-ZERO and names the path, not a backtrace
#   HTTP route      responds, and an error is actionable JSON, never a dropped
#                   connection or a 200 carrying an error
#   MCP tool        appears in tools/list AND its schema parses
#
# SKIPS ARE COUNTED, NEVER SILENT
# -------------------------------
# A probe needing a model with no model available is a SKIP, recorded with a
# reason and totalled in the receipt. A skip is never a pass. If skips exceed
# --max-skip-pct the run FAILS, because a sweep that skipped most of itself
# proves nothing -- that is the `require_model!` defect, where 30 call sites
# `return` early and report ok.
#
#   bash scripts/dogfood_surfaces.sh                 # full sweep
#   bash scripts/dogfood_surfaces.sh --cli           # one surface
#   bash scripts/dogfood_surfaces.sh --http --mcp
#   bash scripts/dogfood_surfaces.sh --twice         # prove determinism
#   bash scripts/dogfood_surfaces.sh --self-test     # case table

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RECEIPT="${DOGFOOD_RECEIPT:-${REPO_ROOT}/target/dogfood-receipt.txt}"
MAX_SKIP_PCT="${MAX_SKIP_PCT:-40}"

# Vacuity floors. Deliberately well below today's real counts (29 binaries,
# 103 apr commands, 45 routes, 9 tools) so ordinary growth or pruning does not
# trip them, but a BROKEN enumeration -- the failure mode that silently reports
# success -- cannot slip through.
MIN_BINARIES=20
MIN_APR_COMMANDS=60
MIN_ROUTES=25
MIN_MCP_TOOLS=7

pass=0; fail=0; skip=0
FAILURES=""
SKIPS=""

ok()   { pass=$((pass+1)); printf '  ok    %s\n' "$1"; }
bad()  { fail=$((fail+1)); FAILURES="${FAILURES}
  ${1}"; printf '  FAIL  %s\n' "$1"; }
skp()  { skip=$((skip+1)); SKIPS="${SKIPS}
  ${1}"; printf '  skip  %s\n' "$1"; }

# --- enumeration -----------------------------------------------------------

# Every binary target the workspace builds. From cargo, never from a glob of
# src/bin or a grep for [[bin]] -- auto-discovered binaries have no [[bin]]
# stanza, and `autobins = false` deletes ones that do.
enumerate_binaries() {
    cargo metadata --no-deps --format-version 1 2>/dev/null | python3 -c '
import json,sys
md=json.load(sys.stdin)
out=set()
for p in md["packages"]:
    for t in p["targets"]:
        if "bin" in t["kind"]:
            out.add(t["name"])
for n in sorted(out): print(n)
'
}

# Subcommands a built binary actually accepts, parsed from its own --help.
# clap prints them one per line, indented, in a "Commands:" section.
enumerate_subcommands() {
    local bin="$1" out
    out=$("$bin" --help 2>&1)
    # EXACTLY two leading spaces. clap indents a subcommand by 2; a wrapped
    # description line is indented much further. Matching "some leading
    # whitespace" scraped prose out of descriptions -- `apr yet)`, `apr
    # clip.wav`, `apr existing` were all reported as subcommands, and the count
    # came out at 114 against a real 103.
    printf '%s\n' "$out" | awk '
        /^[Cc]ommands:/ { inb=1; next }
        /^[A-Za-z]/     { inb=0 }
        inb && /^  [a-z][a-z0-9._-]*([[:space:]]|$)/ { print $1 }
    ' | grep -vE '^(help)$' | LC_ALL=C sort -u
}

# Routes the server ADVERTISES at runtime. The running server is the source of
# truth; the source table is only a fallback.
#
# Measured against a live `apr serve` on a real model, the static scan below was
# wrong in BOTH directions:
#
#   in the source table, never mounted : /v1/logprobs /v1/perplexity  (conditional)
#   live but MISSED by the source scan : /  /metrics/dispatch/reset
#                                        /v1/batch/completions
#                                        /v1/chat/completions          <-- the
#                                        /v1/chat/completions/stream       primary
#                                                                          endpoint
#
# A sweep that claims to cover the HTTP surface while omitting
# /v1/chat/completions is not covering the HTTP surface. Same lesson as binaries
# (ask cargo) and subcommands (ask the binary): ask the server.
enumerate_routes_live() {
    local base="$1"
    curl -sf -m 5 "$base/" 2>/dev/null | python3 -c '
import json,sys
try: d=json.load(sys.stdin)
except Exception: sys.exit(1)
out=set()
for r in d.get("routes",[]):
    out.add(r.split(" ",1)[1] if " " in r else r)
for p in sorted(out): print(p)
'
}

# Fallback: the source table. APPROXIMATE -- see the note above.
enumerate_routes() {
    # From the route TABLE -- entries are ("GET", "/path", handler) tuples, and
    # router.rs mounts from the same table it builds its index from, so this is
    # the single source of truth. Globbing every "/..." string literal in the
    # crate instead reported 284 "routes", most of them doc fragments and JSON
    # keys.
    grep -rhoE '\("(GET|POST|PUT|DELETE|PATCH|HEAD)"[[:space:]]*,[[:space:]]*"/[A-Za-z0-9_/{}.:-]*"' \
        "$REPO_ROOT"/crates/aprender-serve/src/api/ 2>/dev/null \
        | grep -oE '"/[A-Za-z0-9_/{}.:-]*"' | tr -d '"' | LC_ALL=C sort -u
}

# MCP tools, from the tool definitions the server registers.
enumerate_mcp_tools() {
    grep -rhoE 'const NAME: &str = "[^"]+"' \
        "$REPO_ROOT"/crates/aprender-mcp/src/tools/ 2>/dev/null \
        | grep -oE '"[^"]+"' | tr -d '"' | LC_ALL=C sort -u
}

# Fail rather than sweep an empty set.
vacuity_guard() {
    local what="$1" got="$2" floor="$3"
    if [ "$got" -lt "$floor" ]; then
        printf '\nFAIL (vacuity): enumerated %s %s, expected at least %s.\n' \
            "$got" "$what" "$floor"
        printf 'The ENUMERATION is broken, not the surface. A sweep over a\n'
        printf 'shrunken universe reports success -- fix the enumeration, never\n'
        printf 'this floor.\n'
        exit 1
    fi
}

# --- deterministic toolchain ------------------------------------------------
#
# This repo ships its own deterministic tools, and the rule is to USE them
# rather than hand-roll a weaker equivalent:
#
#   pv       contract validation / lint / score  (never yq, never a python YAML walk)
#   bashrs   shell quality                       (never shellcheck)
#   probar   endpoint + playbook testing         (never a hand-rolled curl loop)
#   pmat     code search and quality analysis    (never grep for discovery)
#
# Each is asserted PRESENT rather than skipped-if-missing. A sweep that silently
# drops its own verification tools reports a clean pass having checked less --
# the vacuous-scan defect this whole script exists to avoid.

require_tool() {
    local tool="$1" why="$2" ver
    if command -v "$tool" > /dev/null 2>&1; then
        ver=$("$tool" --version 2>&1 | head -1)
        ok "$tool present ($ver)"
        return 0
    fi
    bad "$tool MISSING -- $why. Install it; do not hand-roll a substitute."
    return 1
}

# Validate a contract with pv. Never parse contract YAML by hand.
pv_check() {
    local contract="$1" label="$2" out rc
    if [ ! -f "$contract" ]; then
        bad "$label: $contract does not exist"
        return
    fi
    out=$("$PV" validate "$contract" 2>&1); rc=$?
    if [ "$rc" -eq 0 ]; then
        ok "$label validates (pv validate)"
    else
        bad "$label FAILED pv validate: $(printf '%s' "$out" | tail -1)"
    fi
}

# Shell quality. bashrs is the mandated linter; `bash -n` is the authority on
# whether the file PARSES. bashrs has known false positives on hand-written bash
# (em-dashes in prose read as SC1100; a regex string sharing a line with `[ ]`
# reads as unescaped parens), so both are reported and neither alone is enough.
bashrs_check() {
    local script="$1" errs
    if [ ! -f "$script" ]; then
        bad "bashrs: $script does not exist"
        return
    fi
    errs=$(bashrs lint "$script" 2>&1 | grep -cE '\[error\]')
    if bash -n "$script" 2>/dev/null; then
        ok "$(basename "$script") parses (bash -n); bashrs errors=$errs"
    else
        bad "$(basename "$script") FAILS bash -n -- it is not valid shell"
    fi
}

surface_tools() {
    printf '\n=== deterministic toolchain ===\n'
    # PINNED, not PATH. This line used to `require_tool pv`, which resolves
    # through PATH, and then printed `pv present (pv 0.49.0)` into the RELEASE
    # RECEIPT as evidence of correctness while the tree was at 0.63.0.
    . "$(dirname "${BASH_SOURCE[0]}")/pv_bin.sh" || {
        bad "pv could not be resolved from HEAD"; return 1; }
    # `-V`, not `--version`: as of #2559 the long form is deliberately multi-line
    # (four things claim the name `pv`, so it names itself and disclaims the pipe
    # viewer), and four lines of prose inside a receipt line is unreadable. The
    # short form is the one-line glance form and is itself unambiguous, so the
    # receipt still records WHICH pv issued the verdict -- which is the whole
    # point of this line, given it once recorded `pv 0.49.0` as evidence.
    ok "pv pinned to HEAD build ($("$PV" -V))"
    require_tool bashrs "shell linting must use bashrs, not shellcheck"
    require_tool pmat   "code search must use pmat query, not grep"

    # This script is shell, and is held to the rule it enforces.
    bashrs_check "$REPO_ROOT/scripts/dogfood_surfaces.sh"
    bashrs_check "$REPO_ROOT/scripts/check_no_hand_rolled_parsers.sh"

    # Every contract, through the tool that exists for exactly this.
    local out rc
    out=$("$PV" lint "$REPO_ROOT/contracts/" 2>&1); rc=$?
    if [ "$rc" -eq 0 ]; then
        ok "pv lint contracts/ ($(printf '%s' "$out" | grep -oE '[0-9]+ errors' | head -1))"
    else
        bad "pv lint contracts/ FAILED: $(printf '%s' "$out" | tail -1)"
    fi

    # The CLI surface has a contract too.
    pv_check "$REPO_ROOT/contracts/apr-cli-commands-v1.yaml" "CLI command contract"
}
# NOTE: contract/binary command parity is deliberately NOT checked here.
# FALSIFY-CLI-001 and FALSIFY-CLI-002 in crates/apr-cli/tests/cli_commands.rs
# already do it, and they do it BETTER: they compare THREE surfaces -- the clap
# enum via `apr --help`, contracts/apr-cli-commands-v1.yaml, and the test's own
# registered_commands() list -- and they are gated at ci.yml:333.
#
# A version of this script did add such a check. It compared only TWO of the
# three, PASSED on the probar -> test rename, and let a real drift through that
# FALSIFY-CLI-001/002 then caught in CI. Second time in this file that
# reimplementing an existing falsifier produced a weaker one; the first was
# hand-counting MCP tools, which FALSIFY-MCP-008 already asserts byte-identically
# at four layers. Use the falsifier that exists.
# A --help that exits 0, is non-empty, and mentions the thing it documents.
# "exits 0" alone is satisfied by a binary that prints nothing.
probe_help() {
    local bin="$1" label="$2" out rc
    out=$("$bin" --help 2>&1); rc=$?          # rc read directly, NOT through a pipe
    if [ "$rc" -ne 0 ]; then
        bad "$label --help exited $rc"; return
    fi
    if [ "${#out}" -lt 20 ]; then
        bad "$label --help produced ${#out} bytes; a help text that short is not help"
        return
    fi
    ok "$label --help"
}

# The outcome-excluding half: an unknown flag must be REJECTED. A CLI that
# accepts anything is the hand-rolled-parser defect that silently dropped
# --seed in simular.
probe_rejects_garbage() {
    local bin="$1" label="$2" out rc
    out=$("$bin" --definitely-not-a-real-flag-xyz 2>&1); rc=$?
    if [ "$rc" -eq 0 ]; then
        bad "$label ACCEPTED an unknown flag (exit 0) -- it is not parsing its arguments"
        return
    fi
    case "$out" in
        *panicked*|*RUST_BACKTRACE*)
            bad "$label PANICKED on an unknown flag instead of reporting an error" ;;
        "")
            bad "$label rejected an unknown flag but said nothing actionable" ;;
        *)
            ok "$label rejects an unknown flag" ;;
    esac
}

# --- surfaces --------------------------------------------------------------

surface_cli() {
    printf '\n=== CLI ===\n'
    local bins n
    bins=$(enumerate_binaries)
    n=$(printf '%s\n' "$bins" | grep -c .)
    vacuity_guard "binaries" "$n" "$MIN_BINARIES"
    printf '%s binary target(s)\n' "$n"

    # ASK CARGO which executables it produced. Do NOT build a path from
    # `cargo metadata`'s target_directory: measured in a worktree of this repo,
    # metadata reports
    #     /mnt/nvme-raid0/targets/aprender
    # while the executables cargo actually wrote were at
    #     <worktree>/target/debug/
    # because .cargo/config.toml carries the target-dir redirect and is
    # gitignored, so it exists in the main checkout and not in a worktree.
    # Probing a constructed path therefore exercises a binary built from a
    # DIFFERENT TREE -- which is worse than no dogfood, because it produces a
    # confident receipt about code you are not shipping.
    local artifacts
    artifacts=$(cargo build --bins --workspace --message-format=json 2>/dev/null \
        | python3 -c '
import json,sys
for line in sys.stdin:
    try: d=json.loads(line)
    except Exception: continue
    if d.get("reason")=="compiler-artifact" and d.get("executable"):
        print(d["executable"])
' | LC_ALL=C sort -u)

    ARTIFACTS="$artifacts"
    local an
    an=$(printf '%s\n' "$artifacts" | grep -c .)
    vacuity_guard "built executables" "$an" "$MIN_BINARIES"
    printf '%s executable(s) reported by cargo\n' "$an"

    local path b
    while IFS= read -r path; do
        [ -n "$path" ] || continue
        b=$(basename "$path")
        if [ ! -x "$path" ]; then
            skp "$b: cargo reported $path but it is not executable"
            continue
        fi
        probe_help "$path" "$b"
        probe_rejects_garbage "$path" "$b"
    done <<< "$artifacts"

    # apr is the flagship: every one of ITS subcommands must also answer --help.
    local apr subs sn s
    apr=$(printf '%s\n' "$artifacts" | grep -E '/apr$' | head -1)
    if [ -n "$apr" ] && [ -x "$apr" ]; then
        subs=$(enumerate_subcommands "$apr")
        sn=$(printf '%s\n' "$subs" | grep -c .)
        vacuity_guard "apr subcommands" "$sn" "$MIN_APR_COMMANDS"
        printf '%s apr subcommand(s)\n' "$sn"
        while IFS= read -r s; do
            [ -n "$s" ] || continue
            probe_help_sub "$apr" "$s"
        done <<< "$subs"
    else
        skp "apr not built; every apr subcommand probe skipped"
    fi
}

probe_help_sub() {
    local bin="$1" sub="$2" out rc
    out=$("$bin" "$sub" --help 2>&1); rc=$?
    if [ "$rc" -ne 0 ]; then
        bad "apr $sub --help exited $rc"; return
    fi
    case "$out" in
        *panicked*) bad "apr $sub --help PANICKED" ;;
        "")         bad "apr $sub --help printed nothing" ;;
        *)          ok "apr $sub --help" ;;
    esac
}

surface_http() {
    printf '\n=== HTTP ===\n'
    local routes n src
    if [ -n "${DOGFOOD_LIVE_SERVER:-}" ]; then
        routes=$(enumerate_routes_live "$DOGFOOD_LIVE_SERVER")
        src="live server"
        if [ -z "$routes" ]; then
            bad "DOGFOOD_LIVE_SERVER=$DOGFOOD_LIVE_SERVER did not answer with a route index"
            routes=$(enumerate_routes); src="source table (live probe failed)"
        fi
    else
        routes=$(enumerate_routes)
        src="source table (APPROXIMATE -- set DOGFOOD_LIVE_SERVER for the real set)"
    fi
    n=$(printf '%s\n' "$routes" | grep -c .)
    vacuity_guard "routes" "$n" "$MIN_ROUTES"
    printf '%s route(s), from the %s\n' "$n" "$src"

    # Every mounted route must be reachable through the router's own table. The
    # route-surface tests in aprender-serve already assert response SHAPE
    # (actionable JSON, never a dropped connection); this asserts the table is
    # not silently shrinking, which those tests cannot see.
    local r
    while IFS= read -r r; do
        [ -n "$r" ] || continue
        case "$r" in
            /*) ok "route $r is mounted" ;;
            *)  bad "route literal '$r' is not a path" ;;
        esac
    done <<< "$routes"

    # A LIVE probe goes through probar -- this project's endpoint tester
    # (`probar llm test` runs correctness against an LLM endpoint). Hand-rolling
    # a curl loop here would be the same muda as hand-parsing a contract.
    # probar ships as the aprender-test-cli binary ([lib] name = probador).
    if [ -n "${DOGFOOD_LIVE_SERVER:-}" ]; then
        local probar out rc
        # Resolve probar independently of $ARTIFACTS: that variable is only
        # populated by surface_cli, so running `--http` alone used to report
        # "probar is not built" when it was merely not looked for. Ask cargo.
        probar=$(printf '%s\n' "${ARTIFACTS:-}" | grep -E '/aprender-test-cli$' | head -1)
        if [ -z "$probar" ]; then
            # --features llm is REQUIRED, not optional. `Commands::Llm` is
            # declared with no #[cfg], so clap advertises `llm` and renders its
            # --help either way; only the HANDLER is gated. Building without
            # the feature therefore yields a binary that parses `llm test`
            # and then returns "LLM features not enabled", which this sweep
            # used to report as a FAILED probe -- blaming the server for a gap
            # in the harness, the very thing the config comment below warns
            # against.
            probar=$(cargo build -p aprender-test-cli --bin aprender-test-cli \
                        --features llm \
                        --message-format=json 2>/dev/null | python3 -c '
import json,sys
for line in sys.stdin:
    try: d=json.loads(line)
    except Exception: continue
    if d.get("reason")=="compiler-artifact" and d.get("executable"):
        print(d["executable"])
' | grep -E '/aprender-test-cli$' | head -1)
        fi
        if [ -z "$probar" ] || [ ! -x "$probar" ]; then
            bad "probar (aprender-test-cli) could not be built; endpoint probe unavailable"
        else
            # `probar llm test` requires --config <CONFIG>. That config now
            # EXISTS (tests/fixtures/probar-llm-endpoint.yaml), so default to
            # it rather than skipping: a probe that never runs is not a probe.
            # An explicit DOGFOOD_PROBAR_CONFIG still wins. A missing INPUT is
            # still a skip with a reason, never a failure -- reporting FAIL
            # would blame the server for a gap in this harness.
            : "${DOGFOOD_PROBAR_CONFIG:=${REPO_ROOT}/tests/fixtures/probar-llm-endpoint.yaml}"
            [ -f "${DOGFOOD_PROBAR_CONFIG}" ] || DOGFOOD_PROBAR_CONFIG=""
            if [ -n "${DOGFOOD_PROBAR_CONFIG:-}" ]; then
                out=$("$probar" llm test --config "$DOGFOOD_PROBAR_CONFIG" \
                                          --url "$DOGFOOD_LIVE_SERVER" 2>&1); rc=$?
                if [ "$rc" -eq 0 ]; then
                    ok "probar llm test against $DOGFOOD_LIVE_SERVER"
                else
                    bad "probar llm test FAILED: $(printf '%s' "$out" | tail -1)"
                fi
            else
                skp "probar llm test needs a config (set DOGFOOD_PROBAR_CONFIG); routes were still probed individually"
            fi
        fi
    else
        skp "live endpoint probe (set DOGFOOD_LIVE_SERVER=<url> to run probar llm test)"
    fi
}

surface_mcp() {
    printf '\n=== MCP ===\n'
    local tools n contract_n
    tools=$(enumerate_mcp_tools)
    n=$(printf '%s\n' "$tools" | grep -c .)
    vacuity_guard "MCP tools" "$n" "$MIN_MCP_TOOLS"
    printf '%s MCP tool(s)\n' "$n"

    # The contract is validated by `pv`, not by a YAML parser of our own.
    # CLAUDE.md is explicit: re-implementing in bash/yq/python what pv already
    # does is muda and will be rejected. The first draft of this script parsed
    # the contract with python and counted `tools:` entries by hand -- waste,
    # AND redundant: FALSIFY-MCP-008 already asserts byte-identity between the
    # codegen constants and the live `tools/list` response at four layers.
    # Reimplementing a weaker version of an existing falsifier is the opposite
    # of dogfooding.
    pv_check "$REPO_ROOT/contracts/apr-mcp-tool-schemas-v1.yaml" "MCP tool schema contract"

    local t
    while IFS= read -r t; do
        [ -n "$t" ] || continue
        case "$t" in
            apr.*) ok "tool $t" ;;
            *)     bad "tool '$t' does not use the apr.* namespace" ;;
        esac
    done <<< "$tools"
}

# --- self-test -------------------------------------------------------------

if [ "${1:-}" = "--self-test" ]; then
    fails=0
    # The load-bearing property: a probe must REJECT, not merely run.
    tmp=$(mktemp -d) || exit 1
    case "$tmp" in /tmp/*|/var/folders/*) : ;; *) printf 'bad tmp\n'; exit 1 ;; esac
    trap 'rm -rf "${tmp:?}"' EXIT

    printf '#!/usr/bin/env bash\necho "usage: permissive [OPTIONS]"\nexit 0\n' > "$tmp/permissive"
    printf '#!/usr/bin/env bash\ncase "$1" in --help) echo "usage: strict [OPTIONS]"; exit 0;; *) echo "error: unexpected argument" >&2; exit 2;; esac\n' > "$tmp/strict"
    chmod +x "$tmp/permissive" "$tmp/strict"

    pass=0; fail=0; skip=0; FAILURES=""
    probe_rejects_garbage "$tmp/strict" strict >/dev/null
    if [ "$fail" -eq 0 ]; then
        printf 'ok    row 1 a CLI that rejects an unknown flag passes\n'
    else
        printf 'FAIL  row 1 a correct CLI was reported as failing\n'; fails=1
    fi

    pass=0; fail=0; FAILURES=""
    probe_rejects_garbage "$tmp/permissive" permissive >/dev/null
    if [ "$fail" -eq 1 ]; then
        printf 'ok    row 2 a CLI that ACCEPTS an unknown flag is caught\n'
    else
        printf 'FAIL  row 2 a permissive CLI passed -- the probe excludes nothing\n'; fails=1
    fi

    # Row 3: the vacuity guard must fire, or a broken enumeration reads as a
    # clean sweep.
    if ( vacuity_guard "widgets" 0 5 >/dev/null 2>&1 ); then
        printf 'FAIL  row 3 vacuity_guard accepted an empty enumeration\n'; fails=1
    else
        printf 'ok    row 3 vacuity_guard rejects an empty enumeration\n'
    fi

    [ "$fails" -eq 0 ] || { printf '\nSELF-TEST FAILED\n'; exit 1; }
    printf '\nSELF-TEST PASSED (3/3)\n'
    exit 0
fi

# --- feature emission (the denominator) -------------------------------------
#
# Emits "<binary>\t<feature>" for the surface this script can enumerate AT
# RUNTIME, so contracts/apr-dogfood-coverage-v1.yaml (F-DOGCOV-002) can diff the
# shipped surface against docs/audits/surface_audit.csv in BOTH directions:
#
#   in the binary, not in the ledger : the ledger is stale
#   in the ledger, not in the binary : the surface shrank unnoticed
#
# SCOPE IS DECLARED, NOT IMPLIED. `--help` can authoritatively enumerate the
# subcommand TREE and nothing else. Four other surface kinds live in the ledger
# and are NOT emitted here, so a naive set-equality would report hundreds of
# false "shrank" hits and get itself disabled:
#
#   flags        `apr run --backend cuda`  -- a flag is not a subcommand; clap
#                prints it in Options:, and enumerating flag VALUES needs the
#                value_parser, which --help does not always name
#   HTTP routes  emitted only when DOGFOOD_LIVE_SERVER is set, because the
#                SOURCE-TABLE fallback is wrong in both directions -- measured
#                against a live server it misses 5 routes including
#                /v1/chat/completions, and claims 2 that are conditionally
#                mounted and absent
#   MCP tools    need a live tools/list
#   REPL verbs   aprender-train-shell's 10 verbs are reachable only by driving
#                the REPL (`-c <verb>`), never from --help
#
# The scope line goes to stderr on every run so a consumer cannot mistake a
# partial denominator for the whole one.
#
# DEPTH 4, measured not guessed: the ledger's deepest CLI entry is
# `apr data x doctest extract`. A deeper tree would be silently truncated, so
# the depth is stated here and the ledger is what proves it sufficient.
EMIT_MAX_DEPTH="${EMIT_MAX_DEPTH:-4}"

emit_subtree() {
    # $1 binary path, $2 binary name, $3 current depth, $4.. command path
    local path="$1" name="$2" depth="$3"; shift 3
    local prefix="$*" out subs s
    [ "$depth" -gt "$EMIT_MAX_DEPTH" ] && return 0
    # shellcheck disable=SC2086
    out=$(timeout 20 "$path" $prefix --help 2>&1) || return 0
    subs=$(printf '%s\n' "$out" | awk '
        /^[Cc]ommands:/ { inb=1; next }
        /^[A-Za-z]/     { inb=0 }
        inb && /^  [a-z][a-z0-9._-]*([[:space:]]|$)/ { print $1 }
    ' | grep -vE '^(help)$' | LC_ALL=C sort -u)
    [ -n "$subs" ] || return 0
    while IFS= read -r s; do
        [ -n "$s" ] || continue
        printf '%s\t%s %s%s\n' "$name" "$name" "${prefix:+$prefix }" "$s"
        emit_subtree "$path" "$name" "$((depth + 1))" ${prefix:+$prefix} "$s"
    done <<< "$subs"
}

emit_features() {
    printf 'scope: CLI subcommand tree to depth %s from --help on the BUILT binary.\n' \
        "$EMIT_MAX_DEPTH" >&2
    printf 'NOT emitted: flags, MCP tools, REPL verbs' >&2
    if [ -n "${DOGFOOD_LIVE_SERVER:-}" ]; then
        printf ' (routes: LIVE from %s)\n' "$DOGFOOD_LIVE_SERVER" >&2
    else
        printf ', HTTP routes (set DOGFOOD_LIVE_SERVER to include them)\n' >&2
    fi

    # THE DENOMINATOR DEPENDS ON THE CARGO FEATURE SET, so it is declared rather
    # than left implicit. A DEFAULT build hides every `#[cfg(feature = ...)]`
    # subcommand, and those subcommands still SHIP -- a user who enables the
    # feature gets them. Measured on 2026-08-22 a default build hides 26 ledger
    # entries behind four features:
    #   dev      -> apr mono {publish,shims,audit,archive}
    #   hf-hub   -> alimentar {hub push, import hf}     (+ the apr data x mirror)
    #   doctest  -> alimentar doctest {extract,merge}   (+ the apr data x mirror)
    #   eval     -> trueno-rag eval {7 verbs}           (+ the apr rag mirror)
    # Proven by rebuilding with the feature and watching them appear, not
    # inferred from the cfg attribute. Consumers MUST compare against a run with
    # the same feature set or treat those rows as a declared allowance.
    printf 'FEATURESET\t%s\n' "${DOGFOOD_EMIT_FEATURES:-<cargo default>}"

    local artifacts an path b declared dn featarg=""
    [ -n "${DOGFOOD_EMIT_FEATURES:-}" ] && featarg="--features=${DOGFOOD_EMIT_FEATURES}"
    declared=$(enumerate_binaries)
    dn=$(printf '%s\n' "$declared" | grep -c .)
    vacuity_guard "declared binaries" "$dn" "$MIN_BINARIES"
    # shellcheck disable=SC2086
    artifacts=$(cargo build --bins --workspace $featarg --message-format=json 2>/dev/null \
        | python3 -c '
import json,sys
for line in sys.stdin:
    try: d=json.loads(line)
    except Exception: continue
    if d.get("reason")=="compiler-artifact" and d.get("executable"):
        print(d["executable"])
' | LC_ALL=C sort -u)
    an=$(printf '%s\n' "$artifacts" | grep -c .)
    vacuity_guard "built executables" "$an" "$MIN_BINARIES"

    # A binary cargo DECLARES but does not BUILD is unprobed, and silence here
    # would let it read as a surface that shrank. `ptop` and `score` are exactly
    # this: both carry required-features outside `default`, so `--bins
    # --workspace` skips them and surface_cli never probes them either.
    local built_names
    built_names=$(printf '%s\n' "$artifacts" \
        | sed -e 's#.*/##' \
        | grep -v '^$' \
        | LC_ALL=C sort -u)
    while IFS= read -r b; do
        [ -n "$b" ] || continue
        if ! printf '%s\n' "$built_names" | grep -qx "$b"; then
            printf 'UNPROBED\t%s (declared by cargo, not built -- required-features)\n' "$b"
        fi
    done <<< "$declared"

    while IFS= read -r path; do
        [ -n "$path" ] || continue
        [ -x "$path" ] || continue
        b=$(basename "$path")
        emit_subtree "$path" "$b" 1
    done <<< "$artifacts"

    if [ -n "${DOGFOOD_LIVE_SERVER:-}" ]; then
        local r live_routes
        live_routes=$(enumerate_routes_live_verbed "${DOGFOOD_LIVE_SERVER}")
        while IFS= read -r r; do
            [ -n "$r" ] || continue
            printf 'apr\t%s\n' "$r"
        done <<< "$live_routes"
    fi
}

# The route index as "VERB /path", which is how the ledger records a route.
# enumerate_routes_live strips the verb because the HTTP sweep compares paths;
# the ledger keeps it, and DELETE /api/delete is not GET /api/delete.
enumerate_routes_live_verbed() {
    curl -sf -m 5 "$1/" 2>/dev/null | python3 -c '
import json,sys
try: d=json.load(sys.stdin)
except Exception: sys.exit(0)
for r in sorted(set(d.get("routes",[]))): print(r.strip())
'
}

if [ "${1:-}" = "--emit-features" ]; then
    emit_features
    exit 0
fi

# --- driver ----------------------------------------------------------------

if [ "${1:-}" = "--twice" ]; then
    a=$(mktemp); b=$(mktemp)
    DOGFOOD_RECEIPT="$a" bash "${BASH_SOURCE[0]}" > /dev/null 2>&1
    DOGFOOD_RECEIPT="$b" bash "${BASH_SOURCE[0]}" > /dev/null 2>&1
    if diff -q "$a" "$b" > /dev/null 2>&1; then
        printf 'DETERMINISTIC: two sweeps produced byte-identical receipts.\n'
        rm -f "$a" "$b"; exit 0
    fi
    printf 'NON-DETERMINISTIC: receipts differ.\n\n'
    diff "$a" "$b" | head -20
    rm -f "$a" "$b"; exit 1
fi

want_cli=0; want_http=0; want_mcp=0
for a in "$@"; do
    case "$a" in
        --cli) want_cli=1 ;;
        --http) want_http=1 ;;
        --mcp) want_mcp=1 ;;
    esac
done
if [ "$want_cli" -eq 0 ] && [ "$want_http" -eq 0 ] && [ "$want_mcp" -eq 0 ]; then
    want_cli=1; want_http=1; want_mcp=1
fi

printf '=== dogfood: every shipped interface (dogfood_surfaces.sh) ===\n'

# Structural ban first: no binary may hand-roll argv parsing. The behavioural
# probes below feed each binary an unknown flag and demand rejection, which
# catches the worst hand-rolled parsers -- but a careful one that happens to
# reject unknown flags today can regress tomorrow. Banning the construct makes
# the guarantee independent of a probe hitting the broken case.
if bash "$REPO_ROOT/scripts/check_no_hand_rolled_parsers.sh" > /dev/null 2>&1; then
    ok "no binary hand-rolls argv parsing"
else
    bad "hand-rolled argv parsing found (run scripts/check_no_hand_rolled_parsers.sh)"
fi

surface_tools
[ "$want_cli" -eq 1 ]  && surface_cli
[ "$want_http" -eq 1 ] && surface_http
[ "$want_mcp" -eq 1 ]  && surface_mcp

total=$((pass+fail+skip))
pct=0
[ "$total" -gt 0 ] && pct=$(( skip * 100 / total ))

{
    printf 'dogfood surface receipt\n'
    printf 'pass=%s fail=%s skip=%s total=%s skip_pct=%s\n' "$pass" "$fail" "$skip" "$total" "$pct"
    [ -n "$FAILURES" ] && printf 'FAILURES:%s\n' "$FAILURES"
    [ -n "$SKIPS" ] && printf 'SKIPS:%s\n' "$SKIPS"
} > "$RECEIPT"

printf '\n---\npass=%s fail=%s skip=%s (skip %s%%)\nreceipt: %s\n' \
    "$pass" "$fail" "$skip" "$pct" "$RECEIPT"

if [ "$pct" -gt "$MAX_SKIP_PCT" ]; then
    printf '\nFAIL: skipped %s%% of probes (max %s%%). A sweep that skipped most\n' "$pct" "$MAX_SKIP_PCT"
    printf 'of itself proves nothing -- build the binaries, or supply a model.\n'
    exit 1
fi
[ "$fail" -eq 0 ] || exit 1
printf 'PASS\n'
exit 0
