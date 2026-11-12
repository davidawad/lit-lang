# Stale `path_sha256` pins (lg-pb1fe.56)

## What the existing check compared

- `crates/litigate-engine/src/cite/corpus.rs:87-98` hashes text held in a
  `SourceCorpus`, and `:117-124` loads `sources/` from disk only when
  `LITGRAPH_SOURCES` is set; otherwise `:110-113` returns the binary's
  embedded corpus.
- `crates/litigate-engine/build.rs:28-38` embeds every `sources/*.txt` at
  build time when the `embed-sources` feature is enabled.
- `crates/litigate-sema/src/pkg/manifest.rs:506-542` validates the syntax
  and presence relationship for `path` and `path_sha256`, then stores both
  strings in the manifest. It does not read the named file.
- `crates/litigate-sema/src/pkg/load.rs:120-137` hashes each L0 file on disk
  into `LoadedPackage::texts`; `crates/litigate-sema/src/types/build/package.rs`
  uses those hashes for `Lit.lock`, not to compare the manifest's
  `path_sha256`.
- `crates/litigate-verify/src/cite.rs:800-845` compares a binding's pin with
  `citations.pin(id)`, which reads the selected `SourceCorpus`. Consequently
  it checked embedded text by default (or override-directory text when
  `LITGRAPH_SOURCES` was set), and only while verifying a cite. The three
  bindings passed because this check did not compare their pins with the
  current disk file, and no cite forced an error on those paths.

`litigate check` now compares each pinned `sources/` path's disk digest with
the binding pin in both profiles. It reports L1010 at the manifest pin and
offers a machine-applicable hash replacement. A per-run path map reuses the
disk digest across package bindings. When disk text and the embedded corpus
differ, the same source diagnostic identifies both hashes and states that
cite checks use the embedded side and how to rebuild (or select
`LITGRAPH_SOURCES`). With an explicit `LITGRAPH_SOURCES` override, checks use
that directory and do not report the binary as the selected corpus.

## Pin decisions

In each case the extract is authoritative: its recorded extraction digest
matches the file, while the pin does not. Only the manifest pin was changed.

| Binding | Evidence | Correct `path_sha256` |
|---|---|---|
| `lit/core`, FRBP 9006 | `git show d98c50e8 -- sources/frbp/frbp-9006-2024-12-01.txt` shows the Nov. 3, 2026 change intentionally removed 6,981 unrelated lines, retaining the Rule 9006 heading. `sources/frbp/PROVENANCE.md` records the served PDF SHA-256 `e78659c1dcf9ee050415d2f4916776f5f55989bde21d5c8fe025b5d14ad9acd3` and extraction SHA-256 `7f1969f4148d3c5d965f6ebe0073392ca90dcc5a33e5abaaec7f375d0aaaf1a6`; the original PDF is recorded at that served hash. | `7f1969f4148d3c5d965f6ebe0073392ca90dcc5a33e5abaaec7f375d0aaaf1a6` |
| `lit/core`, Georgia 2026 holidays | The source was added in `67f10683`; `sources/ga-state-civil/PROVENANCE.md` records the extraction SHA-256 `4d365d905e2c0a5e96b166d2311048ed90b770fe68c3a9f6aea7ba393f55e41b`, which matches the extract. Its original served-page SHA-256 remains recorded separately as `7d7a50b093e50849062262a66f96c918ffd8f01177fb60394cb3ea2a1f59d0c0`. | `4d365d905e2c0a5e96b166d2311048ed90b770fe68c3a9f6aea7ba393f55e41b` |
| `lit/sd-criminal-trial-court`, SDCL Title 23A | `git show 37c54f3e -- sources/sd-criminal-trial-court/sd-sdcl-title-23a-criminal-procedure.txt` documents the Oct. 28, 2026 re-extraction: corrected repeated `<html><head>` pairs, restored 31 empty and 16 truncated chapters, with originals unchanged. The provenance's extraction digest `42cc33cb64dc2b201bf37bdbcb9aa6aa6bc5b511c8eb20862b89263f99eaaf0c` matches the file. | `42cc33cb64dc2b201bf37bdbcb9aa6aa6bc5b511c8eb20862b89263f99eaaf0c` |

## 28 U.S.C. § 2107 expression date

The vendored GovInfo 2008 page's amendment history says: “1991&mdash;Pub.
L. 102&ndash;198 designated first and second pars. as subsecs. (a) and
(b), respectively, added subsec. (c), designated fifth par. as subsec.
(d), and struck out third and fourth pars. which read as follows:”. The
law is dated Dec. 9, 1991 (Pub. L. 102-198), so
`docs/litigate/change/frap-base/Lit.toml` now uses `as_of = "1991-12-09"`.
The original HTML matches the provenance SHA-256
`d44f221964296f6036179e7b8746e5019953acc7440e56abcf975bdeaaee805b`.
