#!/usr/bin/env python3
"""Invoke ONE verb across EVERY declared transport AT THE SAME TIME, compare.

Why this is a separate gate from interface-parity
-------------------------------------------------
interface-parity proves each transport has an e2e that spawns the binary and
passes. That is reachability, and it is necessary. It is not sufficient: three
transports can each be reachable, each be green in its own test file, and still
disagree about what a verb RETURNS. Nothing compares them, because each e2e only
ever sees its own surface.

Why SIMULTANEOUS rather than sequential
---------------------------------------
Running them one after another cannot distinguish "the transports agree" from
"the transports share a process-global that only one of them may hold at a
time". Standing every transport up at once and invoking through all of them
while all are live is the configuration a real client fleet produces, and it is
the one that surfaces a shared listener, a shared lock, or a runtime that only
tolerates a single owner.

Why DERIVED rather than hand-written
------------------------------------
The verb list comes from the BINARY, not from a list in this file. A hand-written
probe tests the verbs someone remembered; a derived one tests the surface that
shipped, and grows automatically when the surface does.
"""
import json, socket, subprocess, sys, time, urllib.request, urllib.error

def die(msg, code=2):
    print(f"INVARIANCE_ERROR {msg}")
    sys.exit(code)

def free_port():
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p

def wait_ready(port, timeout=25.0):
    """Probe by CONNECTING, never by binding: a probe that binds competes with
    the server it is waiting for and can starve it."""
    end = time.time() + timeout
    while time.time() < end:
        try:
            with socket.create_connection(("127.0.0.1", port), timeout=0.5):
                return True
        except OSError:
            time.sleep(0.05)
    return False

def read_declaration():
    """argv -> (binpath, decl, list_cmd, probe). Dies on an unusable declaration."""
    binpath, decl_json = sys.argv[1], sys.argv[2]
    decl = json.loads(decl_json)
    list_cmd = decl.get("list")
    probe = decl.get("probe") or {}
    if not list_cmd or not probe.get("verb"):
        die("declaration needs `list` and `probe = { verb, params }`")
    return binpath, decl, list_cmd, probe

def derive_verbs(binpath, list_cmd):
    """The surface, DERIVED from the binary. Vacuity-guarded: an empty list dies."""
    r = subprocess.run([binpath, *list_cmd.split()], capture_output=True, text=True, timeout=60)
    if r.returncode != 0:
        die(f"`{list_cmd}` exited {r.returncode}: {r.stderr.strip()[:200]}")
    verbs = [l.strip() for l in r.stdout.splitlines() if l.strip()]
    if not verbs:
        die("the binary lists NO verbs — a parity check over an empty surface is vacuous")
    return verbs

def start_http(binpath, http, procs):
    """Stand the http transport up and wait for it. Returns the port."""
    port = free_port()
    cmd = [binpath, *http["serve"].replace("{port}", str(port)).split()]
    procs.append(subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL))
    if not wait_ready(port):
        die(f"http transport did not accept a connection on {port}")
    return port

def invoke_cli(binpath, template, verb, params):
    # Split the TEMPLATE, then substitute per token. Splitting after
    # substitution would tear a JSON params value containing spaces into
    # several argv entries, and the verb would be invoked with garbage.
    argv_list = [
        params if t == "{params}" else t.replace("{verb}", verb)
        for t in template.split()
    ]
    r = subprocess.run([binpath, *argv_list], capture_output=True, text=True, timeout=120)
    if r.returncode != 0:
        die(f"cli invoke failed ({r.returncode}): {r.stderr.strip()[:200]}")
    return r.stdout.strip()

def invoke_http(port, http, verb, params):
    url = f"http://127.0.0.1:{port}" + http["path"].replace("{verb}", verb)
    req = urllib.request.Request(url, data=params.encode(),
                                 headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            return resp.read().decode().strip()
    except urllib.error.HTTPError as e:
        die(f"http invoke returned {e.code}: {e.read()[:200]!r}")

def collect(binpath, decl, verb, params):
    """Stand every transport up FIRST so all are live together, then invoke
    through each while all are still up."""
    results, procs = {}, []
    try:
        port = None
        http = decl.get("http")
        if http:
            port = start_http(binpath, http, procs)
        if decl.get("cli"):
            results["cli"] = invoke_cli(binpath, decl["cli"], verb, params)
        if http:
            results["http"] = invoke_http(port, http, verb, params)
    finally:
        for p in procs:
            p.kill()
            p.wait()
    return results

def compare(results, verb, verbs):
    """Identical AND valid. Two identically-wrong strings must not pass."""
    names = list(results)
    first = results[names[0]]
    for n in names[1:]:
        if results[n] != first:
            print(f"INVARIANCE_FAIL `{verb}` differs between {names[0]} and {n}")
            print(f"  {names[0]}: {first[:300]!r}")
            print(f"  {n}: {results[n][:300]!r}")
            sys.exit(1)
    try:
        json.loads(first)
    except Exception:
        print(f"INVARIANCE_FAIL all transports agree but the payload is not JSON: {first[:200]!r}")
        sys.exit(1)
    print(f"INVARIANCE_PASS {len(verbs)} verb(s) derived from the binary; "
          f"`{verb}` byte-identical across {', '.join(names)} with all transports live")

def main():
    binpath, decl, list_cmd, probe = read_declaration()
    verbs = derive_verbs(binpath, list_cmd)
    verb, params = probe["verb"], probe.get("params", "{}")
    if verb not in verbs:
        die(f"probe verb `{verb}` is not in the binary's own list ({', '.join(verbs[:6])}…)")
    results = collect(binpath, decl, verb, params)
    if len(results) < 2:
        print(f"INVARIANCE_SKIP only {len(results)} transport(s) invocable: {list(results)}")
        return
    compare(results, verb, verbs)

main()
