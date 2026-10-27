;;; litgraph-menu.el --- Transient front door for litgraph -*- lexical-binding: t; -*-

;; Copyright (C) 2026 David Awad

;; Author: David Awad <me@davidaw.ad>
;; Maintainer: David Awad <me@davidaw.ad>
;; Keywords: tools, convenience, law
;; SPDX-License-Identifier: GPL-3.0-or-later

;; This file is not part of GNU Emacs.

;; This program is free software: you can redistribute it and/or modify
;; it under the terms of the GNU General Public License as published by
;; the Free Software Foundation, either version 3 of the License, or
;; (at your option) any later version.

;; This program is distributed in the hope that it will be useful,
;; but WITHOUT ANY WARRANTY; without even the implied warranty of
;; MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
;; GNU General Public License for more details.

;; You should have received a copy of the GNU General Public License
;; along with this program.  If not, see <https://www.gnu.org/licenses/>.

;;; Commentary:

;; `litgraph-menu' is a transient menu: set the target (a `.case'
;; matter or a package) and parameters at the top, then run an
;; analysis, draw the graph, file deadlines in Org, or show a timeline.  Ops that the installed
;; binary does not support (`posterior', `voi' and `settle' arrive with
;; newer litgraph builds) are shown greyed out; `r' runs any op the binary
;; reports, so ops added later need no change here.

;;; Code:

(require 'transient)
(require 'litgraph)

(declare-function litgraph-graph "litgraph-graph")
(declare-function litgraph-timeline "litgraph-timeline" (file))

;; Ops that only newer litgraph builds have get their own commands, so the
;; menu can offer them (greyed out where the binary lacks them).
(litgraph--define-op-command litgraph-calibration "calibration"
  "Which uncalibrated inputs deserve real data next.")
(litgraph--define-op-command litgraph-posterior "posterior"
  "Credible intervals and P(option optimal) under uncertain probabilities.")
(litgraph--define-op-command litgraph-voi "voi"
  "What information is worth paying for (EVPI and EVSI against cost).")
(litgraph--define-op-command litgraph-settle "settle"
  "Settlement range, bargaining prices and when to settle.")

(defun litgraph-menu--graph-neighborhood ()
  "Draw the neighborhood of a node."
  (interactive)
  (let ((current-prefix-arg '(4)))
    (call-interactively #'litgraph-graph)))

(defun litgraph-menu--has-op (op)
  "Non-nil when OP is available (or availability is not known yet)."
  (litgraph-op-available-p op))

(defun litgraph-menu--description ()
  "Heading of the menu: the current context."
  (format "litgraph  %s" (propertize (litgraph-context-label) 'face 'bold)))

(transient-define-prefix litgraph-menu-transient ()
  "Front door for litgraph."
  [:description litgraph-menu--description
   ["Context"
    ("C" "Matter or package…" litgraph-set-target :transient t)
    ("V" "Parameter…" litgraph-set-param :transient t)
    ("Z" "Clear context" litgraph-clear-context :transient t)]
   ["Analyze"
    ("s" "Solve: value and best line" litgraph-solve)
    ("c" "Chain: outcome odds, expected cost" litgraph-chain)
    ("e" "Explain a decision" litgraph-explain)
    ("m" "Simulate (Monte Carlo)" litgraph-simulate)
    ("t" "Tornado: what drives it" litgraph-tornado)
    ("w" "Sweep a parameter" litgraph-sweep)]
   ["Lines"
    ("p" "Paths to a node/outcome" litgraph-path)
    ("f" "Pareto frontier" litgraph-pareto)
    ("u" "Structure" litgraph-structure)
    ("x" "Compare with a variant" litgraph-compare)]
   ["Newer engine ops"
    ("i" "Posterior" litgraph-posterior
     :inapt-if-not (lambda () (litgraph-menu--has-op "posterior")))
    ("v" "Value of information" litgraph-voi
     :inapt-if-not (lambda () (litgraph-menu--has-op "voi")))
    ("l" "Settlement" litgraph-settle
     :inapt-if-not (lambda () (litgraph-menu--has-op "settle")))
    ("a" "Calibration gaps" litgraph-calibration
     :inapt-if-not (lambda () (litgraph-menu--has-op "calibration")))]]
  [["See"
    ("g" "Graph (Graphviz)" litgraph-graph)
    ("G" "Graph around a node" litgraph-menu--graph-neighborhood)
    ("d" "Deadlines to Org agenda" litgraph-deadlines-to-org)
    ("T" "Timeline of a .case/.lit file" litgraph-timeline)]
   ["Engine"
    ("r" "Run any op…" litgraph-run-op)
    ("k" "Packs" litgraph-list-packs)
    ("L" "Lint" litgraph-lint)
    ("D" "Describe (the manual)" litgraph-describe)
    ("!" "Refresh metadata" litgraph-refresh :transient t)
    ("K" "Cancel running" litgraph-cancel)]])

;;;###autoload
(defun litgraph-menu ()
  "Open the litgraph transient menu."
  (interactive)
  (litgraph-prefetch)
  (litgraph-menu-transient))

(provide 'litgraph-menu)

;;; litgraph-menu.el ends here
