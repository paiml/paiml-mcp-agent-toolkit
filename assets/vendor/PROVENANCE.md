# Vendored demo assets — provenance

PMAT-695 (#1156). These four files are **committed**, not fetched. `build.rs`
used to download them from unpkg at build time — two of them pinned to
`@latest` — into this directory, which was gitignored, gzip them, and then
treat its own output as a build input. That made every build depend on a
network route and on whatever upstream had published that day, and it meant a
"clean-room" build passed only because the machine could reach the internet.

What is committed here is the **gzip** form of each asset, because that is what
the crate reads: `src/demo/assets.rs` embeds each `.gz` with `include_bytes!`
and inflates it at runtime. The uncompressed originals are not committed and
stay gitignored — nothing reads them, and `mermaid.min.js` alone is 2,754,895
bytes against crates.io's 10 MiB package ceiling.

`build.rs` verifies every file listed in `SHA256SUMS` on every build and fails,
naming the file, on a mismatch. Verify by hand with:

```sh
sha256sum -c assets/vendor/SHA256SUMS
```

## Upstream sources, exactly pinned

The two `@latest` specifiers are replaced below by the version actually
present, read from the version banner inside each file (`d3` carries it in its
leading comment, `mermaid` in its embedded `version:"…"` field).

| committed file | upstream URL (exact version) | upstream SHA-256 (uncompressed) | uncompressed bytes |
| --- | --- | --- | --- |
| `gridjs.min.js.gz` | `https://unpkg.com/gridjs@6.0.6/dist/gridjs.umd.js` | `865cc398a589292ad3d206287d7b9feb226f7e7755b561c8de1c20e6a05d890e` | 51,836 |
| `gridjs-mermaid.min.css.gz` | `https://unpkg.com/gridjs@6.0.6/dist/theme/mermaid.min.css` | `ab9585e3983a57267a8f22f708fe40ad70f8c1bd5688ebfba31d11a0c7cca331` | 7,774 |
| `mermaid.min.js.gz` | `https://unpkg.com/mermaid@11.12.2/dist/mermaid.min.js` | `d0830a6c05546e9edb8fe20a8f545f3e0dc7c4c3134d584bad9c13a99d7a71e0` | 2,754,895 |
| `d3.min.js.gz` | `https://unpkg.com/d3@7.9.0/dist/d3.min.js` | `f2094bbf6141b359722c4fe454eb6c4b0f0e42cc10cc7af921fc158fceb86539` | 279,706 |

The uncompressed digests above are the ones the previous build script recorded
beside each asset (`*.hash`), so they identify the exact upstream bytes these
`.gz` files were produced from.

## Licences

- `gridjs` 6.0.6 — MIT (Grid.js contributors)
- `mermaid` 11.12.2 — MIT (Knut Sveidqvist and contributors)
- `d3` 7.9.0 — ISC (Mike Bostock)

## Updating an asset

1. Fetch the new file by hand, **outside** the build, at an exact version.
2. `gzip -9 -n -c <file> > assets/vendor/<file>.gz` (`-n` keeps it byte-stable).
3. Regenerate the checksums **from the repository root**, so the recorded
   names stay root-relative and `sha256sum -c assets/vendor/SHA256SUMS` works
   there: `sha256sum assets/vendor/*.gz > assets/vendor/SHA256SUMS`.
4. Update the table above with the new version and digest.
5. Commit the `.gz`, `SHA256SUMS` and this file together.
