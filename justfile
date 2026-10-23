# litgraph task runner. `just --list` to see all recipes; `just ci` is the
# single entry point CI (and you, locally) should run before landing changes.

set shell := ["bash", "-euo", "pipefail", "-c"]

# Line coverage floor for the pure-logic `litgraph` crate.
cov_floor := "97"

default:
    just --list

# --- setup -----------------------------------------------------------------

# Install the cargo subcommand tools this justfile's recipes call out to.
# Idempotent -- skips anything already on PATH. Prefer `nix develop` or
# `devenv shell` over this where available (pre-built, no compile time).
install:
    command -v cargo-nextest >/dev/null 2>&1 || cargo install cargo-nextest --locked
    command -v cargo-llvm-cov >/dev/null 2>&1 || cargo install cargo-llvm-cov --locked
    command -v cargo-audit >/dev/null 2>&1 || cargo install cargo-audit --locked
    command -v cargo-deny >/dev/null 2>&1 || cargo install cargo-deny --locked
    command -v cargo-machete >/dev/null 2>&1 || cargo install cargo-machete --locked
    just lit-merge-driver

# Register the `litlock` git merge driver `.gitattributes` names for every
# `Lit.lock` (SPEC §4.2) in this clone. Idempotent; a no-op outside git.
lit-merge-driver:
    #!/usr/bin/env bash
    set -euo pipefail
    git rev-parse --git-dir > /dev/null 2>&1 || exit 0
    git config merge.litlock.name "Lit.lock package-by-package merge (litigate lock --merge)"
    git config merge.litlock.driver 'cargo run -q --manifest-path "$(git rev-parse --show-toplevel)/Cargo.toml" -p litigate-cli -- lock --merge %O %A %B > /dev/null'

# Run the CLI (release build) with any extra args, e.g. `just run describe`.
run *ARGS:
    cargo run --release -p litgraph-cli -- {{ARGS}}

# --- formatting & linting -----------------------------------------------

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

# --- tests ----------------------------------------------------------------

test:
    cargo nextest run --workspace

doctest:
    cargo test --doc --workspace

# proptest-based property tests live under tests/ and src/ like any other
# test and run as part of `test`; nothing extra to invoke here today. This
# recipe exists as the documented hook once dedicated proptest suites land.
proptest:
    cargo nextest run --workspace -E 'test(prop_)'

cov:
    cargo llvm-cov nextest -p litgraph --fail-under-lines {{cov_floor}}

cov-html:
    cargo llvm-cov nextest -p litgraph --html --open

# Every litigate-* crate, each measured by its own tests, must stay at 100%
# lines, regions and branches -- no exclusions. Branch coverage needs a
# nightly toolchain (`rustup toolchain install nightly -c llvm-tools-preview`).
litigate_crates := "litigate-base litigate-syntax litigate-sema litigate-lower litigate-verify litigate-import litigate-cli litigate-lsp litigate-smt"

cov-litigate *CRATES=litigate_crates:
    #!/usr/bin/env bash
    set -euo pipefail
    failed=()
    for crate in {{CRATES}}; do
      json="$(cargo +nightly llvm-cov nextest --branch -p "$crate" --json --summary-only)"
      line="$(jq -r '.data[0].totals | [.lines, .regions, .branches]
        | map("\(.covered)/\(.count)") | join(" ")' <<<"$json")"
      read -r lines regions branches <<<"$line"
      echo "cov-litigate: $crate lines $lines regions $regions branches $branches"
      if ! jq -e '.data[0].totals | [.lines, .regions, .branches]
          | all(.covered == .count)' <<<"$json" >/dev/null; then
        failed+=("$crate")
      fi
    done
    if [ "${#failed[@]}" -ne 0 ]; then
      echo "cov-litigate: below 100% lines/regions/branches: ${failed[*]}" >&2
      echo "cov-litigate: see \`cargo +nightly llvm-cov nextest --branch -p <crate>\`" >&2
      exit 1
    fi
    echo "cov-litigate: every litigate crate at 100% lines, regions and branches"

# Ten-minute local parser and case-input fuzz smoke (five minutes each).
fuzz:
    mkdir -p crates/litigate-syntax/fuzz/corpus/parse crates/litigate-syntax/fuzz/corpus/case_replay
    cp -n crates/litigate-syntax/fuzz/seeds/* crates/litigate-syntax/fuzz/corpus/parse/
    cp -n crates/litigate-syntax/fuzz/seeds/* crates/litigate-syntax/fuzz/corpus/case_replay/
    cargo +nightly fuzz run parse --fuzz-dir crates/litigate-syntax/fuzz -- -max_total_time=300
    cargo +nightly fuzz run case_replay --fuzz-dir crates/litigate-syntax/fuzz -- -max_total_time=300

# --- supply chain & static checks -----------------------------------------

audit:
    cargo audit

deny:
    cargo deny check

# Manual lane: unused-dependency check, not part of `ci`.
machete:
    cargo machete

# --- formal verification ---------------------------------------------------

# Kani harnesses live behind #[cfg(kani)] in crates/litgraph/src/proofs.rs.
# The workspace `rust-version` (1.93) matches kani-verifier 0.67's bundled
# nightly (rustc 1.93.0-nightly), so `cargo kani` builds. Keep harnesses
# bounded: CBMC cannot tractably unwind std collections/string formatting.
verify-kani:
    cargo kani -p litgraph

# Focused mutation lane for temporal, precedence, and case semantics.
mutants:
    cargo mutants -p litigate-sema -f 'crates/litigate-sema/src/temporal/interval.rs' -f 'crates/litigate-sema/src/temporal/transition.rs' -f 'crates/litigate-sema/src/precedence/conflict.rs' -f 'crates/litigate-sema/src/case/event.rs' -f 'crates/litigate-sema/src/case/replay.rs' -f 'crates/litigate-sema/src/case/validate.rs' --re '.*(Window::contains|Window::overlaps|Window::within|governing|Cx.*order|Cx.*deadline|claim|declared|canon|winner|active_claimants).*' -j 4 --timeout 120

# --- benchmarking (optional, not part of `ci`) -----------------------------

bench-smoke:
    cargo bench --workspace -- --test

# --- container -------------------------------------------------------------

docker-build:
    docker build -t litgraph:dev .

docker-smoke: docker-build
    docker run --rm litgraph:dev describe | head -20
    echo '{"packs":["cofc","cafc"],"op":{"op":"solve"}}' | docker run --rm -i litgraph:dev q -

# --- wasm --------------------------------------------------------------

# Build litgraph-wasm for the browser and Node targets into dist/wasm/.
# Packs are embedded in the wasm binary (same build.rs as the CLI), so no
# LITGRAPH_PACKS/packs-dir setup is needed at runtime.
wasm:
    cargo build --release -p litgraph-wasm --target wasm32-unknown-unknown
    mkdir -p dist/wasm/web dist/wasm/node
    wasm-bindgen --target web --out-dir dist/wasm/web target/wasm32-unknown-unknown/release/litgraph_wasm.wasm
    wasm-bindgen --target nodejs --out-dir dist/wasm/node target/wasm32-unknown-unknown/release/litgraph_wasm.wasm

# Build the Node target and run one solve through it.
wasm-smoke: wasm
    node -e "const m=require('./dist/wasm/node/litgraph_wasm.js'); const r=JSON.parse(m.handle(JSON.stringify({packs:['cofc'],op:{op:'solve'}}))); if(!r.ok){console.error(r);process.exit(1)} console.log('wasm smoke ok')"

# --- typescript (packages/litigate-ts) ---------------------------------------
# @litgraph/litigate-ts: types generated from the Rust schemas plus a typed
# client over litgraph-wasm. Needs node >= 20 and npm (and, for ts-build /
# ts-test, the wasm toolchain `just wasm` needs).

ts_pkg := "packages/litigate-ts"
# Generated, committed files; `ts-gen-check` fails when they are stale.
ts_generated := ts_pkg + "/schema/litgraph.schema.json " + ts_pkg + "/src/generated/schemas.ts"

# Install the package's pinned dev dependencies (package-lock.json).
ts-install:
    cd {{ts_pkg}} && npm ci --no-audit --no-fund

# Regenerate the JSON Schema bundle from the Rust types, then the
# TypeScript types from it.
ts-gen: ts-install
    cargo run -q -p litgraph-schema-bundle > {{ts_pkg}}/schema/litgraph.schema.json
    cd {{ts_pkg}} && npm run --silent gen

# CI gate: regenerate, then fail if the committed generated files differ.
ts-gen-check: ts-gen
    #!/usr/bin/env bash
    set -euo pipefail
    stale="$(git status --porcelain --untracked-files=all -- {{ts_generated}})"
    if [ -n "$stale" ]; then
      echo "$stale" >&2
      git --no-pager diff --stat -- {{ts_generated}} >&2
      echo "ts-gen-check: generated TypeScript types are stale; run \`just ts-gen\` and commit the result" >&2
      exit 1
    fi
    echo "ts-gen-check: generated types are up to date"

# Build litgraph-wasm into the package (wasm/node, wasm/web), then compile
# the TypeScript to dist/. The package is ESM and wasm-bindgen's nodejs
# target is CommonJS, so wasm/node gets its own package.json.
ts-build: wasm ts-install
    rm -rf {{ts_pkg}}/wasm
    mkdir -p {{ts_pkg}}/wasm
    cp -r dist/wasm/node dist/wasm/web {{ts_pkg}}/wasm/
    echo '{"type": "commonjs"}' > {{ts_pkg}}/wasm/node/package.json
    cd {{ts_pkg}} && npm run --silent build

# Unit, type-level, and golden tests (every scenarios/*.json through wasm
# against the native `litgraph` CLI, and `litigate check`/`build` over lit/
# and cases/ in memory against the native `litigate` CLI, both built here).
ts-test: ts-build
    cargo build -q -p litgraph-cli -p litigate-cli
    cd {{ts_pkg}} && LITGRAPH_BIN="$(cd ../.. && pwd)/target/debug/litgraph" LITIGATE_BIN="$(cd ../.. && pwd)/target/debug/litigate" npm test

# --- nix ---------------------------------------------------------------

nix-build:
    nix build .#default
    ./result/bin/litgraph describe | head -5

nix-check:
    nix flake check

# --- aggregate --------------------------------------------------------------

# The single CI entry point: fast, deterministic gates only. Coverage,
# audit, and deny hit the network/build cache so they're last.
ci: fmt-check lint lit-sync-check ts-gen-check test ts-test doctest cov cov-litigate audit deny
    @echo "ci: all gates passed"

clean:
    cargo clean
    rm -r -f dist result

# --- litigate authoring language -------------------------------------------
# Strict check of the stdlib and matters (SPEC §9.3); the epic gate once
# lit/core lands. `just lit-check --profile dev lit/examples` for one package.
lit-check *ARGS='--profile strict lit cases':
    cargo run -q -p litigate-cli -- --format human check {{ARGS}}

# The generated repository files (DESIGN §11.4, SPEC §9.4): every
# packs/*.json (and packs/links.json) is built from these lit/ packages,
# every scenarios/*.json from these cases/ matters. lit/core and
# lit/examples are library/teaching packages and emit no pack; the
# hand-authored clock library (packs/clock/*.json other than a generated
# pack's own fragment) and packs/*-SOURCES-NOTES.md are not generated.
# `--pack-file ID=STEM` keeps a legacy file name where it is not the id.
lit_sync_inputs := "lit/cafc lit/cofc lit/frap lit/frcp lit/frcrimp lit/itc lit/mpep lit/ptab lit/ttab lit/links cases/cofc-1498-patent-case.case cases/cofc-bid-protest.case cases/frcp-civil-case.case cases/ptab-ipr-defense.case cases/ttab-opposition-applicant-defense.case"
lit_sync_names := "--pack-file cafc=cafc-federal-circuit --pack-file cofc=cofc-court-of-federal-claims"

# Regenerate every committed Lit.lock, then packs/*.json, packs/links.json,
# generated packs/clock/ fragments and scenarios/*.json from lit/ and
# cases/. Never hand-edit those files; edit the source and run this. Run
# it after a merge or rebase touching lit/: the Lit.lock merge driver
# leaves a package both sides changed unmerged (L1007) until it does.
lit-sync: lit-merge-driver
    #!/usr/bin/env bash
    set -euo pipefail
    out="$(mktemp -d)"
    trap 'rm -rf "$out"' EXIT
    lit=(cargo run -q -p litigate-cli -- --format human)
    for lockfile in lit/*/Lit.lock; do
      "${lit[@]}" lock "${lockfile%/Lit.lock}" > /dev/null
    done
    "${lit[@]}" lock cases > /dev/null
    "${lit[@]}" build {{lit_sync_inputs}} {{lit_sync_names}} --out "$out" > /dev/null \
      || "${lit[@]}" build {{lit_sync_inputs}} {{lit_sync_names}} --out "$out"
    rm -f packs/*.json scenarios/*.json
    for f in "$out"/packs/*.json "$out"/packs/clock/*.json "$out"/scenarios/*.json; do
      case "$f" in *.map.json) continue ;; esac
      cp "$f" "${f#"$out"/}"
    done
    echo "lit-sync: regenerated Lit.lock files, packs/ and scenarios/ from lit/ and cases/"

# CI gate: regenerate, then fail if any generated file (a Lit.lock
# included) differs from (or is missing from) what is committed.
lit-sync-check: lit-sync
    #!/usr/bin/env bash
    set -euo pipefail
    stale="$(git status --porcelain --untracked-files=all -- packs scenarios '*Lit.lock')"
    if [ -n "$stale" ]; then
      echo "$stale" >&2
      git --no-pager diff --stat -- packs scenarios '*Lit.lock' >&2
      echo "lit-sync-check: packs/, scenarios/ or a Lit.lock is stale; run \`just lit-sync\` and commit the result" >&2
      exit 1
    fi
    echo "lit-sync-check: generated files are up to date"

# Semantic diff (SPEC §8.6) of a fresh build against the committed files,
# without writing: what `lit-sync` would change, by JSON pointer.
lit-diff:
    #!/usr/bin/env bash
    set -euo pipefail
    out="$(mktemp -d)"
    trap 'rm -rf "$out"' EXIT
    lit=(cargo run -q -p litigate-cli -- --format human)
    "${lit[@]}" build {{lit_sync_inputs}} {{lit_sync_names}} --out "$out" > /dev/null \
      || "${lit[@]}" build {{lit_sync_inputs}} {{lit_sync_names}} --out "$out"
    "${lit[@]}" diff "$out" packs --check
    "${lit[@]}" diff "$out" scenarios --check

lit-test:
    cargo nextest run -p litigate-base -p litigate-syntax -p litigate-sema -p litigate-verify -p litigate-lower -p litigate-import -p litigate-cli -p litigate-lsp -p tree-sitter-litigate
    cargo test --doc -p litigate-base -p litigate-syntax -p litigate-sema -p litigate-lower -p litigate-import -p litigate-cli

# tree-sitter grammar (editors/tree-sitter-litigate): regenerating the parser
# from grammar.js must reproduce the committed src/ byte for byte, then the
# corpus (test/corpus) and highlight (test/highlight) tests must pass. Uses a
# `tree-sitter` on PATH when it is the pinned version, else npx (needs node).
# (Named tree-sitter-test: the ts-* recipes belong to packages/litigate-ts.)
ts_cli_version := "0.27.1"
tree-sitter-test:
    #!/usr/bin/env bash
    set -euo pipefail
    cd editors/tree-sitter-litigate
    if tree-sitter --version 2>/dev/null | grep -q " {{ts_cli_version}}"; then ts=(tree-sitter); \
    else ts=(npx --yes "tree-sitter-cli@{{ts_cli_version}}"); fi
    "${ts[@]}" generate
    git diff --exit-code --stat -- src || { echo "tree-sitter-test: src/ is stale; commit the regenerated parser" >&2; exit 1; }
    "${ts[@]}" test

# Editor modes (editors/{emacs,helix,nvim}): Emacs ERT batch suite, Helix
# queries + `hx --health`, headless Neovim (editors/litigate-editors/tests).
# Part of the workspace tests too; each editor's test skips (and logs to
# $TEST_SKIP_LOG) when that editor is not installed.
editors-test:
    cargo nextest run -p litigate-editors
