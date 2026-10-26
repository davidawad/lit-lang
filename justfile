# litgraph task runner. `just --list` to see all recipes; `just ci` is the
# single entry point CI (and you, locally) should run before landing changes.

set shell := ["bash", "-euo", "pipefail", "-c"]

# Line coverage floor for the pure-logic `litigate-engine` crate.
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
    just merge-drivers

# Register every git merge driver `.gitattributes` names in this clone.
merge-drivers: tree-sitter-merge-driver

# Register the `tsgenerate` git merge driver `.gitattributes` names for the
# generated tree-sitter parser (editors/tree-sitter-litigate/src). When both
# sides changed it, the merge keeps ours (`true` leaves %A as is), so two
# branches changing different grammar rules merge and rebase cleanly; then
# `just tree-sitter-generate` regenerates it from the merged grammar. Needs
# neither node nor cargo. Idempotent; a no-op outside git.
tree-sitter-merge-driver:
    #!/usr/bin/env bash
    set -euo pipefail
    git rev-parse --git-dir > /dev/null 2>&1 || exit 0
    git config merge.tsgenerate.name "generated tree-sitter parser: keep ours, then just tree-sitter-generate"
    git config merge.tsgenerate.driver true

# Run `litigate` (release build) with any extra args, e.g. `just run solve cases/frcp-civil-case.case`.
run *ARGS:
    cargo run --release -p litigate-cli -- {{ARGS}}

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

# Coverage runs serialize on a lock in the target dir and start from
# `cargo llvm-cov clean --workspace`: report collects every workspace
# member's executable under target/llvm-cov-target, but a `-p` run cleans
# only that package, so binaries left by an earlier branch or another
# crate's run (stale coverage data) otherwise skew the number (lg-flu.42).
cov_lock := 'target="$(cargo metadata --format-version 1 --no-deps | jq -r .target_directory)"; mkdir -p "$target"; exec 9> "$target/llvm-cov.lock"; if command -v flock > /dev/null; then flock 9; fi'

# The per-file table, then the verdict: scripts/cov-gate's, on the one
# JSON summary it prints from ("cov: litigate-engine TOTAL lines P% (covered/count)
# >= floor F%: pass"); cargo-llvm-cov's --fail-under-lines on the same data
# must agree. Then the self-checks: an unreachable floor (100.01) and a
# known-low input must fail.
cov:
    #!/usr/bin/env bash
    set -euo pipefail
    scripts/cov-gate self-test
    {{cov_lock}}
    cargo llvm-cov clean --workspace
    cargo llvm-cov nextest -p litigate-engine --no-report
    cargo llvm-cov report -p litigate-engine --summary-only
    json="$(cargo llvm-cov report -p litigate-engine --json --summary-only)"
    scripts/cov-gate lines {{cov_floor}} litigate-engine <<<"$json"
    cargo llvm-cov report -p litigate-engine --summary-only --fail-under-lines {{cov_floor}} > /dev/null
    if cargo llvm-cov report -p litigate-engine --summary-only --fail-under-lines 100.01 > /dev/null 2>&1; then
      echo "cov self-check: floor not enforced (cargo llvm-cov report --fail-under-lines 100.01 passed)" >&2
      exit 1
    fi
    scripts/cov-gate reject-low lines {{cov_floor}} litigate-engine <<<"$json"

cov-html:
    cargo llvm-cov nextest -p litigate-engine --html --open

# Every litigate-* crate, each measured by its own tests, must stay at 100%
# lines, regions and branches -- no exclusions. (litigate-engine, the
# renamed litgraph engine, keeps the `cov` floor above.) Branch coverage needs a
# nightly toolchain (`rustup toolchain install nightly -c llvm-tools-preview`).
litigate_crates := "litigate-base litigate-syntax litigate-sema litigate-lower litigate-verify litigate-import litigate-cli litigate-lsp litigate-smt"

cov-litigate *CRATES=litigate_crates:
    #!/usr/bin/env bash
    set -euo pipefail
    scripts/cov-gate self-test
    {{cov_lock}}
    cargo +nightly llvm-cov clean --workspace
    failed=()
    for crate in {{CRATES}}; do
      json="$(cargo +nightly llvm-cov nextest --branch -p "$crate" --json --summary-only)"
      scripts/cov-gate full "$crate" <<<"$json" || failed+=("$crate")
      scripts/cov-gate reject-low full "$crate" <<<"$json"
    done
    if [ "${#failed[@]}" -ne 0 ]; then
      echo "cov-litigate: below 100% lines/regions/branches: ${failed[*]}" >&2
      echo "cov-litigate: see \`cargo +nightly llvm-cov nextest --branch -p <crate>\`" >&2
      exit 1
    fi
    echo "cov-litigate: every litigate crate at 100% lines, regions and branches"

# The coverage gate's own verdicts on fixed known-low and known-ok inputs
# (no build; `cov` and `cov-litigate` run it first too).
cov-gate-test:
    scripts/cov-gate self-test

# GitHub-mirrors-GitLab sync verdicts against throwaway bare repositories,
# including the divergence refusal (maintenance/mirror-sync/README.md).
mirror-sync-test:
    scripts/mirror-sync self-test

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

# Not part of `ci` (each lane needs its own toolchain); `verify` runs both
# provers. Design review §3.7 lists the metatheory still to prove in Lean.
verify: verify-kani verify-verus

# Kani harnesses live behind #[cfg(kani)]: the engine's in
# crates/litigate-engine/src/proofs.rs and clock/, litigate-sema's beside the
# date arithmetic (types/date/proofs.rs) and window algebra
# (temporal/interval/proofs.rs). The workspace `rust-version` (1.93) matches
# kani-verifier 0.67's bundled nightly (rustc 1.93.0-nightly), so `cargo
# kani` builds. Keep harnesses bounded: CBMC cannot tractably unwind std
# collections/string formatting. `just verify-kani litigate-sema` for one crate.
kani_crates := "litigate-engine litigate-sema"

verify-kani *CRATES=kani_crates:
    for crate in {{CRATES}}; do cargo kani -p "$crate"; done

# Verus proofs of the counting laws (design review §3.7 item 6) over a model
# of the day-count semantics: roll lands on the first open day, counting is
# monotone, FRCP 6(d) service days follow the roll, the backward roll is the
# mirrored forward roll, and the Gregorian 400-year cycle is whole weeks.
verify-verus:
    verus crates/litigate-verify/verus/counting.rs

# Focused mutation lane for date arithmetic, temporal, precedence, and case
# semantics (cargo-mutants; not part of `ci`).
mutants:
    cargo mutants -p litigate-sema -f 'crates/litigate-sema/src/types/date.rs' -f 'crates/litigate-sema/src/temporal/interval.rs' -f 'crates/litigate-sema/src/temporal/transition.rs' -f 'crates/litigate-sema/src/precedence/conflict.rs' -f 'crates/litigate-sema/src/case/event.rs' -f 'crates/litigate-sema/src/case/replay.rs' -f 'crates/litigate-sema/src/case/validate.rs' --re '.*(days_in_month|days_from_civil|civil|Window::contains|Window::overlaps|Window::within|governing|Cx.*order|Cx.*deadline|claim|declared|canon|winner|active_claimants).*' -j 4 --timeout 120

# Full mutation sweep over litigate crates, one at a time (hours; manual).
# `just mutants-all litigate-lower` for one crate.
mutants-all *CRATES=litigate_crates:
    for crate in {{CRATES}}; do cargo mutants -p "$crate" -j 4 --timeout 120; done

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

# Install the package's pinned dev dependencies (package-lock.json).
ts-install:
    cd {{ts_pkg}} && npm ci --no-audit --no-fund

# Generate the JSON Schema bundle from the Rust types, then the
# TypeScript types from it. Both are build outputs (gitignored), never
# committed, so parallel schema changes cannot conflict on them; ts-build
# and ts-test run this first.
ts-gen: ts-install
    mkdir -p {{ts_pkg}}/schema {{ts_pkg}}/src/generated
    cargo run -q -p litgraph-schema-bundle > {{ts_pkg}}/schema/litgraph.schema.json
    cd {{ts_pkg}} && npm run --silent gen

# Build litgraph-wasm into the package (wasm/node, wasm/web), then compile
# the TypeScript to dist/. The package is ESM and wasm-bindgen's nodejs
# target is CommonJS, so wasm/node gets its own package.json.
ts-build: wasm ts-gen
    rm -rf {{ts_pkg}}/wasm
    mkdir -p {{ts_pkg}}/wasm
    cp -r dist/wasm/node dist/wasm/web {{ts_pkg}}/wasm/
    echo '{"type": "commonjs"}' > {{ts_pkg}}/wasm/node/package.json
    cd {{ts_pkg}} && npm run --silent build

# Unit, type-level, and golden tests (every built scenarios/*.json, from
# `lit-sync`'s target/lit-build/, through wasm against the native `litgraph`
# CLI, and `litigate check`/`build` over lit/ and cases/ in memory against
# the native `litigate` CLI, both built here).
ts-test: ts-build lit-sync
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
ci: fmt-check lint cov-gate-test mirror-sync-test lit-sync-check tree-sitter-test test ts-test doctest cov cov-litigate audit deny
    @echo "ci: all gates passed"

# The per-bead gate (scripts/ci-scoped): global checks (fmt, typos,
# lit-sync-check, tree-sitter-test when the grammar changed, cargo check
# --workspace, audit, deny), then clippy, one instrumented nextest run and
# doctests for the packages changed since BASE (default: the merge-base
# with origin/main) plus their workspace reverse dependencies. Coverage
# from that run: a litigate crate below 100% prints COV_WAIVED, which
# `ci` (cov-litigate) still enforces. Ends `CI_SCOPED: pass|fail scope=...`.
ci-scoped BASE='':
    LITIGATE_CRATES="{{litigate_crates}}" scripts/ci-scoped run {{BASE}}

# The scope computation's self-test (no build; `ci-scoped` runs it first).
ci-scoped-test:
    scripts/ci-scoped self-test

clean:
    cargo clean
    rm -r -f dist result

# --- litigate authoring language -------------------------------------------
# Strict check of the stdlib and matters (SPEC §9.3); the epic gate once
# lit/core lands. `just lit-check --profile dev lit/examples` for one package.
lit-check *ARGS='--profile strict lit cases':
    cargo run -q -p litigate-cli -- --format human check {{ARGS}}

# The generated files (DESIGN §11.4, SPEC §9.4): every pack, links.json,
# generated clock fragment and named scenario is built from lit/ and cases/
# with the arguments in crates/litgraph-packs/build.args. They are build
# outputs, never committed: crates/litgraph-packs builds and embeds them at
# compile time, so parallel lit/ changes cannot conflict on them. Only the
# Lit.lock files are committed (each row holds only its own package's
# digests, so edits to different packages merge as plain text).
lit_build_args := "crates/litgraph-packs/build.args"
# Where `lit-sync` writes the build for inspection (`--packs-dir`,
# `--scenarios-dir`, `litigate diff`).
lit_build_dir := "target/lit-build"

# Regenerate every committed Lit.lock, then build packs/, packs/links.json,
# generated packs/clock/ fragments and scenarios/ from lit/ and cases/ into
# target/lit-build/ (the hand-authored packs/clock/*.json copied beside
# them, as litgraph-packs embeds them). Run it after a merge or rebase in
# which both sides changed one package: its Lit.lock row conflicts (or is
# stale, L1007) until it does, like a Cargo.lock.
lit-sync: merge-drivers
    #!/usr/bin/env bash
    set -euo pipefail
    out="{{lit_build_dir}}"
    lit=(cargo run -q -p litigate-cli -- --format human)
    for lockfile in lit/*/Lit.lock docs/litigate/examples/change/*/Lit.lock; do
      "${lit[@]}" lock "${lockfile%/Lit.lock}" > /dev/null
    done
    "${lit[@]}" lock cases > /dev/null
    mapfile -t args < <(grep -v -e '^#' -e '^[[:space:]]*$' {{lit_build_args}})
    rm -rf "$out"
    "${lit[@]}" build "${args[@]}" --out "$out" > /dev/null \
      || "${lit[@]}" build "${args[@]}" --out "$out"
    find "$out" -name '*.map.json' -delete
    rm -rf "$out/litigatables"
    cp -n packs/clock/*.json "$out/packs/clock/"
    echo "lit-sync: regenerated Lit.lock files; built packs/ and scenarios/ into $out"

# CI gate: regenerate, then fail if a Lit.lock differs from (or is missing
# from) what is committed, or if a generated file is tracked again.
lit-sync-check: lit-sync
    #!/usr/bin/env bash
    set -euo pipefail
    stale="$(git status --porcelain --untracked-files=all -- '*Lit.lock')"
    if [ -n "$stale" ]; then
      echo "$stale" >&2
      git --no-pager diff --stat -- '*Lit.lock' >&2
      echo "lit-sync-check: a Lit.lock is stale; run \`just lit-sync\` and commit the result" >&2
      exit 1
    fi
    # crates/litgraph-packs refuses a generated clock fragment committed
    # under packs/clock/, so only these two places can hold one.
    tracked="$(git ls-files -- ':(glob)packs/*.json' scenarios)"
    if [ -n "$tracked" ]; then
      echo "$tracked" >&2
      echo "lit-sync-check: generated files are committed; \`git rm --cached\` them (crates/litgraph-packs builds them)" >&2
      exit 1
    fi
    echo "lit-sync-check: Lit.lock files are up to date; no generated file is committed"

# Semantic diff (SPEC §8.6) of a fresh build against the last `lit-sync`
# build in target/lit-build/, without writing: what changed since then.
lit-diff:
    #!/usr/bin/env bash
    set -euo pipefail
    out="$(mktemp -d)"
    trap 'rm -rf "$out"' EXIT
    lit=(cargo run -q -p litigate-cli -- --format human)
    mapfile -t args < <(grep -v -e '^#' -e '^[[:space:]]*$' {{lit_build_args}})
    "${lit[@]}" build "${args[@]}" --out "$out" > /dev/null \
      || "${lit[@]}" build "${args[@]}" --out "$out"
    "${lit[@]}" diff "$out/packs" "{{lit_build_dir}}/packs" --check
    "${lit[@]}" diff "$out/scenarios" "{{lit_build_dir}}/scenarios" --check

lit-test:
    cargo nextest run -p litigate-base -p litigate-syntax -p litigate-sema -p litigate-verify -p litigate-lower -p litigate-import -p litigate-cli -p litigate-lsp -p tree-sitter-litigate
    cargo test --doc -p litigate-base -p litigate-syntax -p litigate-sema -p litigate-lower -p litigate-import -p litigate-cli

# tree-sitter grammar (editors/tree-sitter-litigate): regenerating the parser
# from grammar.js must reproduce the committed src/ byte for byte, then the
# corpus (test/corpus) and highlight (test/highlight) tests must pass, and
# two branches changing different grammar rules must rebase cleanly
# (script/merge-test). Uses a `tree-sitter` on PATH when it is the pinned
# version, else npx (needs node).
# (Named tree-sitter-*: the ts-* recipes belong to packages/litigate-ts.)
ts_cli_version := "0.27.1"
ts_cli := "if tree-sitter --version 2>/dev/null | grep -q ' " + ts_cli_version + "'; then ts=(tree-sitter); else ts=(npx --yes tree-sitter-cli@" + ts_cli_version + "); fi"
tree-sitter-test:
    #!/usr/bin/env bash
    set -euo pipefail
    cd editors/tree-sitter-litigate
    {{ts_cli}}
    before="$(mktemp -d)"
    trap 'rm -rf "$before"' EXIT
    cp -R src "$before/"
    "${ts[@]}" generate
    diff -r -q "$before/src" src || { echo "tree-sitter-test: src/ is stale; run just tree-sitter-generate and commit src/" >&2; exit 1; }
    "${ts[@]}" test
    script/merge-test "${ts[@]}"

# Regenerate the tree-sitter parser (editors/tree-sitter-litigate/src) from
# grammar.js and grammar/*.js. Run it after editing the grammar, and after a
# merge or rebase touching the grammar: the tsgenerate merge driver keeps
# our parser when both sides changed it.
tree-sitter-generate:
    #!/usr/bin/env bash
    set -euo pipefail
    cd editors/tree-sitter-litigate
    {{ts_cli}}
    "${ts[@]}" generate

# Editor modes (editors/{emacs,helix,nvim}): Emacs ERT batch suite, Helix
# queries + `hx --health`, headless Neovim (editors/litigate-editors/tests).
# Part of the workspace tests too; each editor's test skips (and logs to
# $TEST_SKIP_LOG) when that editor is not installed.
editors-test:
    cargo nextest run -p litigate-editors
