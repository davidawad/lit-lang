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

# Pure-logic coverage lane for the authoring compiler. All code in syntax,
# sema and lower is measured; engine, CLI and LSP crates are outside this
# compiler-only floor.
cov-litigate:
    cargo llvm-cov nextest -p litigate-syntax -p litigate-sema -p litigate-lower --fail-under-lines {{cov_floor}}

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

# KNOWN GAP (2026-09-28, verified locally with wasm-bindgen-cli 0.2.127):
# this currently panics -- crates/litgraph/src/api/mod.rs:143 calls
# std::time::Instant::now() (elapsed_ms timing), which has no clock source
# on wasm32-unknown-unknown and traps with "RuntimeError: unreachable".
# Out of scope for this packaging workstream (crates/** owned elsewhere);
# needs a #[cfg(target_arch = "wasm32")] time source (e.g. js_sys::Date).
wasm-smoke: wasm
    node -e "const m=require('./dist/wasm/node/litgraph_wasm.js'); const r=JSON.parse(m.handle(JSON.stringify({packs:['cofc'],op:{op:'solve'}}))); if(!r.ok){console.error(r);process.exit(1)} console.log('wasm smoke ok')"

# --- nix ---------------------------------------------------------------

nix-build:
    nix build .#default
    ./result/bin/litgraph describe | head -5

nix-check:
    nix flake check

# --- aggregate --------------------------------------------------------------

# The single CI entry point: fast, deterministic gates only. Coverage,
# audit, and deny hit the network/build cache so they're last.
ci: fmt-check lint lit-sync-check test doctest cov audit deny
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

# Regenerate packs/*.json, packs/links.json, generated packs/clock/
# fragments and scenarios/*.json from lit/ and cases/. Never hand-edit
# those files; edit the source and run this.
lit-sync:
    #!/usr/bin/env bash
    set -euo pipefail
    out="$(mktemp -d)"
    trap 'rm -rf "$out"' EXIT
    lit=(cargo run -q -p litigate-cli -- --format human)
    "${lit[@]}" build {{lit_sync_inputs}} {{lit_sync_names}} --out "$out" > /dev/null \
      || "${lit[@]}" build {{lit_sync_inputs}} {{lit_sync_names}} --out "$out"
    rm -f packs/*.json scenarios/*.json
    for f in "$out"/packs/*.json "$out"/packs/clock/*.json "$out"/scenarios/*.json; do
      case "$f" in *.map.json) continue ;; esac
      cp "$f" "${f#"$out"/}"
    done
    echo "lit-sync: regenerated packs/ and scenarios/ from lit/ and cases/"

# CI gate: regenerate, then fail if any generated file differs from (or is
# missing from) what is committed.
lit-sync-check: lit-sync
    #!/usr/bin/env bash
    set -euo pipefail
    stale="$(git status --porcelain --untracked-files=all -- packs scenarios)"
    if [ -n "$stale" ]; then
      echo "$stale" >&2
      git --no-pager diff --stat -- packs scenarios >&2
      echo "lit-sync-check: packs/ or scenarios/ is stale; run \`just lit-sync\` and commit the result" >&2
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
ts_cli_version := "0.27.1"
ts-test:
    #!/usr/bin/env bash
    set -euo pipefail
    cd editors/tree-sitter-litigate
    if tree-sitter --version 2>/dev/null | grep -q " {{ts_cli_version}}"; then ts=(tree-sitter); \
    else ts=(npx --yes "tree-sitter-cli@{{ts_cli_version}}"); fi
    "${ts[@]}" generate
    git diff --exit-code --stat -- src || { echo "ts-test: src/ is stale; commit the regenerated parser" >&2; exit 1; }
    "${ts[@]}" test

# Editor modes (editors/{emacs,helix,nvim}): Emacs ERT batch suite, Helix
# queries + `hx --health`, headless Neovim (editors/litigate-editors/tests).
# Part of the workspace tests too; each editor's test skips (and logs to
# $TEST_SKIP_LOG) when that editor is not installed.
editors-test:
    cargo nextest run -p litigate-editors
