# pmat quality hooks — feedback, not gates

`pmat-quality-feedback.sh` runs `pmat quality-gate --file <edited file>` when an
agent writes a Rust file, in Claude Code and in the Google Antigravity Agent.

## These are not gates, and cannot be

Both clients **fail open**.

> **Antigravity**, verbatim: *"If a hook script crashes (non-zero exit status),
> an HTTP hook returns a non-2xx status code, or an operation times out or
> returns unrecognized JSON, the runtime treats it as an approval (`allow`)."*

> **Claude Code**: exit 2 blocks; exit 1 blocks nothing. A hook binary that
> fails to launch is not a deny.

Demonstrated, not asserted — with `pmat` off `PATH`:

```
PATH=/usr/bin:/bin .agents/hooks/pmat-quality-feedback.sh claude       -> exit 0
PATH=/usr/bin:/bin .agents/hooks/pmat-quality-feedback.sh antigravity  -> {"decision":"allow"}
```

A quality gate that returns *allow* when it crashes is the inverse of Jidoka.
**The gate is `ci / gate` on protected `master`**, alongside `feature-gate`,
`docs build (docs.rs environment)` and `pmat score`. These hooks only shorten
the feedback loop from "CI, in 20 minutes" to "this edit, now".

Antigravity is additionally self-tamperable — an agent with filesystem write
tools can edit `.agents/hooks.json` or this script. Mount `.agents/` from a
read-only source if that matters.

## Exit-code translation, and why it is here

`pmat quality-gate` exits **1** on violations. Neither client treats 1 as a
block: Claude Code needs **2**, Antigravity needs `{"decision":"deny"}` on
stdout. The script translates; wiring either client straight to `pmat` would
produce a hook that runs, reports, and blocks nothing.

`pmat comply check --strict` is the wrong command for this, for three reasons:
it is project-scoped with no `--file`, it takes tens of seconds, and its exit
code does not separate failure from warnings.

## Behaviour

| situation | result |
|---|---|
| Rust file, violations | Claude `exit 2` / Antigravity `deny` |
| Rust file, clean | allow |
| non-Rust file, missing file, unparseable payload | allow |
| `pmat` not installed | **allow** — the documented fail-open limit |

Sub-second, because `--file` scopes it to the one file that changed.
