import glob, subprocess, sys, yaml

bad = []
for f in sorted(glob.glob("contracts/*.yaml")):
    if f.endswith("binding.yaml"):
        continue
    if subprocess.run(["pv", "validate", f], capture_output=True).returncode != 0:
        bad.append(f"{f}: pv validate failed")
    doc = yaml.safe_load(open(f)) or {}
    # An obligation that names a TEST belongs under `falsification_tests`, which
    # is the key pv reads. Under `falsification` pv reports zero and the audit
    # it is supposed to perform silently covers nothing.
    hidden = [e for e in (doc.get("falsification") or [])
              if isinstance(e, dict) and "test" in e]
    if hidden:
        bad.append(f"{f}: {len(hidden)} test-bearing obligation(s) under "
                   f"`falsification:`, which pv cannot read — use `falsification_tests:`")
for b in bad:
    print(f"::error::{b}")
print(f"pv obligation gate: {len(bad)} problem(s)")
sys.exit(1 if bad else 0)
