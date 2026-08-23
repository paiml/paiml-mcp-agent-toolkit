#!/usr/bin/env bash
# check_verifier_pinning.sh — the verifier-pinning rule, ENFORCED.
#
# THE RULE IS NOT RESTATED HERE. It is stated once, in scripts/verifier_pin.sh,
# under the heading "THE RULE" — read it there. A rule written down twice is two
# rules that can disagree, which is the thesis of the ticket this gate closes
# (#2640): one protocol, two copies, nine silent divergences. This file is the
# rule's ENFORCEMENT, not a second copy of its text.
#
# WHY A GATE AND NOT A PARAGRAPH
# ------------------------------
# The rule already had FIVE independent ad-hoc rediscoveries before anyone
# noticed it was one rule (PMAT_BIN, scripts/pv_bin.sh, scripts/apr_bin.sh,
# aprender#2384, APR-BENCH-RFC-001). Five is the evidence that a rule merely
# stated is documentation. #2640 merged the two dogfood runners into one; this
# is what stops the sixth tool re-discovering the rule in a sixth copy.
#
# FOUR PARTS, because presence is not behaviour and definition is not use:
#
#   PART 1  (static)      no bare pv/pmat/apr in command position — and no PATH
#                         PROBE of one — anywhere the release verdict is
#                         decided. Mutation: reintroduce a bare `pmat` call.
#   PART 1b (call site)   the pins are CALLED by the runner, before the first
#                         use of their result, and the runner never assigns
#                         PMAT_BIN/PV itself. Mutation: replace the
#                         `verifier_pin_pmat` call with `PMAT_BIN=pmat`.
#                         PART 1 alone passes on that mutation — a bare-token
#                         scan cannot tell a DEFINED pin from a USED one, and
#                         `PMAT_BIN=pmat` is a legal assignment everywhere the
#                         scanner looks. That gap shipped once; this is its row.
#   PART 2  (behavioural) the two pins are EXERCISED and must select something
#                         other than what PATH offers, and exercising them must
#                         leave Cargo.lock alone. Mutations: delete the
#                         PMAT_BIN self-referential branch; bypass pv_bin.sh to
#                         a PATH pv; delete a `[[package]]` block from
#                         Cargo.lock, whereupon the pv build repairs it in
#                         passing and the lockfile row must go RED with the file
#                         restored. That last one is why this guard now runs
#                         AFTER scripts/check_lockfile_current.sh in ci.yml: it
#                         cargo-builds pv without --locked, so placed first it
#                         quietly repaired the very staleness its neighbour
#                         exists to find.
#   PART 3  (fleet path)  the runner still RUNS when invoked the way its own
#                         Usage line documents — by a relative path, against
#                         another repo. #2640 made the pin library load-bearing
#                         and fail-closed while SKILL_DIR was still resolved
#                         after `cd "$REPO_DIR"`, so `bash scripts/dogfood.sh
#                         ../other-crate` exited 2 before any gate ran.
#
# THE UNIVERSE, and why it is these three tokens
# ----------------------------------------------
# VERIFIERS = pv, pmat, apr — exactly the tools for which THIS REPO ships a pin
# (scripts/pv_bin.sh, verifier_pin_pmat, scripts/apr_bin.sh). bashrs and
# probador are verifiers too and are NOT listed: no pin exists for them, and the
# rule's second clause is "where the repo does not pin, report" — which the
# runner does. Adding them here would demand a pin that does not exist and make
# the gate unfixable. The day one ships, add the token here.
#
# THE SCANNED SURFACE, and why it is not two files
# ------------------------------------------------
# The runner DISCOVERS gates from `[package.metadata.dogfood] gates` and
# executes each one with `bash "$dg_path"` INSIDE the release verdict. A
# declared gate that resolves a verifier through PATH decides the release with
# an unknown binary exactly as directly as the runner would. So the scope is the
# runner, the pin library, AND every declared gate — derived from the manifest,
# never hardcoded, because a hardcoded list is the same defect one level down.
#
# What counts as an invocation: the token in COMMAND POSITION after comments and
# INERT text are removed. Three things are NOT inert and each has cost a real
# false negative:
#   · `$( … )` and `` ` … ` `` are COMMAND POSITION even inside double quotes.
#     `"pmat $(pmat --version)"` was live in the runner, on the line that writes
#     which pmat ran into the receipt, and the scanner reported the file clean.
#   · a PATH PROBE (`command -v pmat`, `type -aP pv`) does not run the tool but
#     DECIDES whether a gate runs at all, against PATH rather than the pin. The
#     runner gated its whole pmat section on `command -v pmat`, so releasing
#     pmat itself with no pmat on PATH would have skipped every pmat gate while
#     the pin sat resolved and unused.
#   · a verifier NAME in a `for … in` word list is a call site whose command
#     word is a variable. `for t in bashrs pmat probador; do command -v "$t"`
#     is a PATH probe of pmat that no command-position scan can see.
# A bare mention in a comment, in a single-quoted string, or in a DATA heredoc
# is NOT an invocation — the runner's own prose says "`pv lint <DIR>` is a real
# gate", and both must stay legal. The distinction is the whole difficulty, so
# it ships a case table (--self-test) rather than a reviewed regex: the
# apr-invocation patterns in this repo were wrong FIVE times and every one was
# caught by a table, none by review.
#
#   bash scripts/check_verifier_pinning.sh              # check
#   bash scripts/check_verifier_pinning.sh --self-test  # the case table only
#   bash scripts/check_verifier_pinning.sh --scan FILE… # the raw scanner
#
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# The two files that ARE the protocol. Every other in-scope file is derived from
# the manifest below, in resolve_scope().
CORE_SCOPE="scripts/dogfood.sh scripts/verifier_pin.sh"

# ---------------------------------------------------------------------------
# The scanner. Reports "<file>:<line>: <token> [<kind>]" per finding.
#
# It tokenises rather than regexing the raw line, because a regex over raw text
# gets `pmat-verify` wrong: \bpmat\b MATCHES inside `pmat-verify`, since `-` is a
# non-word character. Token equality does not. Inert spans collapse to a single
# @Q placeholder rather than being deleted, so the ARITY of wrappers such as
# `run_to <log> <cmd...>` is preserved and `run_to "$LOG" pmat query` is still
# seen as pmat in command position. Command substitutions are LIFTED out of the
# line and scanned as lines of their own, which preserves that arity while still
# reaching the code inside them.
scan() {
    python3 - "$@" <<'PY'
import re, sys

VERIFIERS = {"pv", "pmat", "apr"}

# Control words and operators: after one of these, the next word is a command.
SEPARATORS = {";", ";;", "&&", "||", "|", "|&", "(", ")", "{", "}", "&", "!",
              "if", "then", "else", "elif", "fi", "while", "until", "do",
              "done", "case", "esac", "$("}

# Wrappers that RUN their remaining arguments. After one of these the command
# position moves along, past the wrapper's own options. `command` is here
# because it was missing and `command pmat verify` was therefore invisible;
# `nice`/`ionice`/`stdbuf` are here because a prefix that takes its own option
# (`nice -n 5 pmat`) is the shape that hid it.
PREFIX = {"env", "exec", "nohup", "time", "sudo", "doas", "xargs", "command",
          "builtin", "nice", "ionice", "stdbuf", "setsid", "timeout", "chrt"}

# Probes: they ask PATH where a tool is. They do not run it, and they still
# decide whether a gate runs.
PROBES = {"type", "hash", "which", "whereis"}

ASSIGN = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
DURATION = re.compile(r"^[0-9]+[smhd]?$")
# A heredoc introducer. `<<<` is a here-STRING and must not match.
HEREDOC = re.compile(r"<<-?\s*(?!<)(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1")
# Heredoc bodies are DATA unless the command on the introducing line is a shell.
# `python3 <<'PY' … PY` holds python; `bash <<'EOF' … EOF` holds shell that RUNS.
SHELL_CMDS = {"bash", "sh", "zsh", "dash", "ksh"}


def _scan(s, i, n, mode, subs):
    """Walk s from i. Returns (inert-collapsed text, next index).

    mode: 'top' | 'dq' (inside "…") | 'sub' (inside $(…)) | 'bt' (inside `…`).
    Command substitutions found at any depth are appended to `subs` verbatim,
    to be scanned as lines of their own.
    """
    out = []
    while i < n:
        c = s[i]
        if c == "\\" and i + 1 < n:
            # An escaped char is literal — including \` inside a double-quoted
            # string, which is how the runner quotes `pv verify-bindings` in a
            # note without invoking anything.
            out.append("@" if mode == "dq" else "X")
            i += 2
            continue
        if mode == "sub" and c == ")":
            return "".join(out), i + 1
        if mode == "bt" and c == "`":
            return "".join(out), i + 1
        if c == "$" and s.startswith("$((", i):
            j = s.find("))", i + 3)
            i = n if j < 0 else j + 2
            out.append(" @Q ")
            continue
        if c == "$" and s.startswith("$(", i):
            body, i = _scan(s, i + 2, n, "sub", subs)
            subs.append(body)
            out.append(" @Q ")
            continue
        if c == "`":
            body, i = _scan(s, i + 1, n, "bt", subs)
            subs.append(body)
            out.append(" @Q ")
            continue
        if c == "'" and mode != "dq":
            i += 1
            while i < n and s[i] != "'":
                i += 1
            i += 1
            out.append(" @Q ")
            continue
        if c == '"':
            if mode == "dq":
                return "".join(out), i + 1
            _body, i = _scan(s, i + 1, n, "dq", subs)
            out.append(" @Q ")
            continue
        if c == "#" and mode != "dq":
            prev = out[-1] if out else ""
            # A `#` starts a comment only at the start of a word.
            if not out or prev.isspace() or prev in "(;&|":
                return "".join(out), n
            out.append(c)
            i += 1
            continue
        out.append("@" if mode == "dq" else c)
        i += 1
    return "".join(out), i


def strip_line(line):
    """Returns (inert-collapsed line, [command-substitution bodies])."""
    subs = []
    text, _ = _scan(line, 0, len(line), "top", subs)
    return text, subs


def strip_comments(line):
    """Remove comments but KEEP quoted text. Used where the question is textual
    (does this file reference $PMAT_BIN?) rather than syntactic."""
    out, i, n, q = [], 0, len(line), ""
    while i < n:
        c = line[i]
        if c == "\\" and i + 1 < n:
            out.append(line[i:i + 2]); i += 2; continue
        if q:
            if c == q:
                q = ""
            out.append(c); i += 1; continue
        if c in ("'", '"'):
            q = c; out.append(c); i += 1; continue
        if c == "#":
            prev = out[-1] if out else ""
            if not out or prev.isspace() or prev in "(;&|":
                break
            out.append(c); i += 1; continue
        out.append(c); i += 1
    return "".join(out)


SPLIT = re.compile(r"(\$\(|[();|&{}]|&&|\|\|)")


def tokens(s):
    return [t for t in SPLIT.sub(r" \1 ", s).split() if t]


def _follow(toks, j, hits, probes):
    """Index j is a command position. Record it, then follow wrapper prefixes."""
    while j < len(toks):
        t = toks[j]
        if ASSIGN.match(t) and "=" in t:
            j += 1
            continue
        if t in PROBES:
            _probe_args(toks, j + 1, probes)
            return
        hits.add(j)
        if t in PREFIX:
            k = j + 1
            saw_probe_flag = False
            while k < len(toks) and (
                toks[k].startswith("-") or DURATION.match(toks[k])
                or (ASSIGN.match(toks[k]) and "=" in toks[k])
            ):
                if t == "command" and re.match(r"^-[a-zA-Z]*[vVp]", toks[k]):
                    saw_probe_flag = True
                k += 1
            if saw_probe_flag:
                _probe_args(toks, k, probes)
                return
            j = k
            continue
        return


def _probe_args(toks, j, probes):
    """Every non-option word a probe is handed is a PATH question about it."""
    while j < len(toks):
        t = toks[j]
        if t in SEPARATORS:
            return
        if not t.startswith("-"):
            probes.add(j)
        j += 1


def command_positions(toks):
    hits, probes = set(), set()
    expect = True
    i = 0
    while i < len(toks):
        t = toks[i]
        if expect:
            if t in SEPARATORS or (ASSIGN.match(t) and "=" in t):
                i += 1
                continue
            _follow(toks, i, hits, probes)
            expect = False
            i += 1
            continue
        if t in SEPARATORS:
            expect = True
        i += 1
    return hits, probes


def wrapper_positions(toks):
    """Command position created by this repo's own runner wrappers."""
    hits, probes = set(), set()
    for i, t in enumerate(toks):
        if t == "run_to" and i + 2 < len(toks):
            _follow(toks, i + 2, hits, probes)
        elif t == "run_split" and i + 3 < len(toks):
            _follow(toks, i + 3, hits, probes)
        elif t == "gate" and i + 2 < len(toks):
            _follow(toks, i + 2, hits, probes)
    return hits, probes


def for_list_positions(toks):
    """`for t in bashrs pmat probador` — the loop body's command word is a
    variable, so no command-position scan can see the tool. The literal list is
    where it IS visible."""
    out = set()
    for i, t in enumerate(toks):
        if t == "in" and i >= 2 and toks[i - 2] == "for":
            j = i + 1
            while j < len(toks) and toks[j] not in (";", "do"):
                out.add(j)
                j += 1
    return out


def scan_units(text):
    """Yield (lineno, unit_text) for every span of the file that is CODE.

    Heredoc bodies are data unless the introducing command is a shell — a
    quoted python heredoc holding the word `apr` is not an invocation, and
    `bash <<'EOF'` holding `pmat verify` is.
    """
    lines = text.splitlines()
    i, n = 0, len(lines)
    while i < n:
        raw = lines[i]
        lineno = i + 1
        yield lineno, raw
        stripped_for_hd = strip_comments(raw)
        m = HEREDOC.search(stripped_for_hd)
        if not m:
            i += 1
            continue
        delim = m.group(2)
        head_toks = tokens(strip_line(raw)[0])
        is_shell = any(t in SHELL_CMDS for t in head_toks[:2])
        i += 1
        while i < n and lines[i].strip() != delim:
            if is_shell:
                yield i + 1, lines[i]
            i += 1
        i += 1


def findings(path, text):
    out = []
    for lineno, raw in scan_units(text):
        main, subs = strip_line(raw)
        for unit, is_sub in [(main, False)] + [(s, True) for s in subs]:
            if not unit.strip():
                continue
            toks = tokens(unit)
            hits, probes = command_positions(toks)
            whits, wprobes = wrapper_positions(toks)
            hits |= whits
            probes |= wprobes
            fors = set() if is_sub else for_list_positions(toks)
            for idx in sorted(hits):
                if idx < len(toks) and toks[idx] in VERIFIERS:
                    out.append("%s:%d: %s [bare]" % (path, lineno, toks[idx]))
            for idx in sorted(probes):
                if idx < len(toks) and toks[idx] in VERIFIERS:
                    out.append("%s:%d: %s [PATH probe]" % (path, lineno, toks[idx]))
            for idx in sorted(fors):
                if idx < len(toks) and toks[idx] in VERIFIERS:
                    out.append("%s:%d: %s [for-list]" % (path, lineno, toks[idx]))
    # One finding per line/token/kind; a line can otherwise report twice when a
    # construct is both a wrapper argument and a command position.
    seen, uniq = set(), []
    for f in out:
        if f not in seen:
            seen.add(f)
            uniq.append(f)
    return uniq


rc = 0
for p in sys.argv[1:]:
    try:
        text = open(p, encoding="utf-8", errors="replace").read()
    except OSError as e:
        print("SCANERROR %s %s" % (p, e)); rc = 2; continue
    for f in findings(p, text):
        print(f); rc = 1
sys.exit(rc)
PY
}

# ---------------------------------------------------------------------------
# THE KNOWN-GAP TIER (#2644 audit, VP-01/02/08/09 — tracked by QUAL-015).
#
# Four shapes this tokeniser gets wrong are recorded here as EXECUTABLE
# fixtures asserting the CURRENT behaviour, not as prose. The reason they are
# not patched: the regex-scanning approach has now been wrong sixteen times in
# this programme, and nine more patches buys the seventeenth. The replacement
# is a parser-grade scanner (QUAL-015); this corpus is its acceptance test.
#
# Two properties make this a ratchet rather than a TODO:
#   · a gap that gets WORSE turns the row red (the assertion is exact), and
#   · a gap that gets FIXED also turns the row red, telling the fixer to
#     promote the case into the real table above. A known gap cannot be
#     silently closed and left undocumented, and it cannot silently widen.
#
# What is NOT here is as important: none of these is a false GREEN on the
# repo's current sources — PART 1 scans clean over the real scope, and the
# gaps are shapes no in-scope file currently uses. They are latent, and the
# corpus is what keeps them from becoming live without anyone noticing.
known_gap_table() {
    local td gaps=0 got
    td=$(mktemp -d) || return 2

    # VP-01 — command position after a leading redirection, through `eval`,
    # and as a `find -exec` payload. All four RUN a bare verifier.
    # Built with printf and indirection, exactly like the MUST-FLAG table
    # above: writing the literal tokens into this file would put a bare
    # verifier and an indirect-execution builtin in this guard's own source,
    # which its own PART 1 (and bashrs SEC001) would then read as real code.
    local M=pmat P=pv A=apr EV=eval
    {
        printf '>/dev/null %s verify\n' "$M"
        printf '2>/dev/null %s lint contracts\n' "$P"
        printf '%s %s comply check\n' "$EV" "$M"
        printf 'find . -name "*.apr" -exec %s qa {} \\;\n' "$A"
    } > "$td/g-cmdpos.sh"
    got=$(scan "$td/g-cmdpos.sh"; printf 'rc=%s' $?)
    if [ "$got" = "rc=0" ]; then
        printf 'GAP   scanner    redirect-prefixed / indirect / find -exec bare invocations are MISSED (VP-01, QUAL-015)\n'
    else
        printf 'FAIL  scanner    VP-01 behaviour CHANGED — promote these cases into the\n'
        printf '                 MUST-FLAG table above and delete this row:\n%s\n' "$got"; gaps=1
    fi

    # VP-02 — heredoc detection runs on text that still carries quoted spans,
    # so a string literal merely NAMING `<< WORD` opens a fake heredoc and
    # swallows every line after it.
    cat > "$td/g-fakehd.sh" <<'EOF'
echo "the cmd << EOF form is documented above"
pmat verify --format json
EOF
    got=$(scan "$td/g-fakehd.sh"; printf 'rc=%s' $?)
    if [ "$got" = "rc=0" ]; then
        printf 'GAP   scanner    a quoted mention of `<< WORD` suppresses the REST OF THE FILE (VP-02, QUAL-015)\n'
    else
        printf 'FAIL  scanner    VP-02 behaviour CHANGED — promote into MUST-FLAG and\n'
        printf '                 delete this row:\n%s\n' "$got"; gaps=1
    fi

    # VP-09 — no continuation joining, so the first word of a continued line
    # is read as command position. A false POSITIVE: argument-position words.
    printf 'echo the pinned tools are \\\n  pv pmat apr\n' > "$td/g-cont.sh"
    got=$(scan "$td/g-cont.sh" | awk -F: '{print $3}' | tr -d ' ' | tr '\n' ' ')
    if [ "$got" = "pv[bare] " ]; then
        printf 'GAP   scanner    a backslash-continuation argument list FALSE-POSITIVES (VP-09, QUAL-015)\n'
    else
        printf 'FAIL  scanner    VP-09 behaviour CHANGED — if fixed, move this into\n'
        printf '                 MUST-NOT-FLAG; got [%s]\n' "$got"; gaps=1
    fi

    # VP-08 — pin_audit has no heredoc awareness (unlike the PART 1 scanner),
    # so a `$PMAT_BIN` mention inside a DATA heredoc counts as consumption
    # while the gates consume something else entirely.
    cat > "$td/g-hdconsume.sh" <<'EOF'
verifier_pin_pmat "$CRATE" "$BINPATH"
gate pmat-verify /some/other/binary verify
PV=""
verifier_pin_pv
run_to "$L" "$PV" lint contracts
cat <<'DOC'
the pinned binary is $PMAT_BIN
DOC
EOF
    if pin_audit "$td/g-hdconsume.sh" >/dev/null 2>&1; then
        printf 'GAP   pin_audit  `$PMAT_BIN` inside a DATA heredoc counts as consumption (VP-08, QUAL-015)\n'
    else
        printf 'FAIL  pin_audit  VP-08 behaviour CHANGED — promote into the CALL-SITE\n'
        printf '                 table above and delete this row\n'; gaps=1
    fi

    rm -rf "${td:?}"
    return "$gaps"
}

# ---------------------------------------------------------------------------
# PART 1b's audit. Prints "ROW <ok|FAIL> <text>"; exits 1 if any row failed.
#
# The question here is NOT "is there a bare token" — PART 1 answers that, and it
# answers it GREEN on a runner that defines a perfect pin and never calls it.
# The question is whether the pin is the thing the gates consume.
pin_audit() {
    python3 - "$1" <<'PY'
import re, sys

path = sys.argv[1]
lines = open(path, encoding="utf-8", errors="replace").read().splitlines()


def strip_comments(line):
    out, i, n, q = [], 0, len(line), ""
    while i < n:
        c = line[i]
        if c == "\\" and i + 1 < n:
            out.append(line[i:i + 2]); i += 2; continue
        if q:
            if c == q:
                q = ""
            out.append(c); i += 1; continue
        if c in ("'", '"'):
            q = c; out.append(c); i += 1; continue
        if c == "#":
            prev = out[-1] if out else ""
            if not out or prev.isspace() or prev in "(;&|":
                break
            out.append(c); i += 1; continue
        out.append(c); i += 1
    return "".join(out)


code = [(n + 1, strip_comments(l)) for n, l in enumerate(lines)]

# The left-context class carries the FAIL-CLOSED call shapes too. It had
# then/else/do but not if/elif/while/until/!, so the most defensive spelling
# there is — `if ! verifier_pin_pmat "$C" "$B"; then exit 2; fi` — was reported
# as "the runner never CALLS verifier_pin_pmat" (#2644 audit, VP-07). A guard
# that rejects the most careful form of the thing it demands trains people to
# write the careless one.
_LEFT = r"(?:^|[;&|(){}]|\bthen\b|\belse\b|\bdo\b|\bif\b|\belif\b|\bwhile\b|\buntil\b|!)\s*"
CALL = {
    "pmat": re.compile(_LEFT + r"verifier_pin_pmat\b"),
    "pv": re.compile(_LEFT + r"verifier_pin_pv\b"),
}
USE = {
    "pmat": re.compile(r"\$\{?PMAT_BIN\b"),
    "pv": re.compile(r"\$\{?PV\}?(?![A-Za-z0-9_])"),
}
# An assignment to the pinned variable in the RUNNER is a bypass: it is exactly
# the mutation `PMAT_BIN=pmat`, and a token scan cannot distinguish it from any
# other assignment. Empty initialisation stays legal — the runner sets PV="" so
# an unpinned repo leaves it unmistakably unset.
# `^\s*` was line-anchored and prefix-blind, so the exact mutation this row
# exists to catch survived in two trivial spellings — `export PMAT_BIN=/stale`
# placed right after the genuine pin call, and a post-semicolon reassignment
# (#2644 audit, VP-03). Declarators are matched as prefixes; a bare `readonly
# PMAT_BIN` / `export PV` with no `=` is legitimate and still passes, as does
# empty initialisation (`PV=""`), which is how an unpinned repo says so.
BYPASS = re.compile(
    _LEFT
    + r"""(?:export\s+|readonly\s+|declare\s+(?:-[A-Za-z]+\s+)*|local\s+)?"""
    + r"""(PMAT_BIN|PV)=(?!(""|''|\s|$))"""
)

rc = 0
for tool in ("pmat", "pv"):
    calls = [n for n, l in code if CALL[tool].search(l)]
    uses = [n for n, l in code if USE[tool].search(l)]
    if not calls:
        print("ROW FAIL %s: the runner never CALLS verifier_pin_%s. A pin that is"
              " defined and not invoked leaves every gate on PATH." % (tool, tool))
        rc = 1
    elif not uses:
        print("ROW FAIL %s: the runner calls verifier_pin_%s but never uses the"
              " result — the gates are consuming something else." % (tool, tool))
        rc = 1
    elif min(calls) > min(uses):
        print("ROW FAIL %s: first use of the pin is line %d, but the pin is not"
              " called until line %d — that use reads an unset variable."
              % (tool, min(uses), min(calls)))
        rc = 1
    else:
        print("ROW ok %s: pinned at line %d, consumed %d time(s), first use line %d"
              % (tool, min(calls), len(uses), min(uses)))

# .search(), not .match(): `.match()` anchors at position 0, so widening
# the pattern's left-context class alone left the post-semicolon bypass
# accepted — the anchor lived in the CALL, not only the pattern (found by
# this table's own new row while fixing VP-03).
bypass = [(n, l.strip()) for n, l in code if BYPASS.search(l)]
if bypass:
    for n, l in bypass:
        print("ROW FAIL bypass at line %d: %s" % (n, l[:80]))
    print("ROW FAIL the runner assigns a pinned verifier variable directly. Only"
          " scripts/verifier_pin.sh may decide what PMAT_BIN/PV hold.")
    rc = 1
else:
    print("ROW ok no direct PMAT_BIN=/PV= assignment in the runner")
sys.exit(rc)
PY
}

# ---------------------------------------------------------------------------
# The scan universe: the protocol itself, plus every gate the runner DISCOVERS
# and EXECUTES inside the release verdict. Derived from the manifest so a gate
# added there cannot escape it. An empty derivation is a FAILURE, not a pass:
# a guard sweeping a set it failed to build is the vacuous green this whole
# protocol exists to refuse.
# The scan universe is the set the RUNNER EXECUTES, read through the runner's
# own parser — not a second opinion about the same declaration.
#
# This was an awk+grep scrape for `"..."` strings ending in `.sh`, while
# scripts/dogfood.sh executes EVERY declared string with `bash "$dg_path"`.
# So a gate declared as `scripts/check_x` (no suffix), or with TOML literal
# quotes, was RUN by the release and never SCANNED for unpinned verifiers —
# a guard universe strictly smaller than the surface it guards, which is the
# failure class this guard exists to catch (#2644 audit, CI-3 / VP-05 / F2).
resolve_scope() {
    local declared meta crate rc
    meta=$(mktemp) || { printf 'SCOPE_ERROR'; return 1; }
    if ! ( cd "$REPO_ROOT" && cargo metadata --no-deps --format-version 1 ) \
           > "$meta" 2>/dev/null; then
        rm -f "$meta"; printf 'SCOPE_ERROR'; return 1
    fi
    crate=$(sed -n 's/^name = "\(.*\)"/\1/p' "$REPO_ROOT/Cargo.toml" | head -1)
    declared=$(CRATE="$crate" python3 "$REPO_ROOT/scripts/lib/dogfood_gates.py" "$meta" \
        | awk '$1=="GATE"{print $2}')
    rc=$?
    rm -f "$meta"
    # NOPKG/NODECL/EMPTY/BADSHAPE all yield no GATE lines: the declaration
    # verifies nothing, and a scan over an empty universe would report clean.
    if [ "$rc" -ne 0 ] || [ -z "$declared" ]; then
        printf 'SCOPE_ERROR'
        return 1
    fi
    printf '%s\n%s\n' "$(printf '%s\n' $CORE_SCOPE)" "$declared" | awk 'NF && !seen[$0]++'
    return 0
}

# ---------------------------------------------------------------------------
# PART 1 case table. The nine-plus violating lines are ASSEMBLED from $M/$P/$A
# rather than written out, so this file contains no literal bare invocation of
# its own. Otherwise the sibling guard scripts/check_apr_bin_pinned.sh — which
# scans every script for exactly this construct — reports the case table as real
# violations, and THIS guard now scans itself too (it is a declared gate).
# Fixtures that trip a neighbouring guard get that guard an exemption entry, and
# an exemption is how a guard stops guarding.
self_test() {
    local td fails=0 got want
    td=$(mktemp -d) || return 2

    local M=pmat P=pv A=apr
    {
        printf 'gate %s-verify %s verify --format json\n' "$M" "$M"
        printf 'run_to "$WORKLOG/x.log" timeout 900 %s query "x" --limit 1\n' "$M"
        printf 'run_split "$W/a.json" "$W/b.err" timeout 900 %s comply check\n' "$M"
        printf '%s validate "$c" >/dev/null 2>&1\n' "$P"
        printf '%s qa model.apr\n' "$A"
        printf 'foo && %s comply check\n' "$M"
        printf 'OUT=$(%s lint contracts)\n' "$P"
        printf 'PATH=/stale:$PATH %s verify\n' "$M"
        printf 'if %s validate x; then :; fi\n' "$P"
        # 10 — MAJOR 1, the live construct: command substitution INSIDE a
        # double-quoted string. This was the receipt line naming which pmat ran.
        printf 'mark tools PASS "pmat $(%s --version | head -1)"\n' "$M"
        # 11 — its generalised form.
        printf 'OUT="$(%s lint contracts)"\n' "$P"
        # 12 — an UNescaped backtick inside a double-quoted string is still a
        # substitution. The escaped form is row 6 of MUST-NOT-FLAG.
        printf 'mark tools PASS "ver `%s --version`"\n' "$P"
        # 13 — a wrapper prefix carrying its own option-with-argument.
        printf 'nice -n 5 %s verify\n' "$M"
        # 14/15/16 — PATH probes. They do not run the tool; they decide whether
        # a gate runs, and they ask PATH instead of the pin.
        printf 'command -v %s >/dev/null 2>&1\n' "$M"
        printf 'HAVE=$(command -v %s)\n' "$A"
        printf 'type -aP %s\n' "$P"
        # 17 — the name in a `for` word list, whose loop body probes "$t".
        printf 'for t in bashrs %s probador; do command -v "$t"; done\n' "$M"
        # 18 — `command` as a transparent prefix, which the OPENERS set missed.
        printf 'command %s verify\n' "$M"
    } > "$td/bad.sh"

    # MUST NOT FLAG
    cat > "$td/good.sh" <<'EOF'
# `pv lint <DIR>` is a real gate and is run separately below.
mark pv-contracts REPORT "pv is not pinned in this repo -- contracts NOT validated."
run_to "$WORKLOG/pv-pc.log" "$PV" validate "$WORKLOG/bogus-contract.yaml"
gate pmat-verify "$PMAT_BIN" verify --format json
run_to "$WORKLOG/pmat-index.log" timeout 900 "$PMAT_BIN" query "x" --limit 1
for t in bashrs probador; do
echo "pmat"
. scripts/apr_bin.sh || exit 1
#   run_to "$LOG" pmat query "x"
mark pmat-verify SKIP "package has no lib target"
command -v "$PMAT_BIN" >/dev/null 2>&1
command -v bashrs >/dev/null 2>&1
VER="$("$PMAT_BIN" --version | head -1)"
nice -n 5 "$PMAT_BIN" verify
LIT='$(pmat --version)'
mark pv-bindings FAIL "\`pv verify-bindings\` produced no verification line"
EOF

    # A DATA heredoc holds no invocation; a SHELL heredoc does. Separate
    # fixtures because these are multi-line and the table above is by line.
    cat > "$td/hd-data.sh" <<'EOF'
python3 - "$1" <<'PY'
print("apr")
pv = 1
PY
EOF
    cat > "$td/hd-shell.sh" <<'EOF'
bash <<'EOF2'
pmat verify --format json
EOF2
EOF

    got=$(scan "$td/bad.sh" | awk -F: '{print $2}' | tr '\n' ' ')
    want="1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 "
    if [ "$got" = "$want" ]; then
        printf 'ok    MUST-FLAG     all 18 bare/probe/for-list invocations reported\n'
    else
        printf 'FAIL  MUST-FLAG     got lines [%s], want [%s]\n' "$got" "$want"; fails=1
    fi

    got=$(scan "$td/good.sh" | tr '\n' ' ')
    if [ -z "$got" ]; then
        # count derived from the fixture, never restated: the prose "all 14"
        # had drifted from a 16-line corpus (#2644 audit, VP-10 rider)
        printf 'ok    MUST-NOT-FLAG all %s pinned/comment/string/probe fixture lines accepted\n' "$(grep -c . "$td/good.sh")"
    else
        printf 'FAIL  MUST-NOT-FLAG false positives: %s\n' "$got"; fails=1
    fi

    got=$(scan "$td/hd-data.sh" | tr '\n' ' ')
    if [ -z "$got" ]; then
        printf 'ok    HEREDOC-DATA  a python heredoc naming a verifier is not an invocation\n'
    else
        printf 'FAIL  HEREDOC-DATA  false positives: %s\n' "$got"; fails=1
    fi

    got=$(scan "$td/hd-shell.sh" | awk -F: '{print $2}' | tr '\n' ' ')
    if [ "$got" = "2 " ]; then
        printf 'ok    HEREDOC-SHELL a `bash <<EOF` heredoc IS scanned as code\n'
    else
        printf 'FAIL  HEREDOC-SHELL got [%s], want [2 ]\n' "$got"; fails=1
    fi

    # PART 1b's own table: a runner that defines the pin and bypasses it.
    cat > "$td/bypass.sh" <<'EOF'
PMAT_BIN=pmat
gate pmat-verify "$PMAT_BIN" verify
PV=""
verifier_pin_pv
run_to "$L" "$PV" lint contracts
EOF
    got=$(pin_audit "$td/bypass.sh" 2>&1)
    if printf '%s' "$got" | grep -q 'ROW FAIL pmat' && printf '%s' "$got" | grep -q 'ROW FAIL bypass'; then
        printf 'ok    CALL-SITE     `PMAT_BIN=pmat` instead of the pin call is REJECTED\n'
    else
        printf 'FAIL  CALL-SITE     a runner that assigns PMAT_BIN itself was accepted:\n%s\n' "$got"; fails=1
    fi

    cat > "$td/late.sh" <<'EOF'
gate pmat-verify "$PMAT_BIN" verify
verifier_pin_pmat "$CRATE" "$BINPATH"
PV=""
verifier_pin_pv
run_to "$L" "$PV" lint contracts
EOF
    got=$(pin_audit "$td/late.sh" 2>&1)
    if printf '%s' "$got" | grep -q 'ROW FAIL pmat'; then
        printf 'ok    CALL-SITE     a pin called AFTER its first use is REJECTED\n'
    else
        printf 'FAIL  CALL-SITE     a pin called after its first use was accepted:\n%s\n' "$got"; fails=1
    fi

    # VP-03's two live spellings of the SAME bypass this table already covers
    # in its line-start form. Both were accepted before #2644.
    cat > "$td/bypass-export.sh" <<'EOF'
verifier_pin_pmat "$CRATE" "$BINPATH"
export PMAT_BIN=/opt/stale/pmat
gate pmat-verify "$PMAT_BIN" verify
PV=""
verifier_pin_pv
run_to "$L" "$PV" lint contracts
EOF
    got=$(pin_audit "$td/bypass-export.sh" 2>&1)
    if grep -q 'ROW FAIL bypass' <<< "$got"; then
        printf 'ok    CALL-SITE     `export PMAT_BIN=...` after the pin call is REJECTED\n'
    else
        printf 'FAIL  CALL-SITE     an `export` bypass was accepted:\n%s\n' "$got"; fails=1
    fi

    cat > "$td/bypass-semi.sh" <<'EOF'
verifier_pin_pmat "$CRATE" "$BINPATH"
true; PMAT_BIN=/opt/stale/pmat
gate pmat-verify "$PMAT_BIN" verify
PV=""
verifier_pin_pv
run_to "$L" "$PV" lint contracts
EOF
    got=$(pin_audit "$td/bypass-semi.sh" 2>&1)
    if grep -q 'ROW FAIL bypass' <<< "$got"; then
        printf 'ok    CALL-SITE     a post-semicolon PMAT_BIN= bypass is REJECTED\n'
    else
        printf 'FAIL  CALL-SITE     a post-semicolon bypass was accepted:\n%s\n' "$got"; fails=1
    fi

    # VP-07: the FAIL-CLOSED call shape must count as a call. Rejecting the
    # most careful spelling of the rule is a false positive that teaches the
    # careless one.
    cat > "$td/failclosed-calls.sh" <<'EOF'
if ! verifier_pin_pmat "$CRATE" "$BINPATH"; then
  echo "pin failed" >&2
  exit 2
fi
gate pmat-verify "$PMAT_BIN" verify
PV=""
if ! verifier_pin_pv; then MISSING="$MISSING pv"; fi
run_to "$L" "$PV" lint contracts
EOF
    if pin_audit "$td/failclosed-calls.sh" >/dev/null 2>&1; then
        printf 'ok    CALL-SITE     `if ! verifier_pin_*` counts as calling the pin\n'
    else
        printf 'FAIL  CALL-SITE     a fail-closed call shape was reported as "never CALLS":\n'
        pin_audit "$td/failclosed-calls.sh" 2>&1 | sed 's/^/        /'; fails=1
    fi

    # Legitimate declarators with no assignment must stay legal — the runner
    # itself ends both pins with `readonly`.
    cat > "$td/readonly-ok.sh" <<'EOF'
verifier_pin_pmat "$CRATE" "$BINPATH"
readonly PMAT_BIN
gate pmat-verify "$PMAT_BIN" verify
PV=""
verifier_pin_pv
readonly PV
run_to "$L" "$PV" lint contracts
EOF
    if pin_audit "$td/readonly-ok.sh" >/dev/null 2>&1; then
        printf 'ok    CALL-SITE     `readonly PMAT_BIN` (no assignment) stays legal\n'
    else
        printf 'FAIL  CALL-SITE     the widened bypass regex now rejects a legal declarator:\n'
        pin_audit "$td/readonly-ok.sh" 2>&1 | sed 's/^/        /'; fails=1
    fi

    cat > "$td/good-calls.sh" <<'EOF'
verifier_pin_pmat "$CRATE" "$BINPATH"
gate pmat-verify "$PMAT_BIN" verify
PV=""
verifier_pin_pv
run_to "$L" "$PV" lint contracts
EOF
    if pin_audit "$td/good-calls.sh" >/dev/null 2>&1; then
        printf 'ok    CALL-SITE     a runner that calls both pins before use is accepted\n'
    else
        printf 'FAIL  CALL-SITE     a correct runner was rejected:\n'
        pin_audit "$td/good-calls.sh" 2>&1 | sed 's/^/        /'; fails=1
    fi

    rm -rf "${td:?}"
    return "$fails"
}

# Cargo.lock with `[[patch.unused]]` blocks and blank lines removed. See the
# note at row 5 of behaviour_test for why those blocks are not staleness.
lock_norm() {
    awk '/^\[\[patch\.unused\]\]/ { skip=1; next }
         skip { if ($0 == "") skip=0; next }
         $0 != ""' "$1"
}

# ---------------------------------------------------------------------------
# PART 2. The pins must BEHAVE. Presence of `PMAT_BIN` proves nothing about
# which binary the gate ends up running, which is the only thing that matters.
behaviour_test() {
    local td fails=0 built stale
    td=$(mktemp -d) || return 2

    # This function builds pv, and a cargo build without --locked REWRITES a
    # stale Cargo.lock in place. That is the exact failure scripts/
    # check_lockfile_current.sh exists to catch, so a guard that silently
    # repaired the lock on its way past would disarm its neighbour. Snapshot it,
    # restore it, and REPORT if the build moved it.
    #
    # The snapshot is a FILE COPY, not `$(cat …)`: command substitution strips
    # trailing newlines, so a variable round-trip restores a file that differs
    # from the original in its last byte. The first version of this row did that
    # and its own restore was not byte-identical.
    [ -f "$REPO_ROOT/Cargo.lock" ] && cp "$REPO_ROOT/Cargo.lock" "$td/Cargo.lock.before"

    # shellcheck source=/dev/null
    if ! . "$REPO_ROOT/scripts/verifier_pin.sh"; then
        printf 'FAIL  pins       scripts/verifier_pin.sh could not be sourced\n'
        rm -rf "${td:?}"; return 1
    fi

    # A STALE pmat, first on PATH. This is the binary the gate must NOT pick.
    mkdir -p "$td/stalebin"
    stale="$td/stalebin/pmat"
    printf '#!/bin/sh\necho stale-pmat\n' > "$stale"
    chmod +x "$stale"
    built="$td/target-release-pmat"
    printf '#!/bin/sh\necho built-pmat\n' > "$built"
    chmod +x "$built"

    # Row 1 — the self-referential case: releasing pmat itself.
    PMAT_BIN=""
    PATH="$td/stalebin:$PATH" verifier_pin_pmat "pmat" "$built"
    if [ "$PMAT_BIN" = "$built" ]; then
        printf 'ok    pmat-pin   releasing pmat selects the BUILT artifact, not the PATH copy\n'
    else
        printf 'FAIL  pmat-pin   releasing pmat resolved to [%s]; the stale PATH pmat at %s\n' "$PMAT_BIN" "$stale"
        printf '                 would have measured a different build than the one shipping.\n'
        fails=1
    fi

    # Row 2 — every OTHER crate: PATH is correct there and must stay the answer.
    PMAT_BIN=""
    verifier_pin_pmat "aprender" "$built"
    if [ "$PMAT_BIN" = "pmat" ]; then
        printf 'ok    pmat-pin   a non-pmat crate still uses the fleet pmat\n'
    else
        printf 'FAIL  pmat-pin   non-pmat crate resolved to [%s], expected the PATH pmat\n' "$PMAT_BIN"; fails=1
    fi

    # Row 3 — a crate named pmat with NO built artifact FAILS CLOSED: rc=1 and
    # an EMPTY pin. This row used to bless the silent PATH fallback — the exact
    # stale-3.32.0-measuring-3.32.0 incident the lib's header records (#2644,
    # VPIN-1). The old expectation is preserved here as the mutation direction:
    # a lib reverted to the fallback turns this row RED.
    PMAT_BIN=""
    verifier_pin_pmat "pmat" ""
    r3_rc=$?
    if [ "$r3_rc" -eq 1 ] && [ -z "$PMAT_BIN" ]; then
        printf 'ok    pmat-pin   no built artifact -> rc=1, EMPTY pin (fail closed, no PATH fallback)\n'
    else
        printf 'FAIL  pmat-pin   no built artifact resolved to [%s] rc=%s — a silent\n' "$PMAT_BIN" "$r3_rc"
        printf '                 fallback here measures a PATH pmat that is not the build\n'
        printf '                 being released (the recorded incident, again)\n'
        fails=1
    fi

    # Row 3b — behavior, not existence: a directory and a broken stub both pass
    # `-x`; neither can answer `--version`; neither may become the pin (#2644,
    # VPIN-2).
    mkdir -p "$td/notabinary"
    PMAT_BIN=""
    verifier_pin_pmat "pmat" "$td/notabinary"
    r3b_rc=$?
    printf '#!/bin/sh\nexit 97\n' > "$td/brokenstub"
    chmod +x "$td/brokenstub"
    PMAT_BIN=""
    verifier_pin_pmat "pmat" "$td/brokenstub"
    r3c_rc=$?
    if [ "$r3b_rc" -eq 1 ] && [ "$r3c_rc" -eq 1 ] && [ -z "$PMAT_BIN" ]; then
        printf 'ok    pmat-pin   a directory and a --version-mute stub are both REFUSED\n'
    else
        printf 'FAIL  pmat-pin   non-working artifact accepted (dir rc=%s, stub rc=%s, pin=[%s])\n' "$r3b_rc" "$r3c_rc" "$PMAT_BIN"
        fails=1
    fi

    # Row 3d — delivery: the pin must reach a CHILD process. Every discovered
    # gate is one, and an unexported pin is consumption pin_audit certifies but
    # the environment never carries (#2644, VPIN-4).
    PMAT_BIN=""
    printf '#!/bin/sh\necho built-pmat\n' > "$td/okstub"
    chmod +x "$td/okstub"
    verifier_pin_pmat "pmat" "$td/okstub"
    r3d_child=$(bash -c 'printf %s "${PMAT_BIN:-UNSET}"')
    if [ "$r3d_child" = "$td/okstub" ]; then
        printf 'ok    pmat-pin   the pin arrives in a child process (exported)\n'
    else
        printf 'FAIL  pmat-pin   child saw [%s] — the pin is shell-local and every\n' "$r3d_child"
        printf '                 discovered gate runs unpinned\n'
        fails=1
    fi

    # Row 4 — pv. The pin must not be whatever PATH offers. A decoy `pv` goes
    # first on PATH; the resolved PV must differ from it.
    mkdir -p "$td/pvbin"
    printf '#!/bin/sh\necho "pv 0.0.0-decoy"\n' > "$td/pvbin/pv"
    chmod +x "$td/pvbin/pv"
    (
        cd "$REPO_ROOT" || exit 2
        PATH="$td/pvbin:$PATH"
        export PATH
        PV=""
        verifier_pin_pv
        pv_rc=$?
        # The decoy is named directly, not via `command -v`: this file must not
        # itself contain a PATH resolution of a verifier, and naming the path we
        # planted is strictly more precise than asking PATH what it found.
        decoy="$td/pvbin/pv"
        if [ "$pv_rc" -eq 2 ]; then
            printf 'FAIL  pv-pin     this repo SHIPS scripts/pv_bin.sh but the pin reported "unpinned"\n'
            exit 1
        fi
        if [ "$pv_rc" -ne 0 ]; then
            printf 'FAIL  pv-pin     the pin failed to resolve pv (rc=%s) — a release cannot be\n' "$pv_rc"
            printf '                 decided by a verifier that did not build.\n'
            exit 1
        fi
        if [ "$PV" = "$decoy" ]; then
            printf 'FAIL  pv-pin     resolved pv IS the PATH decoy (%s) — the pin was bypassed\n' "$decoy"
            exit 1
        fi
        printf 'ok    pv-pin     resolved pv is %s, NOT the PATH decoy %s\n' "$PV" "$decoy"
    ) || fails=1

    # Row 4b — the PV_BIN environment channel is CLEARED before the pin sources
    # pv_bin.sh: an inherited PV_BIN short-circuits the cargo build pv_bin.sh
    # itself names "THE FRESHNESS AUTHORITY", so an exported stale path would
    # ride straight through an otherwise-green pin (#2644, VPIN-6/VP-04).
    (
        cd "$REPO_ROOT" || exit 2
        printf '#!/bin/sh\necho pv 0.0.0-poison\n' > "$td/pv-poison"
        chmod +x "$td/pv-poison"
        PV_BIN="$td/pv-poison"
        export PV_BIN
        PV=""
        verifier_pin_pv
        pv_rc=$?
        if [ "$pv_rc" -eq 0 ] && [ "$PV" = "$td/pv-poison" ]; then
            printf 'FAIL  pv-pin     an inherited PV_BIN (%s) rode through the pin —\n' "$PV_BIN"
            printf '                 the freshness authority was bypassed by the environment\n'
            exit 1
        fi
        if [ "$pv_rc" -ne 0 ]; then
            printf 'FAIL  pv-pin     pin failed under an inherited PV_BIN (rc=%s) — clearing\n' "$pv_rc"
            printf '                 the channel must not break resolution\n'
            exit 1
        fi
        printf 'ok    pv-pin     an inherited PV_BIN is cleared; the pin resolved %s\n' "$PV"
    ) || fails=1

    # Row 4c — the pin works from a SUBDIRECTORY of the repo. The cwd-relative
    # form returned 2 ("this repo ships no pin") from anywhere below the root —
    # a false statement that downgraded every pv gate to REPORT (#2644, VPIN-5).
    (
        cd "$REPO_ROOT/scripts" || exit 2
        PV=""
        verifier_pin_pv
        pv_rc=$?
        if [ "$pv_rc" -eq 2 ]; then
            printf 'FAIL  pv-pin     from scripts/ the pin says "this repo ships no pin" — the\n'
            printf '                 discovery is cwd-relative and lies from any subdirectory\n'
            exit 1
        fi
        if [ "$pv_rc" -ne 0 ] || [ -z "$PV" ]; then
            printf 'FAIL  pv-pin     pin failed from a subdirectory (rc=%s)\n' "$pv_rc"
            exit 1
        fi
        printf 'ok    pv-pin     the pin resolves from a subdirectory of the repo\n'
    ) || fails=1

    # Row 5 — this guard must not have moved Cargo.lock.
    #
    # `[[patch.unused]]` blocks are NORMALISED AWAY before comparing, and that is
    # not a loosening. They appear only when a host-local, gitignored
    # `.cargo/config.toml` injects a `[patch.crates-io]` of sibling checkouts —
    # the aprender dev-overrides do exactly that — and then ANY `cargo build -p
    # <one-member>` records which of those patches the sub-graph did not use.
    # They are absent in CI, they say nothing about whether the lock resolves,
    # and failing on them would make this row permanently red on every
    # developer box that ran `make dev-setup`. The resolution itself — the
    # `[[package]]` entries — is compared in full.
    if [ ! -f "$td/Cargo.lock.before" ]; then
        printf 'ok    lockfile   no Cargo.lock in this repo — nothing for the build to move\n'
    elif lock_norm "$td/Cargo.lock.before" > "$td/lock.a" \
         && lock_norm "$REPO_ROOT/Cargo.lock" > "$td/lock.b" \
         && cmp -s "$td/lock.a" "$td/lock.b"; then
        # Leave the tree exactly as it was found, even when the delta was inert.
        cmp -s "$td/Cargo.lock.before" "$REPO_ROOT/Cargo.lock" \
            || cp "$td/Cargo.lock.before" "$REPO_ROOT/Cargo.lock"
        printf 'ok    lockfile   building the pinned pv did not change the resolution\n'
    else
        cp "$td/Cargo.lock.before" "$REPO_ROOT/Cargo.lock"
        printf 'FAIL  lockfile   building the pinned pv REWROTE Cargo.lock (restored). The lock\n'
        printf '                 was stale before this ran; cargo repaired it in passing, which\n'
        printf '                 is exactly what scripts/check_lockfile_current.sh exists to see.\n'
        printf '                 Run `cargo update --workspace --offline` (or check_lockfile_current.sh)\n'
        printf '                 and commit the result.\n'
        fails=1
    fi

    rm -rf "${td:?}"
    return "$fails"
}

# ---------------------------------------------------------------------------
# PART 3. The fleet path: this runner exists to be pointed at OTHER repos, by
# the relative path its own Usage line documents. #2640 made the pin library
# load-bearing and fail-closed, while SKILL_DIR was still computed AFTER
# `cd "$REPO_DIR"` — so a relative ${BASH_SOURCE[0]} resolved against the TARGET
# repo, the library was "missing", and the runner exited 2 before any gate ran.
# Absolute invocation kept working, which is why nothing noticed.
fleet_path_test() {
    local td fails=0 out rc
    td=$(mktemp -d) || return 2

    mkdir -p "$td/fixture-crate/src"
    cat > "$td/fixture-crate/Cargo.toml" <<'EOF'
[package]
name = "dogfood-fixture"
version = "0.0.0"
edition = "2021"
publish = false

[package.metadata.dogfood]
gates = ["gate-ok.sh", "gate-pin-delivery.sh"]
EOF
    printf 'pub fn f() {}\n' > "$td/fixture-crate/src/lib.rs"
    printf '#!/usr/bin/env bash\nexit 0\n' > "$td/fixture-crate/gate-ok.sh"
    # The fixture is a REAL git repo with a clean tree: since the DF-7 fix a
    # non-git directory is a deliberate git-clean FAIL, and this test asserts
    # rc=0 through the runner. Hooks and identity are pinned so no host config
    # can dirty the run.
    git -C "$td/fixture-crate" -c init.defaultBranch=fixture init -q
    git -C "$td/fixture-crate" -c user.name=dogfood -c user.email=dogfood@fixture \
        -c core.hooksPath=/dev/null add -A
    git -C "$td/fixture-crate" -c user.name=dogfood -c user.email=dogfood@fixture \
        -c core.hooksPath=/dev/null commit -q --no-verify -m fixture
    # The committed catcher for the unexported-pin mutation (#2644, VPIN-4):
    # this gate runs as a CHILD of the runner, exactly like every discovered
    # gate, and asserts the pins ARRIVED. PMAT_BIN must be nonempty (the policy
    # answer for a non-pmat crate); PV must EXIST in the environment — this
    # fixture ships no pin, so its VALUE is legitimately empty, but an absent
    # VARIABLE means the runner resolved pins it never delivered.
    cat > "$td/fixture-crate/gate-pin-delivery.sh" <<'EOF'
#!/usr/bin/env bash
[ "${PMAT_BIN+set}" = set ] || { echo "PMAT_BIN not delivered to child env"; exit 1; }
[ -n "$PMAT_BIN" ] || { echo "PMAT_BIN empty for a non-pmat crate"; exit 1; }
[ "${PV+set}" = set ] || { echo "PV not delivered to child env"; exit 1; }
exit 0
EOF

    # DOGFOOD_GATES_ONLY stops after the declared-gate discovery section, which
    # sits well past the pin-library source. It is the runner's own hook, not a
    # copy of it, and it can never print GO.
    out=$( cd "$REPO_ROOT" && DOGFOOD_GATES_ONLY=1 \
        bash scripts/dogfood.sh "$td/fixture-crate" 2>&1 )
    rc=$?
    if [ "$rc" -eq 2 ]; then
        printf 'FAIL  fleet-path a RELATIVE `bash scripts/dogfood.sh <other-crate>` exited 2\n'
        printf '                 (setup error) before running anything. That is the form its own\n'
        printf '                 documented Usage line gives, and the whole reason the protocol is\n'
        printf '                 portable. Resolve SKILL_DIR from ${BASH_SOURCE[0]} BEFORE the\n'
        printf '                 `cd "$REPO_DIR"`, or the path resolves against the TARGET repo.\n'
        printf '%s\n' "$out" | tail -5 | sed 's/^/                 /'
        fails=1
    elif grep -q 'verifier_pin.sh is missing' <<< "$out"; then
        printf 'FAIL  fleet-path the pin library was not found under a relative invocation\n'
        fails=1
    elif ! grep -q 'DOGFOOD_GATES_ONLY' <<< "$out"; then
        printf 'FAIL  fleet-path the runner did not reach the declared-gate section (rc=%s)\n' "$rc"
        printf '%s\n' "$out" | tail -5 | sed 's/^/                 /'
        fails=1
    elif [ "$rc" -ne 0 ]; then
        # The fixture's gates include gate-pin-delivery.sh, the committed
        # catcher for the unexported-pin mutation. A nonzero rc here with the
        # section reached means a declared gate went RED — most likely the
        # probe reporting pins that never arrived. Printing "ok (rc=1)" at
        # this point would be a catcher that observes and does not gate.
        printf 'FAIL  fleet-path the fixture run reached the declared-gate section but exited %s\n' "$rc"
        printf '%s\n' "$out" | grep -E 'declared:|dogfood-gates|not delivered|empty for' | tail -6 | sed 's/^/                 /'
        fails=1
    elif ! grep -q 'declared:gate-pin-delivery' <<< "$out"; then
        printf 'FAIL  fleet-path the pin-delivery probe left no row in the output — a\n'
        printf '                 catcher that did not run is indistinguishable from one that passed\n'
        fails=1
    else
        printf 'ok    fleet-path relative invocation runs; pin-delivery probe green (rc=%s)\n' "$rc"
    fi

    rm -rf "${td:?}"
    return "$fails"
}

# ---------------------------------------------------------------------------
case "${1:-}" in
    --self-test)
        self_test; exit $?
        ;;
    --scan)
        shift; scan "$@"; exit $?
        ;;
esac

rc=0
printf -- '--- verifier pinning ------------------------------------------------\n'

printf 'case table (the tokeniser must be right before its verdict means anything)\n'
if self_test; then :; else rc=1; fi

printf '\nknown gaps (executable, asserted exactly — QUAL-015 replaces this scanner)\n'
if known_gap_table; then :; else rc=1; fi

printf '\nPART 1 — static: no bare verifier in command position\n'
cd "$REPO_ROOT" || exit 2
SCAN_ERR=$(mktemp)
trap 'rm -f "$SCAN_ERR"' EXIT
SCOPE=$(resolve_scope)
if [ "$SCOPE" = "SCOPE_ERROR" ]; then
    printf 'FAIL  [package.metadata.dogfood] gates yielded nothing — the scan universe\n'
    printf '      could not be built, so a clean sweep here would mean nothing.\n'
    rc=1
else
    missing=""
    for f in $SCOPE; do
        [ -f "$f" ] || missing="$missing $f"
    done
    if [ -n "$missing" ]; then
        # A scope entry that does not exist is a gate scanning nothing. The
        # runner could be renamed out from under this guard and it would sweep
        # an empty universe and report clean — the exact failure this repo
        # keeps finding.
        printf 'FAIL  in-scope file(s) missing:%s — the guard scanned an empty universe\n' "$missing"
        rc=1
    else
        # shellcheck disable=SC2086
        hits=$(scan $SCOPE 2>"$SCAN_ERR")
        scan_rc=$?
        # Only ENUMERATED (rc, output) shapes carry meanings: (0, empty) is
        # clean and (1, findings) is findings. Everything else — an input-
        # selective tokeniser crash is exit 1 with an empty stdout and a
        # traceback on stderr — is a scanner failure, and the old
        # rc==2-only branch printed "ok no bare pv/pmat/apr" over a scan that
        # scanned nothing (#2644 audit, F8 rider; v1.13 P1).
        if [ "$scan_rc" -eq 0 ] && [ -z "$hits" ]; then
            printf 'ok    no bare pv/pmat/apr in command position, in:\n'
            printf '%s\n' "$SCOPE" | sed 's/^/        /'
        elif [ "$scan_rc" -eq 1 ] && [ -n "$hits" ]; then
            printf 'FAIL  bare verifier invocation(s) — these resolve through PATH:\n'
            printf '%s\n' "$hits" | sed 's/^/      /'
            printf '      Use the pin: "$PV" / "$PMAT_BIN" / "$APR". See scripts/verifier_pin.sh.\n'
            rc=1
        else
            printf 'FAIL  the scanner did not produce a verdict (rc=%s, %s finding lines) — an\n' "$scan_rc" "$(grep -c . <<< "$hits")"
            printf '      unenumerated exit shape is a scan that cannot be believed:\n'
            sed 's/^/      /' "$SCAN_ERR" 2>/dev/null | tail -5
            printf '%s\n' "$hits" | grep . | sed 's/^/      /' | head -5
            rc=1
        fi
    fi
fi

printf '\nPART 1b — call site: the pin is CALLED, and its result is what runs\n'
audit=$(pin_audit "$REPO_ROOT/scripts/dogfood.sh")
audit_rc=$?
printf '%s\n' "$audit" | sed -e 's/^ROW ok /ok    /' -e 's/^ROW FAIL /FAIL  /' -e 's/^\(ok\|FAIL\)/\1/'
[ "$audit_rc" -eq 0 ] || rc=1

printf '\nPART 2 — behavioural: the pins select something other than PATH\n'
if behaviour_test; then :; else rc=1; fi

printf '\nPART 3 — fleet path: the runner runs when invoked relatively\n'
if fleet_path_test; then :; else rc=1; fi

printf '\n'
if [ "$rc" -eq 0 ]; then
    printf 'PASS  the release runner resolves every pinned verifier through its pin.\n'
else
    printf 'FAIL  see rows above. A gate measured with an unknown binary is not a gate.\n'
fi
exit "$rc"
