# The pre-release protocol lives HERE

Canon for the dogfood release gate is this directory, inside the repository it
gates. Not `~/.claude/skills/`, and not another repository's working tree.

| file | role |
|---|---|
| `pmat-dogfood-runner.sh` | the protocol: every release gate, the receipt, the GO/NO-GO verdict |
| `pmat-fleet-dogfood.sh` | SKILL §11 — the release binary against one real sibling repo |
| `pmat-transport-parity.sh` | SKILL §12 — one question asked over CLI, MCP stdio and HTTP |

## Why here

A canon under `~/.claude/skills/` is not in git, not PR-reviewed, not
CI-reachable and not diffable. Two divergent copies of this protocol had already
caused aprender#2640, and the fix chosen there was to make the skill a shim that
`exec`d `$HOME/src/aprender/scripts/dogfood.sh`.

That got the diagnosis right and the cure wrong: the target was a path in a
**different repository's working tree**, so the protocol's availability depended
on which branch that checkout happened to be on.

Measured 2026-08-23, mid-release: aprender sat on
`fix/wgpu-feature-surface-contract`, `scripts/dogfood.sh` was absent from its
working tree, and pmat's release gate exited 2 with *"the canonical runner is not
reachable"*. **A release protocol that cannot run is indistinguishable from no
release protocol**, and it failed in the one hour it was needed.

Three edits to shared directories were silently lost the same day for the same
underlying reason — uncommitted changes in trees this repo does not own. The
clean-room clippy fix was one of them; it is now committed to `paiml/infra`
(`1b4f01f`).

## Naming

Every file is prefixed `pmat-` so a symlink into a shared skills directory cannot
collide with another crate's runner. The names are unique across the fleet by
construction, not by luck.

## Installing the skill hook (idempotent)

`ln -sfn` replaces an existing link in place, so re-running changes nothing:

```sh
make dogfood-install     # or, equivalently:
ln -sfn "$PWD/scripts/dogfood/pmat-dogfood-runner.sh" \
        ~/.claude/skills/dogfood/pmat-dogfood-runner.sh
```

Run it twice; the second run is a no-op. That is the property the name and the
`-f`/`-n` flags exist to guarantee.

## Provenance

`pmat-dogfood-runner.sh` was restored from `paiml/aprender@6adc4bce4`,
`scripts/dogfood.sh`, 1,419 lines, vendored 2026-08-23. To re-vendor:

```sh
git -C ~/src/aprender show 6adc4bce4:scripts/dogfood.sh \
  > scripts/dogfood/pmat-dogfood-runner.sh
```

Update the sha in this file in the same edit, or the record becomes a lie.
