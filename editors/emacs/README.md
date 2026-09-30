# litgraph-emacs

Emacs front end for [litgraph](https://github.com/davidawad/litgraph), the Rust
engine that models litigation procedure as a stochastic game on a graph. It
takes one JSON request and returns one JSON response; this package makes that
comfortable from Emacs:

- run **any** op asynchronously, with completion for ops, packs, scenarios,
  node ids, edge ids, metrics and parameters, all read from the binary itself
  (new ops in a newer litgraph appear without a package update);
- results as readable buffers (summary, tables, grouped warnings, folded
  provenance, raw JSON one key away);
- **deadlines to Org**: computed due dates become headings with `DEADLINE`
  timestamps, so they show up in the agenda;
- the procedure graph drawn with Graphviz, solved best line highlighted;
- scenario JSON editing with flymake validation and completion;
- an Org Babel backend, `#+begin_src litgraph`.

Nothing blocks Emacs: every call is a `make-process` with a timeout.

> litgraph's numbers are teaching estimates, not legal advice. Result buffers
> print the engine's warnings (`probability-fill`, `mixed-node`, ...) with what
> each one means for how far to trust a figure.

## Requirements

- Emacs 29.1 or newer (uses the built-in `transient`, `flymake`, `json`).
- The `litgraph` binary: `brew install davidawad/tap/litgraph`, or see the
  litgraph README. Ops `settle`, `posterior` and `voi` need a build that has
  them; the menu greys out what your binary lacks.
- Optional: Graphviz (`dot`) for the inline graph image (a text tree is shown
  without it, and always in a terminal frame).

## Install

The package is a directory of files: `litgraph.el` (core), `litgraph-view.el`,
`litgraph-graph.el`, `litgraph-org.el`, `litgraph-scenario.el`,
`litgraph-menu.el` and `ob-litgraph.el`. Entry points are autoloaded.

**use-package with `:vc`** (Emacs 30; `package-vc` in 29):

```elisp
(use-package litgraph
  :vc (:url "https://github.com/davidawad/litgraph-emacs" :rev :newest)
  :bind-keymap ("C-c L" . litgraph-command-map)
  :custom
  (litgraph-default-scenario "cofc-1498-patent-case"))
```

**straight.el**:

```elisp
(straight-use-package
 '(litgraph :type git :host github :repo "davidawad/litgraph-emacs"
            :files ("*.el")))
```

**elpaca**:

```elisp
(elpaca (litgraph :host github :repo "davidawad/litgraph-emacs"
                  :files ("*.el")))
```

**From a local clone**:

```elisp
(add-to-list 'load-path "~/projects/Lawyering/litgraph-emacs")
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
Context          Analyze                          Lines
C Scenario…      s Solve: value and best line     p Paths to a node/outcome
P Packs…         c Chain: outcome odds, cost      f Pareto frontier
V Parameter…     e Explain a decision             u Structure
Z Clear context  m Simulate (Monte Carlo)         x Compare with a variant
                 t Tornado: what drives it
                 w Sweep a parameter
See                      Scenario files       Engine
g Graph (Graphviz)       o Open…              r Run any op…
G Graph around a node    n New…               k Packs
d Deadlines to Org       R Run this buffer    L Lint
                         y Validate buffer    D Describe (the manual)
```

Pick a scenario (`C`, completing from the library or a JSON file), optionally
override a parameter (`V`, e.g. `rate` = 900), then press `s`. The session
context is applied to every op, including graph and deadlines.

`r` (or `M-x litgraph-run-op`) runs any op: complete its name, then answer its
required arguments; optional ones are chosen from a list (or all asked with a
prefix argument). Argument types and help come from `litgraph schema request`.

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

`M-x litgraph-graph` (`g`): draws the scenario's graph. Blue rounded boxes are
your decisions, orange boxes the opponent's, grey ellipses chance nodes;
terminals are green (win), red (loss) or yellow (settlement); chance edges
carry probabilities; the solved best line is red. With Graphviz and a graphical
Emacs it is an inline SVG (`+`/`-` zoom); `t` toggles a text tree (also the
fallback), `d` shows the dot source, `w` writes `.dot`/`.svg`/`.png`. A prefix
argument draws only the neighborhood of a node. `RET` on a tree line explains
that node.

## Scenario editing

- `M-x litgraph-new-scenario` writes a runnable request skeleton extending a
  library scenario; `M-x litgraph-open-scenario` visits one.
- Files named `*.litgraph.json` open in `litgraph-scenario-json-mode`; for
  other JSON files, `M-x litgraph-scenario-mode` or add
  `litgraph-scenario-maybe-enable` to `js-json-mode-hook`.
- Flymake runs `litgraph validate` on the buffer text (unsaved edits count):
  JSON syntax errors first, then the engine's errors (unknown fields, missing
  edges, wrong types), located on the offending token where the message allows.
  `litgraph-flymake-warnings` also shows engine warnings.
- `completion-at-point` inside a string completes ids, fields, packs,
  scenario names, ops and metrics.
- `C-c C-c` runs the document (a request as is; a scenario with an op you
  choose), `C-c C-v` validates now, `C-c C-g` draws its graph, `C-c C-u`
  makes it the session scenario, `C-c C-m` opens the menu.

## Org Babel

```org
#+begin_src litgraph
{"scenario": "cofc-1498-patent-case", "op": {"op": "chain"}}
#+end_src

#+RESULTS:
| Outcome                                      |        p |   Utility | Tags                        |
|----------------------------------------------+----------+-----------+-----------------------------|
| JUDGMENT FOR PLAINTIFF (Bench Trial Verdict) | 0.217164 |   1000000 | win, judgment, fee-eligible |
| ...                                          |          |           |                             |
```

Header arguments: `:op` (name or JSON, when the body is just a scenario),
`:scenario`, `:packs a,b`, `:table REGEXP` (which table of the result),
`:format table|summary|json|org`, and `:var rate=900` (numbers become scenario
parameters). A block never picks up the menu's session scenario, so it is
reproducible. Warnings are reported in the echo area.

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
| `o` `n` `v` | open / new scenario, validate buffer |
| `C` `P` `.` | set scenario / packs / parameter |
| `k` | cancel running processes |

## Customization

| variable | meaning |
|----------|---------|
| `litgraph-executable` | binary name or path (Homebrew paths are tried when it is not on `exec-path`) |
| `litgraph-packs-dir` | exported as `LITGRAPH_PACKS` |
| `litgraph-scenarios-dir` | exported as `LITGRAPH_SCENARIOS` |
| `litgraph-default-scenario` / `litgraph-default-packs` | used until you pick your own |
| `litgraph-timeout` | seconds before a call is killed (120) |
| `litgraph-view-*`, `litgraph-graph-*`, `litgraph-org-*`, `litgraph-scenario-*` | see `M-x customize-group RET litgraph` |

## Development

```
make test       # ERT: stubbed process layer + real binary when installed
make compile    # byte-compile, warnings are errors
make lint       # checkdoc + package-lint (PACKAGE_LINT_DIR=... or installed into .deps/)
make all
```

Unit tests replace `litgraph-process-function` with a stub that answers from
recorded real output in `test/fixtures/`; integration tests
(`test/litgraph-integration-test.el`) skip themselves when no `litgraph`
binary is found.

## License

GPL-3.0-or-later, matching litgraph. See `LICENSE`.
