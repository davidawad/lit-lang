# Beta walkthrough record

Run from this checkout on 2026-11-07. The checkout is `v0.2.1`-tagged
history plus the current working branch, and the installed CLI reports
`litigate-cli v0.6.0`. Commands below were run from `/work/repo`; the
temporary tutorial package followed the examples in
[`docs/litigate/TUTORIAL.md`](../litigate/TUTORIAL.md) and was removed after
the run.

## Install and first matter

`./scripts/beta/install-smoke.sh` creates fresh temporary `CARGO_HOME` and
`CARGO_TARGET_DIR` directories, installs the local source checkout, runs the
README Quickstart, and deletes the temporary directories on exit. It exited
0 on the initial run and again after the reporting-instructions edit. Its
commands and exit codes were:

| Command | Exit |
|---|---:|
| `cargo install --path crates/litigate-cli` | 0 |
| `litigate solve cases/cofc-1498-patent-case.case` | 0 |
| `litigate chain cases/cofc-1498-patent-case.case --set rate=900` | 0 |
| `litigate explain cases/frcp-civil-case.case --arg node=frcp-civil-procedure::pre-filing` | 0 |
| `litigate sweep cases/frcp-civil-case.case --arg param=rate --arg lo=100 --arg hi=1500` | 0 |
| `litigate deadlines lit/frcp --arg trigger=2025-01-06 --arg reachable=true` | 0 |
| `litigate solve lit/links` | 0 |
| `litigate run cases/cofc-1498-patent-case.case --op '{"op":"simulate","runs":50000}'` | 0 |

The beta guide's own first-matter commands also exited 0:

| Command | Exit |
|---|---:|
| `cargo run -q -p litigate-cli -- solve cases/cofc-1498-patent-case.case` | 0 |
| `cargo run -q -p litigate-cli -- explain cases/cofc-1498-patent-case.case --arg node=cofc::sj-ruling` | 0 |
| `cargo run -q -p litigate-cli -- run cases/cofc-1498-patent-case.case --op '{"op":"simulate","runs":20}'` | 0 |
| `cargo run -q -p litigate-cli -- check cases/cofc-1498-patent-case.case` | 0 |

The solve output included `fact-unset` (2), `mixed-node` (5), and
`probability-fill` (27). The guide correctly explains these are modeling
fallbacks; the scenario's results are illustrative, not predictions.

## Language tutorial

Created the tutorial's `lit/my-court` package and
`cases/acme-v-roadrunner.case` under the checkout, using the source and case
text shown in the tutorial. The commands all exited 0; `check lit/my-court
cases` checked the repository cases as well as the temporary matter.

| Command | Exit |
|---|---:|
| `cargo run -q -p litigate-cli -- lock lit/my-court` | 0 |
| `cargo run -q -p litigate-cli -- fmt lit/my-court` | 0 |
| `cargo run -q -p litigate-cli -- check lit/my-court` | 0 |
| `cargo run -q -p litigate-cli -- test lit/my-court` | 0 |
| `cargo run -q -p litigate-cli -- lock cases` | 0 |
| `cargo run -q -p litigate-cli -- check lit/my-court cases` | 0 |
| `cargo run -q -p litigate-cli -- run cases/acme-v-roadrunner.case --op solve` | 0 |

The inline tests passed (3/3). The sample solve returned value `34800`,
selected answering, and gave the merits loss probability as `0.6`, matching
the tutorial's expected result. Temporary tutorial files and the generated
case lock changes were removed after the run.

## Editor setup

| Command | Exit / result |
|---|---|
| `cargo install --path crates/litigate-lsp` | 0; installed `litigate-lsp` v0.6.0 |
| `cargo install --path crates/litigate-cli` | 0; installed `litigate` v0.6.0 beside the LSP |
| `litigate --help` | 0 |
| `cargo test -p litigate-editors` | 0; 8 unit tests, 1 Emacs test, 2 generated-file tests, and 5 Helix/query tests passed; Neovim test passed |
| LSP stdio initialize, shutdown, exit through `litigate lsp --stdio` | 0; received initialize and shutdown replies and the process exited successfully |

The Emacs integration test ran under GNU Emacs 30.1. No `hx`, `nvim`,
`docker`, or `podman` executable is installed. The editor test suite covers
grammar/query behavior without those clients; the GUI health checks in the
Helix and Neovim instructions could not be performed here.

## Limits and problem reporting

Read the guide's modeled-scope/date and known-limitations sections, including
the strict-baseline count and its package list, the absence of ongoing
good-law tracking, and incomplete state-holiday sources. No additional
procedural or legal claims were inferred from the engine output.

The documented report path is GitLab Issues, with the command, relevant
package or matter, `litigate --version`, and full response envelope. The
guide now says a GitLab account and sign-in are required; this was missing
from the first-run instructions. No issue was submitted. The anonymous
request to the new-issue page redirected to GitLab sign-in (HTTP 403 in this
pod).

## Environment-limited checks

The host smoke script used fresh Cargo and build directories but was not run
inside a nested container. `bash scripts/beta/install-smoke.sh --docker`
exited 1 with `install-smoke: docker is required for --docker`.

The newest remote `v*` tag was `v0.2.1`. The unauthenticated GitLab Releases
API request for `davidawad/lit-lang` returned HTTP 404, so no Linux musl
release asset could be identified. GitLab's project and new-issue pages
redirected to sign-in (HTTP 403) in this pod. No binary release was
downloaded or executed.

The relevant discovery commands and results were:

| Command | Result |
|---|---|
| `git ls-remote --tags origin 'refs/tags/v*' \| sort -V \| tail -5` | Exit 0; newest tag `v0.2.1` |
| `curl -sS -w '\nHTTP %{http_code}\n' 'https://gitlab.com/api/v4/projects/davidawad%2Flit-lang/releases'` | Exit 0; HTTP 404 |
| `curl -sSL -o /dev/null -w 'repo_http=%{http_code} final=%{url_effective}\n' https://gitlab.com/davidawad/lit-lang` | Exit 0; HTTP 403, redirected to sign-in |
| `curl -sSL -o /dev/null -w 'issues_http=%{http_code} final=%{url_effective}\n' https://gitlab.com/davidawad/lit-lang/-/issues/new` | Exit 0; HTTP 403, redirected to sign-in |
| `bash scripts/beta/install-smoke.sh --docker` | Exit 1; Docker unavailable |

DECIDED: run the documented host smoke when Docker is absent -- it gives a
fresh Cargo home and target without assuming Docker.
