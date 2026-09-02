# CRUX audit — preserved measurement artefacts (2026-09-02)

Raw evidence behind `docs/specifications/pmat-architecture-crux-audit.md`. The
spec's §1 Provenance block warned that its `research/` citations resolved only
inside an ephemeral scratch directory; this directory is that hedge, committed so
the numbers stay checkable after the session that produced them is gone.

## What is here

| path | what it is |
|---|---|
| `m/<name>.time` | GNU `/usr/bin/time -v` for one command: wall clock, max RSS, CPU%, exit status |
| `m/<name>.meta` | the exact command, its working directory, an ISO timestamp, and the `uptime` at that instant |
| `build{1,2,3}.time` | `cargo build --release` timings — 1 is the near-cold build, 2 the incremental no-op, 3 the instrumented repeat |
| `build{1,2,3}.stderr` | cargo's own output for those builds |
| `measure.sh`, `measure2.sh`, `build.sh`, `build3.sh` | the harnesses that produced the above, kept so a reader can re-run them |
| `measure.progress` | the append-only log each run wrote as it went |

`m/<name>.out` and `m/<name>.err` are **not** committed — several are tens of MB
of JSON (`pmat context` alone emitted 8.67 MB). The `.time` and `.meta` pairs
carry every number the spec cites; regenerate the payloads with the harnesses.

## Reading a measurement

Each `.meta` names the command and the load average it ran under. That matters:
this is a shared 48-core host that also runs the self-hosted CI runners, and the
load average across the window was 0.76–32.14. Sub-second wall-clock deltas are
scheduling noise. **CPU-seconds and CPU% are the robust figures**, and the
findings rest on those.

```
$ cat m/repo_score.meta      # what was run, and under what load
$ grep -E 'Elapsed|Maximum resident|Percent of CPU' m/repo_score.time
```

## Provenance

Binary under test: a release build of `01fba4f65` (v3.34.0), verified by
`pmat --version` reporting that commit with a clean worktree. Every path inside
a `.meta` file is the absolute path as it existed during the run.
