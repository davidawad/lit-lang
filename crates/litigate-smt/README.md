# litigate-smt

SMT backend for litigate's refinement types (lg-962.2; consumed by lg-962.3).
It covers a small, decidable fragment: quantifier-free linear arithmetic over
integers and rationals (`QF_LIRA`), with `not`/`and`/`or` on top. Dates are
`Int` day numbers. There are no quantifiers, no non-linear terms and no
uninterpreted functions, so every check terminates with `sat`, `unsat` or,
once its deterministic work budget runs out, `unknown`.

```rust
use litigate_smt::{Backend, Cached, Formula, LinExpr, Outcome, Query, Rational, Z3};

let mut q = Query::new();
let p = q.real("p")?;                      // Probability {0 <= p <= 1}
let served = q.int("served")?;             // dates are day numbers
let due = q.int("due")?;
q.assert("p>=0", &Formula::ge(&p, 0))?;
q.assert("p<=1", &Formula::le(&p, 1))?;
q.assert("p=3/2", &Formula::eq(&p, Rational::new(3, 2)?))?;
q.assert("due", &Formula::eq(&due, LinExpr::from(&served).plus_const(21)))?;

let mut solver = Cached::new(Z3::new()?);
match solver.check(&q)? {
    Outcome::Sat(model) => { /* model["p"], model["due"]: exact Rationals */ }
    Outcome::Unsat(core) => assert_eq!(core, ["p<=1", "p=3/2"]),
    Outcome::Unknown(reason) => { /* budget exhausted: not a proof */ }
}
```

## API

| Item | What it is |
|---|---|
| `Query` | Declared variables (`int`, `real`) plus labelled assertions (`assert(label, &Formula)`). `to_smtlib()` returns the canonical SMT-LIB 2 script. |
| `Formula` | `Formula::{le, lt, eq, ne, ge, gt, cmp}` over `LinExpr`; `!f`, `Formula::And`, `Formula::Or`, `Formula::implies`. |
| `LinExpr` | `Σ cᵢ·xᵢ + c` with `Rational` coefficients; `From<&Var>`, `From<i64>`, `From<Rational>`. |
| `Rational` | Exact `i64/i64`, always reduced. Arithmetic goes through `i128` and reports `Error::Overflow` rather than wrapping or rounding. |
| `Backend` | `fingerprint()` (solver + version + options) and `check(&Query) -> Outcome`. |
| `Outcome` | `Sat(Model)` (every declared variable, exact), `Unsat(core labels)` or `Unknown(reason)`. |
| `Z3` | The backend: pinned Z3, configured for determinism (below). |
| `Cached<B>` | Any `Backend` wrapped in an in-memory cache keyed by `ContentHash`. |
| `ContentHash` | SHA-256 over a format tag, the backend fingerprint and the canonical SMT-LIB text, each length-prefixed. |

Every atom is normalised to `c + Σ aᵢ·xᵢ ⋈ 0` with `⋈ ∈ {<=, <, =}`: `>=`
and `>` swap sides, `!=` becomes `not =`, like terms are combined, zero
coefficients are dropped and variables are sorted by name. So `x >= 3` and
`3 <= x` are the same question and get the same hash. An atom whose
variables are all `Int` and whose numbers are all integral is solved over
the integers. Any other atom is solved over the rationals, with `Int`
variables cast (`to_real`). Assertion order is part of the question.

## Why Z3, not cvc5

Both are first-rate SMT solvers for this fragment. The decision rests on
how they get into a Rust build.

| | Z3 via `z3` 0.21.1 / `z3-sys` 0.13.1 | cvc5 via `cvc5` 0.6.1 / `cvc5-sys` 0.6.1 |
|---|---|---|
| Licence (solver) | MIT | BSD-3-Clause |
| Licence (crates) | MIT, checked by cargo-deny | BSD-3-Clause |
| Licence (static link) | Nothing else: `libz3.a` is self-contained (plus the system C++ runtime) | Static builds also link GMP and MPFR (LGPL-3.0-or-later) and libpoly (LGPL-3.0), which cargo-deny cannot see. That is compatible with our GPL-3.0-or-later but adds obligations |
| Static build | The official release ships `libz3.a` for x86_64/aarch64 Linux and macOS, so we download it and check its SHA-256 (no compiler needed). `vendored` can also build it from source with cmake | `cvc5-sys` excludes the cvc5 source from the crate. You build cvc5 1.4.0 yourself (`configure.sh`, cmake, Python, GMP) or point `CVC5_LIB_DIR` at an install |
| Bindings | Pregenerated; no libclang needed | Always bindgen, so libclang is needed at build time |
| Docker (`rust:*-slim`, no cmake/clang) | Works: the build only downloads | Fails: needs cmake, libclang and the cvc5 source |
| Maintenance | `z3` crate published since 2015 (57 releases, latest 2026-09), prove-rs org; Z3 is Microsoft Research, with steady releases | `cvc5` crate first published 2026-04 (0.4.0; five releases since), cvc5 org; cvc5 itself is mature |
| Rust API | Safe, typed ASTs (`Int`, `Real`, `Bool`), `assert_and_track`, unsat cores, models with exact rationals | Thin safe layer over the C API, younger |
| Determinism | Deterministic for a fixed version, options, seed and assertion order. `rlimit` is a deterministic work budget | Also deterministic with a fixed seed; `resource-limit`/`rlimit` exist too |

Z3 wins on building statically without a toolchain (it works in the
existing slim Docker image and in CI unchanged), on not needing libclang,
and on binding maturity. Determinism is a wash. The licences are both
permissive, and Z3 static links nothing beyond the solver.

## Build: pinned, static Z3

`build.rs` takes the official **Z3 5.1.0** release asset for the target and
pins it twice:

1. **Bytes.** The zip's SHA-256 must equal the digest GitHub publishes for
   it, which is hard-coded in `build.rs`. A mismatch fails the build.
2. **Licence.** The zip's `LICENSE.txt` must still be the MIT License.
   cargo-deny checks the `z3`/`z3-sys` crates but cannot see a native
   archive, so the check lives here.

Only `bin/libz3.a` and `LICENSE.txt` are extracted, into `OUT_DIR`.
`z3-sys` (default features, pregenerated bindings) emits `-lz3`, and the
linker resolves it to that static archive. At run time,
`Z3::new()` compares the linked library's version with `PINNED_Z3_VERSION`
and returns `Error::VersionMismatch` if they differ, for example if a
system `libz3` were found first. Test binaries depend only on the system C++
runtime (`ldd` shows `libstdc++`, no `libz3`).

| Target | Asset |
|---|---|
| x86_64 Linux | `z3-5.1.0-x64-glibc-2.39.zip`: linking needs glibc ≥ 2.38 (`__isoc23_*`), e.g. Debian 13, Ubuntu 24.04 |
| aarch64 Linux | `z3-5.1.0-arm64-glibc-2.38.zip` |
| x86_64 / aarch64 macOS | `z3-5.1.0-{x64,arm64}-osx-13.3.zip` |
| Windows, wasm, others | Not supported: the build fails with a message naming the supported targets |

**Network.** Each fresh build directory (debug, release, `llvm-cov`,
nightly) downloads the zip once (~40-55 MB) from
`github.com/Z3Prover/z3/releases` by direct URL. It does not use the
GitHub API, so it needs no token and hits no API rate limit. The download uses `ureq` with rustls and Mozilla's roots
(`webpki-roots`, CDLA-Permissive-2.0, a build-only dependency with a
per-crate exception in `deny.toml`). Extraction uses `zip` 9: z3-sys's
own `gh-release` feature was rejected because it pins `zip` 8, which
has open advisories (RUSTSEC-2026-0367/0368/0369).

**Offline builds** (Nix sandbox, air-gapped CI): set
`LITIGATE_SMT_Z3_ARCHIVE=/path/to/z3-5.1.0-….zip`. The file is checked
against the same SHA-256. `flake.nix` does this for the Nix packages and
checks: it fetches the zip for the system with `pkgs.fetchurl` (URL and
digest copied from `build.rs`'s `ASSETS`, so bump both together) and sets
`LITIGATE_SMT_Z3_ARCHIVE` in `commonArgs`.

**Docker.** The image builds `litigate-cli` (the `litigate` binary),
which depends on this crate through `litigate-sema`. `cargo chef cook`
swaps every build script for `fn main() {}`, so the cached dependency
layer compiles `ureq`/`ring`/`zip` (ring needs only the image's gcc) but
never downloads Z3; the real `cargo build` then runs `build.rs`, which
fetches the pinned archive and links it statically into `litigate`. The
runtime image ships no separate Z3 library.

**wasm.** Not built for `wasm32-unknown-unknown`: Z3 is a native C++
library and `litgraph-wasm` does not depend on this crate.

**Bumping Z3.** Change, together: the `z3`/`z3-sys` `=` pins in
`Cargo.toml`, `Z3_VERSION` and `ASSETS` (names and SHA-256 from the release's
API `digest` fields) in `build.rs`, and `PINNED_Z3_VERSION` in
`src/z3_backend.rs`. A new version also changes `Z3::fingerprint`, so
every cache key moves with it. The golden key in `tests/cache.rs` does not
include the fingerprint and pins only the query encoding.

## Determinism

Each `Z3::check` gets a fresh Z3 context on the calling thread (no state
carries over between queries) and a solver configured with
`random_seed=0`, `core.minimize=true` and `rlimit` (default
`DEFAULT_RLIMIT`, `Z3::with_rlimit` to change it). Z3's default of one
solver thread stays as it is. `rlimit` counts solver work, not wall time,
so `unknown` is reproducible too, unlike a timeout. Constants are created
in name order, assertions in assertion order. Model values come back as
exact rationals; unsat cores come back as labels in assertion order.
`tests/z3.rs` checks that repeated checks, including from another thread,
return identical answers.

## Caching

`Cached<B>` keys on `ContentHash::of(fingerprint, query)`. The fingerprint
names the solver, its exact version and every answer-relevant option, so a
cache hit is the answer the backend would give again. Errors are not
cached. The cache is in memory. Since `ContentHash` is stable (see the golden
test), a persistent store can be added later without changing keys.

## Tests and coverage

```bash
cargo nextest run -p litigate-smt
cargo +nightly llvm-cov nextest --branch -p litigate-smt   # 100% lines, regions, branches
just cov-litigate litigate-smt                             # the same gate, as CI runs it
```

`build.rs` is not measured by llvm-cov. It was exercised by hand: download,
`LITIGATE_SMT_Z3_ARCHIVE` override, and hash rejection of a corrupt archive.
