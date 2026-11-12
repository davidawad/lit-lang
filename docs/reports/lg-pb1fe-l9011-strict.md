# L9011 strict profile corpus comparison

Compared main `501999b8` with the rebased tree `be6c8426`. For each of the 635 directories under `lit/` containing `Lit.toml`, ran the release CLI command `litigate check --profile strict lit/<package> --format json` and counted process exit codes and diagnostic codes.

| Measure | Main `501999b8` | Rebased `be6c8426` |
|---|---:|---:|
| Packages checked | 635 | 635 |
| Packages failing strict | 149 | 149 |
| Error diagnostics by code | `{"L5002": 1744}` | `{"L5002": 1744}` |
| L9011 warnings | 2 | 2 |
| L9011 errors | 0 | 0 |

Package exit-code changes: 0 (`none`). The failing package sets are identical: 149 packages fail on both sides, and no package newly fails or newly passes.

## Strict L9011 warnings on the rebased tree

Each warning remains a warning because the deadline ends before the first recorded version of its counting rule. These use the `deadline-predates-rule` exemption; no no-discriminating-date warnings remain in this corpus sweep.

| Package | Deadline | Source | Exemption note |
|---|---|---|---|
| `lit/bankr-mie` | `completion-objection-period` | `lit/bankr-mie/local-rules.lit:403` | the deadline ends before the first recorded version of its counting rule |
| `lit/bankr-ri` | `local-counsel-appearance-period` | `lit/bankr-ri/local-rules.lit:941` | the deadline ends before the first recorded version of its counting rule |

The two warning rows above are the complete strict-profile L9011 warning set. No package newly fails, so no `litigate fix` edits or Lit.lock changes were required.

## Gate results

- `cargo build --release -p litigate-cli` on main and the rebased tree: both exited 0.
- `cargo nextest run -p litigate-verify -p litigate-syntax -p litigate-base --no-fail-fast`: exit 0; 394 passed, 0 skipped.
- `cargo nextest run -p litigate-cli --no-fail-fast -E 'binary(tutorial) | binary(core_stdlib) | binary(change_examples) | binary(change_files) | binary(check) | binary(corpus) | binary(roundtrip) | test(/strict|L9011|counting/)'`: exit 0; 71 passed (1 slow), 587 skipped.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo fmt --all --check`: exit 0.
- `target/release/litigate check lit cases`: exit 0; response `ok: true`, `elapsed_ms: 305979.119046`.
- `target/release/litigate fmt --check lit cases`: exit 0; response `ok: true`, `elapsed_ms: 384.256768`.
