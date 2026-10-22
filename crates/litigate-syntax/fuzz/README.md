# litigate-syntax fuzzing

`parse` feeds arbitrary UTF-8 to both grammars and checks that parsing is
total and lossless and that formatting a clean parse is idempotent.

```bash
cargo install cargo-fuzz
cp -r crates/litigate-syntax/fuzz/seeds crates/litigate-syntax/fuzz/corpus/parse
cargo +nightly fuzz run parse --fuzz-dir crates/litigate-syntax/fuzz
```

`seeds/` holds the SPEC §12 worked examples and the syntax fixtures.
