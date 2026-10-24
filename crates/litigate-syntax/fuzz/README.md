# litigate-syntax fuzzing

`parse` feeds arbitrary UTF-8 to both grammars and checks that parsing is
total and lossless and that formatting a clean parse is idempotent.
`case_replay` stresses `.case` recovery and AST traversal over its history,
facts and analysis inputs.
`manifest_lock` fuzzes the `Lit.toml` and `Lit.lock` readers, checking that
every accepted lock survives canonical render/parse. `lowering` passes
seed-driven dates through the worked package's actual HIR-to-pack lowering,
then checks the result with the engine's pack parser and semantic roundtrip.

```bash
cargo install cargo-fuzz
cp -r crates/litigate-syntax/fuzz/seeds crates/litigate-syntax/fuzz/corpus/parse
cp -r crates/litigate-syntax/fuzz/seeds crates/litigate-syntax/fuzz/corpus/case_replay
cargo +nightly fuzz run parse --fuzz-dir crates/litigate-syntax/fuzz
cargo +nightly fuzz run case_replay --fuzz-dir crates/litigate-syntax/fuzz
cargo +nightly fuzz run manifest_lock --fuzz-dir crates/litigate-syntax/fuzz
cargo +nightly fuzz run lowering --fuzz-dir crates/litigate-syntax/fuzz
```

For a documented 10-minute local smoke of both targets:

```bash
cargo +nightly fuzz run parse --fuzz-dir crates/litigate-syntax/fuzz -- -max_total_time=300
cargo +nightly fuzz run case_replay --fuzz-dir crates/litigate-syntax/fuzz -- -max_total_time=300
```

`seeds/` holds the SPEC §12 worked examples and the syntax fixtures.
