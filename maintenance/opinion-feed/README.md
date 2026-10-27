# Opinion feed

`litigate-feed` (crate `crates/litigate-feed`, lg-8fdj.1) polls
[CourtListener](https://www.courtlistener.com/help/api/rest/) once a day
for two kinds of new opinion, and files one bead per opinion in the lg
store:

- **from a modeled court**: every court a promoted package (a package
  under `lit/` with a pack identity) models, by the CourtListener court ids
  in [`crates/litigate-feed/courts.toml`](../../crates/litigate-feed/courts.toml).
  Every promoted package is listed there, with its `courts` or why it has
  `none`. Promoting a package fails `cargo test -p litigate-feed` until it is
  listed.
- **citing a precedent layer**: every package under `lit/` with
  `kind = "precedent"`. Its CourtListener opinion is found from a source
  `url` on `courtlistener.com/opinion/<cluster>/`, else by the citation
  lookup API on its `[package.instrument] designation`. The feed then
  searches for opinions with `cites:(<opinion ids>)`.

Each bead carries the opinion URL, case name, court, date filed, the
CourtListener cluster and opinion ids, why the feed picked the opinion up,
and the **candidate elements** it may edit. These are nodes of the watched
packages (and their dependencies), each with its reason:

- `cite <provision>`: the opinion cites the element's provision, a
  subdivision of it or a provision containing it. This is an eyecite-style
  match on the national rule sets, the U.S. Code and C.F.R., and every
  `cite_syntax`/`cite_prefixes` spelling the packages declare.
- `name "<label>"`: the opinion uses the element's name (two or more
  distinctive words).

The candidates are leads for review, not edits. The bead is a task at
priority 3, labelled `opinion-feed`, `court:<id>` and `pkg:<package>` per
candidate package, with `--external-ref courtlistener:opinion:<lead opinion
id>` (lg-8fdj.4). The dry run prints the same `external_ref` on every bead.

## Running it

```bash
# what it would file since yesterday (nothing filed, nothing written)
just opinion-feed --since 2026-10-26
# the daily run: file the beads
COURTLISTENER_API_TOKEN=... just opinion-feed --file
```

The daily run is pod-dispatched (infra dot-7o4c2d), never a build on
carbon. Its line is `litigate-feed --repo /work/repo --file`. In the pod
`br` is the shim that records each call as a `BR_INTENT` line, so the run
emits one `br create ... --external-ref courtlistener:opinion:<id>` intent
per opinion and needs no writable store; the dispatcher applies the
intents. The run keeps no state file and commits nothing. Its stdout is the
JSON report: `beads`, `deferred`, `requests`, `warnings` and `errors`. It
exits 1 when any opinion was not filed.

- **Dedupe** is on the external ref, by the dispatcher (or the store). A
  run over the same window proposes the identical refs, so rerunning a
  failed or capped day, or the daily one-day overlap, files nothing twice.
  Within a run an opinion found both by its court and by a citation is one
  bead.
- **Window.** A run polls from `--since`, by default yesterday (the
  overlap absorbs late-indexed opinions). A `--max-opinions` cap (default
  100) reports the rest as `deferred`; rerun with the same `--since` to
  file them.
- **Rate limits.** Requests are at least `--min-interval-ms` apart
  (default 1000; CourtListener allows 5,000 an hour with a token). A 429
  or 503 waits its `Retry-After`, else 30 s doubling, and is retried up to
  four times. Set `COURTLISTENER_API_TOKEN` for the authenticated limit.

Public law only. The feed reads published opinions and the packages' own
manifests and builds, and nothing else.
