# IMPL-PMAT-695 — hermetic build: the vendored demo assets

Ticket: PMAT-695 (GitHub #1156)
Branch: PMAT-695-hermetic-build (from PMAT-691-board)
Orchestrator turns: n/a (worker)

## Defect

`build.rs::get_asset_definitions` listed four upstream URLs and
`download_and_compress_assets` fetched them over the network at build time into
`assets/vendor/`, which `.gitignore` excluded. The script then gzipped each
file, wrote a `*.hash` beside it, and read its own output back as a build input
(`src/demo/assets.rs` embeds the `.gz` forms with `include_bytes!`). Two of the
four URLs were `@latest`:

    https://unpkg.com/gridjs@6.0.6/dist/gridjs.umd.js
    https://unpkg.com/gridjs@6.0.6/dist/theme/mermaid.min.css
    https://unpkg.com/mermaid@latest/dist/mermaid.min.js
    https://unpkg.com/d3@latest/dist/d3.min.js

So the build had a network input, was non-deterministic by construction, and a
"clean-room" pass proved only that the machine had a route out. On docs.rs the
`DOCS_RS` branch wrote placeholder text and an empty `.gz` instead, and a failed
fetch wrote `/* Asset download failed during build */` over the asset — both
silent.

## Fix

The four `.gz` files are committed under `assets/vendor/` with a committed
`SHA256SUMS` (repository-root relative names, so `sha256sum -c
assets/vendor/SHA256SUMS` works from the root) and a committed `PROVENANCE.md`
that pins every upstream URL to an exact version. The uncompressed originals
stay ignored: nothing reads them, and `mermaid.min.js` alone is 2,754,895 bytes
against crates.io's 10 MiB ceiling.

`build.rs` now: reads `SHA256SUMS`, verifies every file it names (a mismatch or
a missing file is a hard build error naming that file), still emits
`ASSET_HASH`, and performs no network access — the fetch, the docs.rs
placeholder branch and the gzip step are deleted. Verification runs on EVERY
build, not only `--features demo`, so an edited asset cannot reach a build that
embeds it. The five new `cargo:rerun-if-changed=` directives name tracked files
individually; the directory is never watched.

### Upstream versions pinned (`@latest` resolved)

| committed file | upstream (exact version) | uncompressed sha256 | bytes |
| --- | --- | --- | --- |
| `gridjs.min.js.gz` | gridjs 6.0.6 `dist/gridjs.umd.js` | `865cc398a589292ad3d206287d7b9feb226f7e7755b561c8de1c20e6a05d890e` | 51,836 |
| `gridjs-mermaid.min.css.gz` | gridjs 6.0.6 `dist/theme/mermaid.min.css` | `ab9585e3983a57267a8f22f708fe40ad70f8c1bd5688ebfba31d11a0c7cca331` | 7,774 |
| `mermaid.min.js.gz` | mermaid **11.12.2** (was `@latest`) | `d0830a6c05546e9edb8fe20a8f545f3e0dc7c4c3134d584bad9c13a99d7a71e0` | 2,754,895 |
| `d3.min.js.gz` | d3 **7.9.0** (was `@latest`) | `f2094bbf6141b359722c4fe454eb6c4b0f0e42cc10cc7af921fc158fceb86539` | 279,706 |

The versions were read from the banner inside each file already present in the
working tree (d3's leading comment, mermaid's embedded `version:"…"`); the
uncompressed digests are the ones the old script had recorded in `*.hash`, so
they identify the exact bytes these `.gz` were produced from.

## RED (commit 944579d61, before the fix)

    running 4 tests
    test ...::build_script_has_no_network_input ... FAILED
    test ...::no_rerun_if_changed_on_ignored_paths ... FAILED
    test ...::vendored_assets_are_tracked ... FAILED
    test ...::vendored_assets_match_the_committed_checksums ... FAILED

    ---- build_script_has_no_network_input ----
    build.rs must contain no network input, found "unpkg.com" on: [
        "            \"https://unpkg.com/gridjs@6.0.6/dist/gridjs.umd.js\",",
        "            \"https://unpkg.com/gridjs@6.0.6/dist/theme/mermaid.min.css\",",
        "            \"https://unpkg.com/mermaid@latest/dist/mermaid.min.js\",",
        "        (\"https://unpkg.com/d3@latest/dist/d3.min.js\", \"d3.min.js\"),",
    ]
    ---- vendored_assets_match_the_committed_checksums ----
    assets/vendor/SHA256SUMS must be committed at …/assets/vendor/SHA256SUMS:
    Err(Os { code: 2, kind: NotFound, message: "No such file or directory" })
    ---- vendored_assets_are_tracked ----
    assets/vendor/SHA256SUMS must be tracked, git knows: []
    ---- no_rerun_if_changed_on_ignored_paths ----
    assets/vendor/SHA256SUMS must be committed: Err(… NotFound …)

    test result: FAILED. 0 passed; 4 failed; 0 ignored; 21409 filtered out

## GREEN (after the fix)

    running 4 tests
    test ...::build_script_has_no_network_input ... ok
    test ...::vendored_assets_are_tracked ... ok
    test ...::vendored_assets_match_the_committed_checksums ... ok
    test ...::no_rerun_if_changed_on_ignored_paths ... ok

    test result: ok. 4 passed; 0 failed; 0 ignored; 21409 filtered out

    $ sha256sum -c assets/vendor/SHA256SUMS
    assets/vendor/d3.min.js.gz: OK
    assets/vendor/gridjs-mermaid.min.css.gz: OK
    assets/vendor/gridjs.min.js.gz: OK
    assets/vendor/mermaid.min.js.gz: OK
    exit 0

`build_support::tests::rerun_if_changed_paths_exist_inside_the_tree` — the
pre-existing CRUX-06 hygiene test that pins the literal watch set and allows
exactly two interpolated sites — also passes with the five new watches.

## Falsifiers

1. **No network namespace.** `unshare -rn env CARGO_TARGET_DIR=… HOME=…
   PATH=… cargo build --locked` → `Finished dev profile … in 27.29s`, **exit
   0**. The build completes with no route to any network at all.
2. **Checksums.** `sha256sum -c assets/vendor/SHA256SUMS` → **exit 0** (4 OK).
3. **Mutation — the gate is live.** One byte flipped in a committed asset
   (`printf '\x00' | dd of=assets/vendor/gridjs.min.js.gz bs=1 seek=10 count=1
   conv=notrunc`), then `cargo build --locked`:

        error: failed to run custom build command for `pmat v3.39.0`
          thread 'main' panicked at build.rs:264:25:
          PMAT-695: vendored asset assets/vendor/gridjs.min.js.gz does not match
          assets/vendor/SHA256SUMS (recorded 256b0539…45c6, found 96da6980…c162b).
          Restore the file from git, or regenerate SHA256SUMS and PROVENANCE.md
          if the change is intended.

   **exit 101**, the file named. Restored with `git checkout` (checksums OK
   again). The same mutation was run first against `d3.min.js.gz`, with the
   same result.

## Crate size

| | bytes | of the 10,000,000 ceiling |
| --- | --- | --- |
| before (published 3.39.0, assets NOT packaged) | 9,031,935 | 90.3% |
| after (`cargo package --locked --no-verify`) | **9,956,257** | 99.6% |

    $ cargo package --list | grep assets/vendor
    assets/vendor/PROVENANCE.md
    assets/vendor/SHA256SUMS
    assets/vendor/d3.min.js.gz
    assets/vendor/gridjs-mermaid.min.css.gz
    assets/vendor/gridjs.min.js.gz
    assets/vendor/mermaid.min.js.gz

The delta is +924,322 bytes, almost all of it `mermaid.min.js.gz` (791,872).
**43,743 bytes of headroom remain** — the next asset, or any large addition to
the packaged tree, will not fit. That is a real constraint this change creates
and it is recorded here rather than discovered by a 503 on upload.

## Gate

`pmat verify --format json` (run with `TMPDIR=/tmp/tmpx`, which avoids the
unrelated PMAT-689 `/tmp/CHANGELOG.md` collision):

- after the fix, before the ledgers: `ok:false`, `stages_measured:5` — format,
  complexity, satd, clippy all `ok:true`; `tests` failed on exactly one test,
  `services::unrun_tests::tests::the_committed_ledger_matches_the_tree`
  ("RENDERED TEXT DIFFERS"), i.e. the new test module's own ledger row.
- the two ledgers were then rewritten from a clean tree at HEAD
  (`analyze unrun-tests --write-ledger`, `analyze reachability --write-ledger`,
  the latter refusing a dirty tree until the fix was committed) and committed.

## Not done, and why

- `ureq` is still declared under `[build-dependencies]` in `Cargo.toml`. Nothing
  calls it now, but removing a dependency rewrites `Cargo.lock`, which is
  outside this ticket's scope_paths, and `cargo build --locked` would then
  refuse. Left for a follow-up that may touch the lock.
- `calculate_asset_hash` (the `ASSET_HASH` cache-buster) still hashes
  `read_dir` output, whose order is not specified. It is unchanged by this
  ticket and affects only a demo cache-busting string.

## Quorum (agy `--mode plan`, width 3, review-only)

Lanes 0f588512-f871-4754-9432-e4b02319e914, 87497a3c-3d11-40ab-8b36-f12c60fbcfaa, 26dc1488-f988-42b7-a185-77e10f872259 — 3/3 needs-changes, none do-not-merge. Every lane affirmed the central claim: the network fetch is gone, the four `.gz` are committed and packaged (6 vendored files of 4,982 paths in `cargo package --list`), `verify_vendored_assets()` is called unconditionally from `main()`, a missing file hard-fails, and the watches name only tracked files.

| # | finding | lanes | disposition |
|---|---|---|---|
| 1 | `rows.len() >= 4` is a count floor — nothing required the four *specific* assets to be named, so a renamed or duplicated row could stand in for a dropped one | 3/3 | **fixed**: the four names are asserted individually. Mutation: drop the gridjs row from SHA256SUMS → `PMAT-695: assets/vendor/SHA256SUMS does not name assets/vendor/gridjs.min.js.gz`, build fails |
| 2 | the test's banned-substring list (`unpkg.com`, `https://`, `http://`, `download`) applies to the whole file including doc comments | 2/3 | kept: 0 hits today; a future doc comment naming a URL *should* be re-examined, which is what the test says |
| 3 | `PROVENANCE.md` "falsely claims versions were read from the banner" | 2/3 (lane 1 called it a blocker) | **refuted** by lane 3 and by the delegate: the sentence is scoped to "the two `@latest` specifiers" (d3, mermaid), both of which carry banners; gridjs was pinned at 6.0.6 in the pre-fix script and was never `@latest` |
| 4 | `ASSET_HASH` is derived from an unsorted `fs::read_dir` | 3/3 | pre-existing, untouched by this diff; not absorbed (§2 emergent-findings protocol, S3) |
| 5 | the receipt's crate size came from `--no-verify` | 1/3 | the number is the tarball size, which is what the 10 MB ceiling measures; the verify build runs in CI (`feature-matrix.yml:495`, `release.yml:101`), so a non-compiling package is caught there |
| 6 | stray untracked `pre_fix_build.rs` at the worktree root | delegate | deleted |

Delegate note the lanes did not reach, recorded because it is the actual mechanism behind "a size regression is caught": the `include_bytes!` block is double-gated (`#[cfg(feature = "demo")]` and `not(cargo_publish)`), and `demo` is not in `default` — so CI's verify build never embeds these assets. What proves they are in the tarball is `verify_vendored_assets()` running unconditionally rather than under `CARGO_FEATURE_DEMO`.

Brief premise corrected before the lanes ran: the pre-fix `build.rs` did **not** watch the gitignored `assets/vendor/` (it deliberately did not, per its own comment). This branch watches *more* — five newly tracked files — rather than stopping an ignored watch.

verdict: PASS — the build script performs no network access, the four assets are committed and verified against a committed SHA256SUMS on every build (mutation → exit 101 naming the file), `unshare -rn cargo build --locked` exits 0, and the package is 9,956,257 bytes, under the 10,000,000 ceiling.

IMPL-PMAT-695-RECEIPT-END
