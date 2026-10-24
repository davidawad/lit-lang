# Contributing to litgraph

Thanks for looking at litgraph. Contributions are welcome from humans and
agents alike — see [AGENTS.md](AGENTS.md) for the engine's operating manual
if you're driving it programmatically.

## Dev setup

Pick whichever fits your workflow; all three land in the same place.

```bash
# reproducible toolchain (nix flake, includes rust-toolchain.toml's pinned version + tools)
nix develop

# if you use devenv for shell management
devenv shell

# or just plain cargo, with the pinned toolchain from rust-toolchain.toml
cargo build
```

[`just`](https://github.com/casey/just) is the task runner — `just --list`
shows every recipe. The single gate to run before opening a PR:

```bash
just ci   # fmt-check, clippy -D warnings, nextest, doctests, coverage, audit, deny
```

CI runs the same recipe, so a clean `just ci` locally means CI should pass.

## Code standards

- **Formatting**: `cargo fmt --all` (`just fmt`); `just fmt-check` in CI.
- **Lints**: `cargo clippy --workspace --all-targets -- -D warnings`
  (`just lint`). Clippy pedantic is on workspace-wide with a short, commented
  allowlist in `Cargo.toml` for lints that fight this domain's own notation
  (short math names like `p`/`q`/`v`, exact float sentinel comparisons).
- **Tests**: `cargo nextest run --workspace` (`just test`), including
  doctests (`just doctest`, run separately — nextest doesn't execute them)
  and `proptest`-based property tests (co-located with the code they cover,
  run as part of `test`; `just proptest` filters to just those).
- **Coverage**: ≥97% line coverage on the `litgraph` crate (the pure-logic
  engine; the CLI crate is thin and not gated the same way):
  `just cov`; its `cov: litgraph TOTAL lines ...` line is the gated number
  (AGENTS.md, "Reading the coverage result").
- **Formal verification**: Kani harnesses (`#[cfg(kani)]` / `#[kani::proof]`
  in `crates/litgraph`) for properties worth proving, not just testing —
  numerical invariants, no-panic guarantees on untrusted input. Run with
  `just verify-kani`. Zero harnesses today is a valid state (the recipe
  passes trivially); add one whenever you find a property proptest can only
  sample.
- **Supply chain**: `cargo audit` and `cargo deny check` (`just audit`,
  `just deny`) must pass — see `deny.toml` for the license/advisory policy.
- **File size**: keep source files under ~500 lines; split by
  responsibility (see `crates/litgraph/src/model/` for the pattern — a
  `mod.rs` plus `schema.rs`/`links.rs`/`graph.rs`/`resolve.rs`) rather than
  growing one file indefinitely.
- **Commits**: [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `refactor:`, `docs:`, `chore:`, ...). The type decides
  the next release (see [Releases](#releases)), so pick it deliberately.
- **SPDX headers**: every `.rs` file starts with
  `// SPDX-License-Identifier: GPL-3.0-or-later`. If you add a new source
  file without one, add the header — `just lint`/CI will eventually enforce
  this, but don't wait for the check to catch it.
- **License**: GPL-3.0-or-later, inbound = outbound — by contributing you
  agree your changes are licensed under the same terms. Sign off every
  commit (Developer Certificate of Origin):

  ```bash
  git commit -s -m "feat: ..."
  ```

  `-s` appends a `Signed-off-by: Your Name <you@example.com>` trailer
  attesting you have the right to submit the change under the project's
  license.

## Releases

Releases are automatic; nobody tags by hand. After every green CI run on
`main`, `.github/workflows/auto-release.yml` reads the Conventional Commits
since the last `v*` tag (`scripts/release/next-version.sh`):

| Commits since the last tag | Next version |
| --- | --- |
| a breaking change (`type!:` or a `BREAKING CHANGE:` footer) | major (minor while the version is `0.x`) |
| a `feat:` | minor |
| a `fix:` or `perf:` | patch |
| only `docs:`/`ci:`/`chore:`/`refactor:`/`test:`/`build:`/`style:` | no release |

When a release is due it stamps `Cargo.toml` and `CHANGELOG.md`
(`## [Unreleased]` becomes the new version), commits `chore(release): vX.Y.Z`,
tags it, and runs the Release workflow: binaries and WebAssembly bundles on
GitHub Releases, the `ghcr.io/davidawad/litgraph` image, and the
`davidawad/tap/litgraph` Homebrew formula. Keep `## [Unreleased]` in
`CHANGELOG.md` current as you go; it becomes the release notes.

The tap update needs the `HOMEBREW_TAP_DEPLOY_KEY` repository secret, the
private half of a write deploy key on `davidawad/homebrew-tap`.

## Procedure and matter authoring

Author procedure in the typed litigate language under `lit/<forum>/` and
matter-specific facts and stakes in `.case` files under `cases/`. Read the
[litigate tutorial](docs/litigate/TUTORIAL.md) for a working walkthrough and
the [language spec](docs/litigate/SPEC.md) for the normative reference. The
compiled files under `packs/` and `scenarios/` are outputs for litgraph; do
not edit them by hand.

### Change a forum's procedure

Edit the relevant `.lit` module in `lit/<forum>/`. A package's `Lit.toml`
declares its forum, dependencies, and sources. Cite the authority in the rule
and make sure it resolves to the vendored text under `sources/` (or is a
verifiable public citation). Do not invent rule numbers or deadlines; record
uncertainty as `UNVERIFIED` in a note. Add a probability only when it has a
stated basis and vintage; otherwise leave it as a hole for the engine to
report rather than presenting a guess as data.

Check the package and run its inline tests before regenerating outputs:

```bash
litigate check lit/frcp
litigate test lit/frcp
just lit-sync
```

Replace `lit/frcp` with the package you changed. `litigate check lit cases`
checks the full source tree. `just lit-sync` regenerates the `Lit.lock`
files and builds the registered source packages and matters into
`target/lit-build/` (`packs/*.json`, `packs/links.json`, generated clock
fragments, `scenarios/*.json`). Those are build outputs and are never
committed: `crates/litgraph-packs` builds the same files at `cargo build`
time and embeds them, so parallel changes to different packages never
conflict on them. Commit the `.lit` or `.case` sources and their `Lit.lock`
changes. `just lit-sync-check` (part of `just ci`) fails on a stale
`Lit.lock` or a generated file that has been committed.

When adding a new forum package or named matter, add its path to
`crates/litgraph-packs/build.args` so the build includes it. `lit/core` and
`lit/examples` are library and teaching packages and do not emit packs. The
shared `packs/clock/*.json` fragments and `packs/*-SOURCES-NOTES.md` files
are hand-authored; the build refuses a generated clock fragment that also
exists there.

### Change a matter

Edit or add a `.case` file in `cases/` for parties, event history, posture,
known facts, stakes, and analysis settings. Procedure shared by matters
belongs in `lit/`, not in the case or a generated pack. Use `litigate check
lit cases` to typecheck rules and replay case histories, then run the case
with `litigate run cases/<matter>.case --op solve` (or another supported
operation) to inspect the result.

### Review the generated change

For a source change, `just lit-sync` before the change, then `just lit-diff`
after it shows what the change does to the generated files, by JSON
pointer:

```bash
just lit-sync
just lit-diff
just lit-sync-check
just ci
```

`litigate diff lit/<forum> --against packs/<pack>.json` can also show the
semantic difference for one package. Keep unrelated changes to ids,
deadlines, probabilities, and outcomes out of the same change; cite and
explain each substantive rule change in its `.lit` source.

## Getting started

Check the [GitHub Issues](https://github.com/davidawad/litgraph/issues) list
for open work; `AGENTS.md` for the CLI commands and response contract;
`docs/ARCHITECTURE.md` for how the pieces fit together before you change one
of them.
