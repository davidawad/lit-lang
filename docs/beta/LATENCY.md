# Beta CLI latency

## Compiled-package cache (lg-zmqc.14)

Measured in the repository root on 2026-11-02 with the release binary from
`CARGO_BUILD_JOBS=4 cargo build --release -p litigate-cli`. The command was
`target/release/litigate run cases/e2e-ttab-unknown.case --op describe`, with
stdout discarded and Bash's `time` builtin reporting wall time:

| Run | Cache setting | Wall time |
| --- | --- | ---: |
| Uncached control | `LITIGATE_NO_CACHE=1` | 6.125 s |
| First cached run (populate) | `LITIGATE_CACHE_DIR=/tmp/litigate-lg-zmqc14-a4-fresh-20261102` | 5.899 s |
| Repeat cached run | same cache directory | 0.500 s |

The repeat was below the 1 s target. The cache key includes the compiler
version, target and library arguments, link mode, and
the contents and paths of the source, matter, manifest, lock, and vendored
source text inputs. A changed input selects a different cache entry.
