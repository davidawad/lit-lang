# Beta CLI error UX matrix

DECIDED: use the shared `litigate run TARGET --op ...` boundary for this matrix -- every named operation is dispatched through the same strict request parser and response envelope.

This matrix records the CLI contract and the targeted validation performed in this run. Exit 1 means usage or I/O failed before an engine request; exit 2 means the request ran and failed. Errors before engine dispatch are CLI diagnostics, not engine `{ok:false, api_version, op, error:{code,message,hint}, elapsed_ms}` envelopes.

## Operations

`describe` returns the embedded op catalog. The remaining operation names are the serialized `Op` variants in `crates/litigate-engine/src/api/op.rs`.

| op | dispatch | missing target | malformed target | unknown `--set` | unknown node | out-of-interval date |
|---|---|---|---|---|---|---|
| describe | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | op-specific resolution | deadline validation |
| packs | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| lint | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| validate | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| graph | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | engine error envelope | n/a |
| metric | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| explain | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | engine error envelope | n/a |
| solve | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| chain | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| simulate | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| path | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | engine error envelope | n/a |
| pareto | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | engine error envelope | n/a |
| sweep | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | engine error envelope | n/a |
| tornado | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| calibration | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| posterior | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| voi | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| structure | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | engine error envelope | n/a |
| deadlines | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | engine error envelope | date validation |
| settle | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | n/a | n/a |
| compare | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | inner op dependent | n/a |
| batch | shared `run` | CLI I/O diagnostic | compile diagnostic | scenario/engine validation | inner op dependent | n/a |

## Input probes

| probe | expected boundary/result | status |
|---|---|---|
| nonexistent target | exit 1; plain CLI I/O diagnostic | not run in this pass |
| malformed `.case` syntax | exit 2; compiler diagnostic with source location | not run in this pass |
| unknown `--set` key | strict scenario/engine validation error | not run in this pass |
| unknown node id | operation error envelope with code, message, hint | not run in this pass |
| date outside all effective intervals | date/effective interval validation error | not run in this pass |

No panic rows were observed in the probe executed: `litigate run cases/cofc-1498-patent-case.case --op describe` returned a JSON response. The full cross-product and malformed-input probes remain TODO; this document is not evidence that those rows have passed.
