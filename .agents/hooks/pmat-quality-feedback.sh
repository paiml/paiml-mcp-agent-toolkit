#!/bin/sh
# pmat quality feedback for agent edits — FEEDBACK, NOT A GATE.
#
# Read this before treating it as enforcement:
#
#   Antigravity, verbatim: "If a hook script crashes (non-zero exit status), an
#   HTTP hook returns a non-2xx status code, or an operation times out or
#   returns unrecognized JSON, the runtime treats it as an approval (allow)."
#
#   Claude Code: exit 2 blocks; exit 1 blocks nothing. A hook binary that fails
#   to launch is not a deny.
#
# So if pmat is missing, slow, or broken, BOTH clients allow the edit. A gate
# that returns "allow" when it crashes is the inverse of Jidoka. The gate is
# `ci / gate` on protected master; this only shortens the feedback loop.
#
# Antigravity additionally lets an agent with write access edit this file.
# Mount `.agents/` read-only if that matters to you.
#
# Usage:  pmat-quality-feedback.sh <mode> <file>
#   mode = claude      -> exit 2 to block, 0 to allow
#   mode = antigravity -> {"decision":"deny"|"allow"} on stdout, always exit 0
set -u

mode="${1:-claude}"
file="${2:-}"

allow() {
    case "$mode" in
        antigravity) printf '{"decision":"allow"}\n' ;;
    esac
    exit 0
}

deny() {
    reason="$1"
    case "$mode" in
        antigravity)
            # JSON-escape the reason: unrecognized JSON is treated as approval.
            esc=$(printf '%s' "$reason" | sed 's/\\/\\\\/g; s/"/\\"/g' | tr -d '\n')
            printf '{"decision":"deny","reason":"%s"}\n' "$esc"
            exit 0
            ;;
        *)
            printf '%s\n' "$reason" >&2
            exit 2   # Claude Code: only exit 2 blocks
            ;;
    esac
}

# Both clients deliver the tool call as JSON on stdin; the file argument is for
# testing and for direct invocation. Extracted with sed rather than a Node or
# Python shim: adding an interpreter dependency to reach a Rust binary is the
# thing this stack is trying not to do, and the only documented Antigravity
# `command` hook examples are python3.
#
# Claude Code:  {"tool_input": {"file_path": "..."}}
# Antigravity:  {"tool_call": {"args": {"file_path"|"path": "..."}}}
#
# A crude extractor is acceptable here precisely because failure is ALLOW: a
# path this cannot parse is a check that does not run, which is the documented
# fail-open behaviour and not a new hazard.
if [ -z "$file" ] && [ ! -t 0 ]; then
    payload=$(cat 2>/dev/null || true)
    file=$(printf '%s' "$payload" \
        | tr ',' '\n' \
        | sed -n 's/.*"\(file_path\|path\|absolute_path\)"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\2/p' \
        | head -1)
fi

# No file, or not a Rust source file: nothing this checker can say.
[ -n "$file" ] || allow
case "$file" in *.rs) ;; *) allow ;; esac
[ -f "$file" ] || allow

command -v pmat >/dev/null 2>&1 || allow   # fail-open, and it is documented above

# --file keeps this sub-second; --format json puts the report on stdout and the
# human chatter on stderr, so the reason below is machine-derived.
out=$(pmat quality-gate --file "$file" --fail-on-violation --format json 2>/dev/null)
rc=$?

[ "$rc" -eq 0 ] && allow

# pmat exits 1 on violations. Claude Code needs 2 to block, Antigravity needs a
# deny decision — neither is "1", so the translation happens here rather than
# being assumed.
count=$(printf '%s' "$out" | tr -d ' \n' | grep -o '"check_type"' | wc -l | tr -d ' ')
deny "pmat quality-gate: ${count:-1} violation(s) in ${file}. Run: pmat quality-gate --file ${file}"
