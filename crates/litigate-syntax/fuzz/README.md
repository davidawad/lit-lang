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

## Corpus

`corpus/` and `artifacts/` are ignored by git: a corpus is a build
product of a run, not a source. Seed each target from public-law inputs
only (the seeds above, the committed `lit/` packages); never from a
client folder.

| Target | Seed from |
|---|---|
| `parse`, `case_replay` | `seeds/` |
| `manifest_lock` | every committed `lit/**/Lit.toml` and `lit/**/Lit.lock` |
| `lowering` | none needed: the input picks only a date, so libFuzzer's empty start covers it |

```bash
FUZZ=crates/litigate-syntax/fuzz
mkdir -p $FUZZ/corpus/{parse,case_replay,manifest_lock,lowering}
cp -n $FUZZ/seeds/* $FUZZ/corpus/parse/
cp -n $FUZZ/seeds/* $FUZZ/corpus/case_replay/
for f in $(git ls-files 'lit/**/Lit.toml' 'lit/**/Lit.lock'); do
  cp "$f" "$FUZZ/corpus/manifest_lock/$(echo "$f" | tr / _)"
done
```

### Long runs in a pod

A pod's disk is lost with it, so run each target against a corpus on a
mounted volume (or copy it out before the pod exits), and keep that
corpus between runs: libFuzzer resumes from it and only adds inputs that
reach new coverage. Run one target at a time per core and bound each run:

```bash
cargo +nightly fuzz run manifest_lock --fuzz-dir $FUZZ "$KEEP/manifest_lock" \
  -- -max_total_time=3600 -artifact_prefix="$KEEP/artifacts/manifest_lock/"
```

After a run, minimize the corpus before archiving it, so it stays small
enough to keep. Merge into a fresh directory and swap it in: `cargo fuzz
cmin` renames across directories and fails (`Invalid cross-device link`)
when the corpus is on a mounted volume.

```bash
mkdir "$KEEP/manifest_lock.min"
cargo +nightly fuzz run manifest_lock --fuzz-dir $FUZZ \
  "$KEEP/manifest_lock.min" "$KEEP/manifest_lock" -- -merge=1
rm -r "$KEEP/manifest_lock" && mv "$KEEP/manifest_lock.min" "$KEEP/manifest_lock"
```

A crash leaves its input under the artifact prefix. Reproduce it with
`cargo +nightly fuzz run <target> --fuzz-dir $FUZZ <crash-file>`, file it
as an issue, and turn the input into a regular unit test of the crate it
breaks once fixed; keep the input in the corpus too. Do not commit the
corpus: a corpus input that matters becomes a test or a file in `seeds/`.
