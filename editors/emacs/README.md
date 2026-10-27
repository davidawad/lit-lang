# litgraph for Emacs

Emacs front end for [litgraph](https://github.com/davidawad/litgraph), the Rust
engine that models litigation procedure as a stochastic game on a graph. The
`litigate` command line compiles a matter (a `.case` file) or a package (a
`.lit` file or package directory) and runs one engine op on it, `litigate run
TARGET --op OP`; this package makes that comfortable from Emacs:

- run **any** op asynchronously on a matter or package, with completion for
  targets, ops, node ids, edge ids, metrics and parameters, all read from the
  project and the binary itself (new ops in a newer litigate appear without a
  package update);
- results as readable buffers (summary, tables, grouped warnings, folded
  provenance, raw JSON one key away);
- **deadlines to Org**: computed due dates become headings with `DEADLINE`
  timestamps, so they show up in the agenda;
- the procedure graph drawn with Graphviz, solved best line highlighted;
- a **timeline** of any `.case` or `.lit` file, `magit-log --graph` style:
  worlds (alternate timelines) as branches, layers you can switch off;
- an Org Babel backend, `#+begin_src litgraph`;
- `litigate-ts-mode`, the tree-sitter major mode for `.lit` and `.case`
  files (`litigate-ts-mode.el`; see [`../README.md`](../README.md) for its
  grammar and language server setup).

This package used to live in its own repository, litgraph-emacs; it was
imported here, history included, at its `996543d`.

Nothing blocks Emacs: every call is a `make-process` with a timeout.

> litgraph's numbers are teaching estimates, not legal advice. Result buffers
> print the engine's warnings (`probability-fill`, `mixed-node`, ...) with what
> each one means for how far to trust a figure.

## Requirements

- Emacs 29.1 or newer (uses the built-in `transient`, `flymake`, `json`).
- The `litigate` binary: `cargo install --path crates/litigate-cli` from a
  checkout of this repository. Every engine call runs `litigate run TARGET
  --op OP [--set NAME=VALUE ...] --format json`; the timeline runs `litigate
  timeline`.
- A project: a directory holding `lit/` (the packages a matter `uses`) and
  `cases/` (matters). Emacs runs `litigate` in `litgraph-project-directory`,
  else in the nearest directory above the current buffer's that holds `lit/`
  (a litgraph checkout, or your own project). Every call compiles its target.
- Optional: Graphviz (`dot`) for the inline graph image (a text tree is shown
  without it, and always in a terminal frame).

## Install

The package is the `editors/emacs` directory of this repository:
`litgraph.el` (core, the package's main file), `litgraph-view.el`,
`litgraph-graph.el`, `litgraph-org.el`, `litgraph-menu.el`, `litgraph-timeline.el`, `ob-litgraph.el` and
`litigate-ts-mode.el`. Entry points are autoloaded.

**use-package with `:vc`** (Emacs 30; `package-vc` in 29):

```elisp
(use-package litgraph
  :vc (:url "https://github.com/davidawad/litgraph" :lisp-dir "editors/emacs"
       :rev :newest)
  :bind-keymap ("C-c L" . litgraph-command-map)
  :custom
  (litgraph-default-target "cofc-1498-patent-case")) ; cases/cofc-1498-patent-case.case
```

**straight.el**:

```elisp
(straight-use-package
 '(litgraph :type git :host github :repo "davidawad/litgraph"
            :files ("editors/emacs/*.el")))
```

**elpaca**:

```elisp
(elpaca (litgraph :host github :repo "davidawad/litgraph"
                  :files ("editors/emacs/*.el")))
```

**From a local clone**:

```elisp
(add-to-list 'load-path "/path/to/litgraph/editors/emacs")
(require 'litgraph-autoloads nil t) ; after `make autoloads', or just:
(autoload 'litgraph-menu "litgraph-menu" nil t)
(global-set-key (kbd "C-c L") #'litgraph-menu)
```

Org Babel (put in your Org config):

```elisp
(with-eval-after-load 'org
  (org-babel-do-load-languages 'org-babel-load-languages
                               '((litgraph . t))))
```

## Quick start

`M-x litgraph-menu` (or `C-c L m` with the keymap bound above):

```
litgraph  cofc-1498-patent-case
Context                Analyze                          Lines
C Matter or package…   s Solve: value and best line     p Paths to a node/outcome
V Parameter…           c Chain: outcome odds, cost      f Pareto frontier
Z Clear context        e Explain a decision             u Structure
                       m Simulate (Monte Carlo)         x Compare with a variant
                       t Tornado: what drives it
                       w Sweep a parameter
See                      Engine
g Graph (Graphviz)       r Run any op…
G Graph around a node    k Packs
d Deadlines to Org       L Lint
T Timeline of a file     D Describe (the manual)
```

Pick a target (`C`: a matter from the project's `cases/`, a package from
`lit/`, or RET for any `.case`, `.lit` or package directory; a bare name such
as `cofc-1498-patent-case` means `cases/NAME.case`), optionally override a
parameter (`V`, e.g. `rate` = 900, passed as `--set rate=900`), then press
`s`. The session context is applied to every op, including graph and
deadlines.

`r` (or `M-x litgraph-run-op`) runs any op: complete its name, then answer its
required arguments; optional ones are chosen from a list (or all asked with a
prefix argument). Argument names come from the op summaries of the target's
`describe`.

## Result buffers

```
litgraph chain  cofc-1498-patent-case  1.18 ms

* Summary
  Start             Claim Against the United States Accrues
  Expected net      48,423
  Expected utility  300,927
  Odds              21.7% JUDGMENT FOR PLAINTIFF (Bench Trial Verdict)  ·  21.7% JUDGMENT FOR THE GOVERNMENT ...
  Expected dollars  252,504
  Expected hours    388

* Outcomes
  Outcome                                           p      Utility    Tags
  JUDGMENT FOR PLAINTIFF (Bench Trial Verdict)      21.7%  1,000,000  win, judgment, fee-eligible
  SETTLED VIA ADR                                   18.6%    450,000  settlement
  ...

* Warnings (3 kinds)
  fact-unset x2 -- a matter fact is not set; the authored prior stands in for it
      at cofc::limitations-check, cofc::section-1500-check
  probability-fill x27 -- probabilities were missing on some edges and got filled by a rule
  ...

* Provenance            (folded)
```

| key | action |
|-----|--------|
| `TAB` / `S-TAB` | fold section / all sections |
| `n` `p` | next / previous heading |
| `j` | raw JSON, pretty-printed |
| `T` | table at point as a sortable `tabulated-list` (numeric sort) |
| `o` | copy the table at point as an Org table |
| `RET` | drill into the row: explain a node, or sweep a parameter (`param:rate`) |
| `g` | re-run this request |
| `G` | draw the graph |
| `d` | (deadlines results) file into Org |
| `?` | the menu |

Each op has a renderer (`solve`, `chain`, `simulate`, `tornado`, `path`,
`pareto`, `sweep`, `explain`, `deadlines`, `compare`, `batch`, `packs`, `lint`,
`describe`, `calibration`); anything else, including ops added by newer
builds, gets a generic renderer that shows every scalar and turns lists of
objects into tables. Extend with `litgraph-view-renderers`.

## Deadlines to Org

`M-x litgraph-deadlines-to-org` (`d` in the menu): pick the **trigger date**
(the Org date prompt) and the **event node** whose outgoing deadlines you want
(say, service completed), then name the matter. litgraph applies the forum's
rule set (FRCP 6, RCFC 6, FRAP 26, 19 CFR 210.6(a)), weekend and holiday
rolls included, and each deadline is filed in `litgraph-org-file` (default
`org-directory/litgraph-deadlines.org`):

```org
* Matter: Acme v. United States                                    :litgraph:
:PROPERTIES:
:LITGRAPH_MATTER: Acme v. United States
:CATEGORY: Acme v. United States
:END:
** TODO United States answers (RCFC 12(a)(1)(A))                   :deadline:
DEADLINE: <2026-11-30 Mon>
:PROPERTIES:
:LITGRAPH_ID: Acme v. United States|cofc::e-responsefork-answer
:LITGRAPH_TRIGGER: 2026-10-01
:LITGRAPH_PERIOD: 60 calendar days
:LITGRAPH_RULES: RCFC 6
:LITGRAPH_AUTHORITY: RCFC 12(a)(1)(A)
:LITGRAPH_SOURCE: https://www.uscfc.uscourts.gov/rules
:END:
:LITGRAPH_STEPS:
Due 2026-11-30 (RCFC 6).
RCFC 6(a)(1)(A): exclude the trigger date 2026-10-01
RCFC 6(a)(1)(B): count 60 calendar day(s) after the trigger, ...
:END:
```

- The matter's `CATEGORY` makes each agenda line say which case it belongs to.
- **Filing is idempotent.** The `LITGRAPH_ID` (matter plus graph edge) lets you
  re-run with a corrected trigger date: the DEADLINE, properties and
  computation drawer update in place, and your TODO state, notes and other
  edits are left alone.
- If the file is not in `org-agenda-files`, you are offered to add it for the
  session (`litgraph-org-offer-agenda`).
- With a prefix argument you are asked every option of the `deadlines` op
  (service method, extra holidays, downstream deadlines, clerk closure).
- Customize `litgraph-org-todo-keyword`, `litgraph-org-tags`,
  `litgraph-org-warning-period` (e.g. `"-14d"`), `litgraph-org-title-function`.

## Graph view

`M-x litgraph-graph` (`g`): draws the target's graph. Blue rounded boxes are
your decisions, orange boxes the opponent's, grey ellipses chance nodes;
terminals are green (win), red (loss) or yellow (settlement); chance edges
carry probabilities; the solved best line is red. With Graphviz and a graphical
Emacs it is an inline SVG (`+`/`-` zoom); `t` toggles a text tree (also the
fallback), `d` shows the dot source, `w` writes `.dot`/`.svg`/`.png`. A prefix
argument draws only the neighborhood of a node. `RET` on a tree line explains
that node.

## Timeline

`M-x litgraph-timeline` (`T` in the menu and in `litgraph-command-map`, and
`C-c C-t` in a `litigate-mode` buffer) shows the timeline of a `.case` or
`.lit` file: one row per event, deadline or fork in date order, with one graph
lane per world. A fork row is where worlds diverge (an assumed order, a grant
or denial, a hypothetical decision); the line under it names the worlds that
leave it. A world shows its parent's rows up to the fork row plus its own;
a row the world re-emits (the same `key`) replaces the parent's row, so `w`
on `w1` shows the original answer date only as superseded.

```
       Date        Kind      Status      Layer       Event or deadline
*      2026-04-06  event     none        base_rule   Motion to reset the answer date filed  [37 C.F.R. § 2.106(a)]
+      —           fork      unknown     base_rule   Ruling on the motion — no ruling recorded as of 2026-04-08
|\-\   w1 reset granted, w2 reset denied
| | *  —           event     none        base_rule   Board denies the motion (hypothetical)
| *    2026-04-10  event     none        case_order  Board order resets the answer date to 2026-05-13  [37 C.F.R. § 2.106(a)]
o |    2026-04-13  deadline  open        base_rule   Answer due (designated 2026-04-11, a Saturday; rolls to Monday)  [...]
  o    2026-04-13  deadline  superseded  base_rule   Answer due (original designation) — reset by order-reset-1  [...]
  o    2026-05-13  deadline  open        case_order  Answer due (as reset)  [37 C.F.R. § 2.106(a)]
```

Status and layer class are printed exactly as the op's JSON spells them
(`met`, `open`, `late`, `lapsed`, `breached`, `unknown`, `superseded`, `none`;
`base_rule`, `local_rule`, `standing_order`, `general_order`,
`emergency_declaration`, `case_order`, `decision`); a status with a reason
(always for `unknown` and `superseded`) shows it after the label.

| key | action |
|-----|--------|
| `RET` | jump to the row's `.case`/`.lit` span; without one, to its cite in the file (or `litgraph-timeline-cite-function`) |
| `=` | diff two worlds: common rows, changed rows (same `key`), rows only in one |
| `l` | toggle a layer on or off (the layer on the line, else asked) |
| `t` | set `as_of` and `known_at` (empty for the default) |
| `w` | show one world only; again for all |
| `g` | refresh; saving the file, or a `.lit` file a row comes from, refreshes too |
| `n` / `p` | next / previous row |
| `j` | raw JSON |

The buffer only renders the JSON of litgraph's `timeline` op (schema
`litigate.timeline/1`, frozen in litgraph's `docs/litigate/TIMELINE.md`). How
the op is called is one option, `litgraph-timeline-command` (defaulting to
the `litigate` executable):

```elisp
(setq litgraph-timeline-command
      '(:program "litigate"
        :args ("timeline" "--format" "json" file) ; `file' is the .case/.lit file
        :as-of "--as-of" :known-at "--known-at"
        :layers "--layers"              ; layers switched off, as -ID,-ID
        :world "--world"))
```

The `litigate-mode` key is `litgraph-timeline-litigate-key` (nil for none); it
is bound when `litigate-mode` (or the older `litigate-ts-mode`) loads.

## Org Babel

```org
#+begin_src litgraph :target cofc-1498-patent-case
chain
#+end_src

#+RESULTS:
| Outcome                                      |        p |   Utility | Tags                        |
|----------------------------------------------+----------+-----------+-----------------------------|
| JUDGMENT FOR PLAINTIFF (Bench Trial Verdict) | 0.217164 |   1000000 | win, judgment, fee-eligible |
| ...                                          |          |           |                             |
```

The body is the op: its name, or an op object as JSON (`{"op": "deadlines",
"trigger": "2026-10-01"}`); empty means `solve`. Header arguments: `:target`
(required: a `.case` matter, `.lit` file or package directory, or a bare
matter name), `:op` (name or JSON, replacing the body's), `:table REGEXP`
(which table of the result), `:format table|summary|json|org`, and `:var
rate=900` (numbers become `--set` scenario parameters). A block never picks up
the menu's session target, so it is reproducible. Warnings are reported in
the echo area.

## Keybindings

`litgraph-command-map` is not bound for you. Suggested:
`(global-set-key (kbd "C-c L") litgraph-command-map)`, then:

| key | command |
|-----|---------|
| `m` | `litgraph-menu` |
| `r` | `litgraph-run-op` (any op) |
| `s` `c` `x` `S` `p` `t` `w` | solve, chain, explain, simulate, path, tornado, sweep |
| `g` | `litgraph-graph` |
| `d` | `litgraph-deadlines-to-org` |
| `T` | `litgraph-timeline` |
| `C` `.` | set target / parameter |
| `k` | cancel running processes |

## Customization

| variable | meaning |
|----------|---------|
| `litgraph-executable` | `litigate` binary name or path (Homebrew and `~/.cargo/bin` paths are tried when it is not on `exec-path`) |
| `litgraph-project-directory` | where `litigate` runs: the directory holding `lit/` and `cases/` (nil: the nearest one holding `lit/`) |
| `litgraph-default-target` | the matter or package used until you pick your own |
| `litgraph-timeout` | seconds before a call is killed (120) |
| `litgraph-timeline-command` | how the timeline op is called (see Timeline) |
| `litgraph-view-*`, `litgraph-graph-*`, `litgraph-org-*`, `litgraph-timeline-*` | see `M-x customize-group RET litgraph` |

## Development

From the repository root, `just emacs-test` (part of `just ci`) byte-compiles
the package, runs checkdoc and runs the ERT suite with `LITGRAPH_IT_REQUIRE=1`
against the debug `litigate` (`LITIGATE_BIN`), with Emacs and Graphviz from
`nix shell nixpkgs#emacs-nox nixpkgs#graphviz`. In this directory:

```
make test       # ERT: stubbed process layer + real litigate when installed
make compile    # byte-compile, warnings are errors
make lint       # checkdoc + package-lint (PACKAGE_LINT_DIR=... or installed into .deps/)
make all
make fixtures   # regenerate test/fixtures/ from the binary (LITIGATE=path)
```

Unit tests replace `litgraph-process-function` with a stub that answers from
recorded real `litigate run` output in `test/fixtures/` (`make fixtures`
regenerates it with `scripts/fixtures.sh`, from this repository's
`cases/cofc-1498-patent-case.case`); integration tests
(`test/litgraph-integration-test.el`) run the binary named by `LITIGATE_BIN`,
else the `litigate` on `PATH`, skip themselves when there is none, and fail
instead under `LITGRAPH_IT_REQUIRE=1`. Every call compiles its matter, which a
debug build does slowly.
`test/litigate-ts-mode-test.el` is litigate-ts-mode's suite; it needs the
tree-sitter grammar, so `cargo nextest run -p litigate-editors` builds the
grammar and runs it.

The timeline tests render `test/fixtures/timeline-ttab-opposition.json`, the
example of the frozen timeline schema; `timeline.v1.schema.json` beside it is
the schema itself (both copied from `docs/litigate/` at `76005c3`), and the
tests check the renderer's statuses and layer classes against its enums. When
the schema changes, copy both again. The timeline integration test runs on
`test/fixtures/timeline-project/cases/examples/acme.case`, a copy of this
repository's `cases/examples` and `lit/examples`, or on
`LITGRAPH_TIMELINE_FILE`; `make fixtures-sync` refreshes the copy.

## License

GPL-3.0-or-later, matching litgraph. See `LICENSE`.
