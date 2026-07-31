#!/usr/bin/env python3
"""Exercise a pmat release artifact against the behaviours previous releases broke.

Run it on the binary that `scripts/verify-artifact.sh` hands back, never on a
path typed by hand:

    BIN=$(scripts/verify-artifact.sh) && python3 scripts/dogfood-release.py "$BIN"

Each check corresponds to a defect that shipped, and each is known to FAIL on the
release before its fix -- verified by running this against v3.28.1 and v3.28.2.
A harness that has never gone red proves nothing.

The MCP one-shot check is the one that matters most: it is the check v3.28.2
claimed to pass (30/30) while the published artifact scored 11/30, because the
measurement was taken against a workspace build rather than an install build.
"""
import json
import subprocess
import sys
import tempfile
import os

# The binary under test must come from scripts/verify-artifact.sh, which builds
# with fresh dependency resolution and refuses to hand back a binary whose
# embedded commit SHA does not match HEAD. Passing a path by hand is what let
# v3.28.2 ship a fix that did not work.
if len(sys.argv) < 2:
    sys.exit("usage: dogfood.py <path-to-verified-pmat>\n"
             "       BIN=$(scripts/verify-artifact.sh) && dogfood.py \"$BIN\"")
BIN = sys.argv[1]
ENV = {"MCP_VERSION": "1", "PATH": "/usr/bin:/bin", "HOME": os.path.expanduser("~")}
results = []


def record(name, ok, detail=""):
    results.append((name, ok, detail))
    print(f"{'PASS' if ok else 'FAIL'}  {name}" + (f"  — {detail}" if detail else ""))


def run(args, **kw):
    return subprocess.run([BIN, *args], capture_output=True, text=True, timeout=300, **kw)


# 0. Right binary?
v = run(["--version"]).stdout.strip()
record("binary reports build provenance", "commit:" in v, v.replace(chr(10), " | "))

# 1. MCP one-shot: every consumed request answered (was 11/30, must be 30/30).
reqs = [
    {"jsonrpc": "2.0", "id": 1, "method": "initialize",
     "params": {"protocolVersion": "2024-11-05", "capabilities": {},
                "clientInfo": {"name": "dogfood", "version": "1"}}},
    {"jsonrpc": "2.0", "method": "notifications/initialized"},
    {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
]
payload = "".join(json.dumps(r) + "\n" for r in reqs)
answered = hangs = 0
TRIALS = 30
for _ in range(TRIALS):
    try:
        p = subprocess.run([BIN], input=payload, capture_output=True, text=True,
                           env=ENV, timeout=45)
    except subprocess.TimeoutExpired:
        hangs += 1
        continue
    ids = set()
    for line in p.stdout.splitlines():
        line = line.strip()
        if line.startswith("{"):
            try:
                m = json.loads(line)
            except json.JSONDecodeError:
                continue
            if "id" in m:
                ids.add(m["id"])
    if 2 in ids:
        answered += 1
record("MCP one-shot answers every request", answered == TRIALS and hangs == 0,
       f"{answered}/{TRIALS} answered, {hangs} hangs")

# 2. MCP still serves 20 tools with stdin held open.
p = subprocess.Popen([BIN], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                     stderr=subprocess.DEVNULL, text=True, bufsize=1, env=ENV)
tools = -1
try:
    for r in reqs:
        p.stdin.write(json.dumps(r) + "\n")
        p.stdin.flush()
    for _ in range(20):
        line = p.stdout.readline()
        if not line:
            break
        line = line.strip()
        if line.startswith("{"):
            m = json.loads(line)
            if m.get("id") == 2:
                tools = len(m["result"]["tools"])
                break
finally:
    p.stdin.close()
    p.wait(timeout=20)
record("MCP serves 20 tools", tools == 20, f"{tools} tools")

# 3. MCP rejects nonexistent paths (#639).
call = [reqs[0], reqs[1],
        {"jsonrpc": "2.0", "id": 3, "method": "tools/call",
         "params": {"name": "analyze_complexity",
                    "arguments": {"paths": ["/definitely-not-real-9f3a"]}}}]
p = subprocess.run([BIN], input="".join(json.dumps(r) + "\n" for r in call),
                   capture_output=True, text=True, env=ENV, timeout=120)
rejected = False
for line in p.stdout.splitlines():
    line = line.strip()
    if line.startswith("{"):
        m = json.loads(line)
        if m.get("id") == 3 and "error" in m:
            rejected = "not found" in json.dumps(m["error"])
record("MCP rejects nonexistent path", rejected)

# 4. CLI rejects nonexistent paths.
r = run(["analyze", "satd", "--path", "/definitely-not-real-9f3a"])
record("CLI satd rejects nonexistent path", r.returncode != 0)

# 5. five-whys withholds a cause it cannot derive (#637).
r = run(["five-whys", "the thing is broken", "--depth", "2", "--format", "markdown"])
record("five-whys withholds unlocatable cause", "Not determined" in r.stdout)
record("five-whys reports TDG honestly", "not measured" in r.stdout,
       "must not print 0.0/100 for an unmeasured metric")

# 6. five-whys locates a real issue and cites it.
with tempfile.TemporaryDirectory() as d:
    os.makedirs(os.path.join(d, "src"), exist_ok=True)
    with open(os.path.join(d, "src", "widget.rs"), "w") as f:
        f.write("fn widget_throttle(retries: u32) -> u32 {\n"
                "    if retries > 3 { retries * 2 } else { retries }\n}\n")
    r = run(["five-whys", "widget_throttle retries miscounted",
             "--depth", "3", "--format", "markdown", "--path", d])
    record("five-whys cites located code", "widget.rs" in r.stdout)
    # Confidence must not be the old constant 100%.
    record("five-whys confidence is not pinned to 100%",
           "**Confidence**: 100%" not in r.stdout)

# 7. serve hint and fatal-error visibility (3.28.1 contracts still hold).
r = run(["serve"])
hint = r.stdout + r.stderr
record("serve hint names only the working route",
       "MCP_VERSION=1" in hint and "agent mcp-server" not in hint and "PMAT_PMCP_MCP" not in hint)
r = subprocess.run([BIN, "agent", "mcp-server"], capture_output=True, text=True,
                   stdin=subprocess.DEVNULL, timeout=60)
record("fatal errors are never silent",
       r.returncode == 0 or r.stderr.strip() != "")

# ---------------------------------------------------------------------------
# INTERFACE COVERAGE
#
# Everything above pins a specific defect that shipped. The checks below are
# breadth instead: they walk each of pmat's three interfaces (CLI, MCP, HTTP)
# so a release cannot ship with a whole surface broken and unnoticed.
#
# This section exists because two separate features were found broken this
# release purely because nothing ever built them: `--features org-intelligence`
# could not compile its tests (E0004, a `Localize` arm never added), and the
# pmcp EOF fix regressed under a dependency set CI never resolved. Untested
# surfaces rot silently.
# ---------------------------------------------------------------------------

# 8. Every top-level subcommand can at least render its own help.
#    Catches broken clap wiring, which is invisible to a `--lib` test run.
help_out = run(["--help"])
# Parse ONLY the "Commands:" block. Scraping the whole help text picks up option
# descriptions ("Control", "Force", "Possible", ...) and reports them as broken
# subcommands -- a harness bug that produces a false red, which is worse than no
# check at all.
subcommands = []
in_commands = False
for line in help_out.stdout.splitlines():
    if line.startswith("Commands:"):
        in_commands = True
        continue
    if in_commands:
        if not line.strip() or not line.startswith(" "):
            break
        name = line.split()[0]
        if name != "help":  # `pmat help --help` is not a thing
            subcommands.append(name)
subcommands = sorted(set(subcommands))
broken = []
for sc in subcommands:
    try:
        r = run([sc, "--help"])
        if r.returncode != 0:
            broken.append(f"{sc}({r.returncode})")
    except subprocess.TimeoutExpired:
        broken.append(f"{sc}(timeout)")
record(f"CLI: all {len(subcommands)} subcommands render help", not broken,
       ", ".join(broken) if broken else f"{len(subcommands)} ok")

# 9. Core analyses produce real, parseable output on a real tree.
with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "src")
    os.makedirs(src, exist_ok=True)
    with open(os.path.join(src, "lib.rs"), "w") as f:
        f.write(
            "pub fn tangled(a: u32, b: u32, c: u32) -> u32 {\n"
            "    if a > 1 { if b > 2 { if c > 3 { return a + b + c; } } }\n"
            "    // TODO: this is self-admitted technical debt\n"
            "    let v: Option<u32> = None;\n"
            "    v.unwrap()\n"
            "}\n")
    with open(os.path.join(d, "Cargo.toml"), "w") as f:
        f.write('[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2021"\n')

    r = run(["analyze", "complexity", "--path", d, "--format", "json"])
    ok = r.returncode == 0 and r.stdout.strip().startswith(("{", "["))
    record("CLI: analyze complexity emits JSON", ok, f"rc={r.returncode}")

    r = run(["analyze", "satd", "--path", d, "--format", "json"])
    record("CLI: analyze satd emits JSON",
           r.returncode == 0 and r.stdout.strip().startswith(("{", "[")),
           f"rc={r.returncode}")

    # NB: `context` spells it --project-path, while `analyze *` uses --path and
    # `tdg` takes a positional. Inconsistent, but that is the real interface.
    r = run(["context", "--project-path", d, "--format", "json"])
    record("CLI: context emits JSON",
           r.returncode == 0 and r.stdout.strip().startswith(("{", "[")),
           f"rc={r.returncode}")

    r = run(["tdg", d, "--format", "json"])
    record("CLI: tdg emits JSON",
           r.returncode == 0 and r.stdout.strip().startswith(("{", "[")),
           f"rc={r.returncode}")

# 10. HTTP interface actually serves, not just prints a hint.
#     `serve` with no MCP_VERSION should bind and answer. Skipped rather than
#     failed if the build has no http-server feature, so a default-feature
#     artifact does not report a false red.
import socket
import time
import urllib.request

def free_port():
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    return port

port = free_port()
proc = subprocess.Popen([BIN, "serve", "--port", str(port)],
                        stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                        text=True, stdin=subprocess.DEVNULL)
served = None
deadline = time.time() + 30
while time.time() < deadline:
    if proc.poll() is not None:
        break
    try:
        with urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=2) as resp:
            served = resp.status
            break
    except Exception:
        try:
            with urllib.request.urlopen(f"http://127.0.0.1:{port}/", timeout=2) as resp:
                served = resp.status
                break
        except Exception:
            time.sleep(0.5)
out = ""
if proc.poll() is None:
    proc.terminate()
    try:
        out = proc.communicate(timeout=10)[0] or ""
    except subprocess.TimeoutExpired:
        proc.kill()
else:
    out = proc.communicate(timeout=5)[0] or ""

if served is not None:
    record("HTTP: serve binds and answers", served < 500, f"status {served}")
else:
    # `pmat serve` HTTP is deliberately unimplemented and says so
    # (utility_serve_handlers.rs). That is honest, so it is not a failure -- but
    # it means pmat ships TWO working interfaces, CLI and MCP, not three.
    unimplemented = "not yet implemented" in out
    record("HTTP: serve fails honestly when unimplemented",
           unimplemented and "MCP_VERSION=1" in out,
           "unimplemented + working hint" if unimplemented
           else f"did not bind and did not explain why: {out.strip()[:200]}")

    # If HTTP does not work, nothing shipped may claim it does. pmat has an
    # explicit zero-hallucination documentation policy; its own package
    # description is the one piece of metadata every crates.io visitor reads.
    if unimplemented:
        manifest = os.path.join(os.path.dirname(os.path.dirname(
            os.path.abspath(__file__))), "Cargo.toml")
        desc = ""
        try:
            for line in open(manifest):
                if line.startswith("description"):
                    desc = line
                    break
        except OSError:
            pass
        claims_http = "HTTP" in desc
        record("docs: package description does not claim unimplemented HTTP",
               not claims_http,
               desc.strip()[:160] if claims_http else "description matches reality")

# 11. `-V` stays short, `--version` carries provenance. Both must work:
#     scripts and packagers parse `-V`, humans and verify-artifact.sh read `--version`.
short = run(["-V"]).stdout.strip()
record("CLI: -V stays a single line", short.count("\n") == 0 and "commit:" not in short,
       short)

failed = [n for n, ok, _ in results if not ok]
print()
print(f"{len(results) - len(failed)}/{len(results)} checks passed")
if failed:
    print("FAILED:", ", ".join(failed))
    sys.exit(1)
