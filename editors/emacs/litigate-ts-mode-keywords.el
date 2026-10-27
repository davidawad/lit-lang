;;; litigate-ts-mode-keywords.el --- Keyword lists of litigate-ts-mode  -*- lexical-binding: t; -*-

;; Copyright (C) 2026 David Awad

;; Author: David Awad
;; URL: https://github.com/davidawad/litgraph
;; SPDX-License-Identifier: GPL-3.0-or-later

;; This file is not part of GNU Emacs.

;;; Commentary:

;; Generated from crates/litigate-syntax/keywords.txt by `just
;; editor-queries'.  Do not edit; edit keywords.txt and rerun.

;;; Code:

(defconst litigate-ts-mode--declaration-keywords
  '("module" "import" "package" "start" "group" "type" "enum" "state" "decision"
    "chance" "terminal" "rule" "deadline" "counting" "calendar" "transition"
    "authority" "default" "exception" "test" "link" "instance" "event" "case"
    "uses" "as_of" "known_at" "summary" "notes" "source" "perspective" "parties"
    "events" "facts" "stakes" "beliefs" "observe" "prior" "analysis" "edition"
    "limit" "grant" "recharacterize" "invalidate" "governing_law")
  "Keywords that open a declaration.")

(defconst litigate-ts-mode--clause-keywords
  '("forum" "duty" "power" "liability" "amends" "allow_hours" "end_of_month"
    "below" "count" "by" "docket" "from" "after" "before" "counted_by" "outcome"
    "payoff" "payoff_by_flag" "fee_shift" "info" "p" "effort" "cost" "within"
    "duration" "min" "mode" "max" "sets" "clears" "requires" "forbids" "tags"
    "attrs" "note" "valence" "basis" "court_listener_url" "action_id"
    "extendable" "extension_authority" "replaces" "pack" "remove_edges"
    "probabilities" "roles" "payoff_transform" "stake" "patch" "effective"
    "exclude_trigger_day" "include_last_day" "roll_forward_on"
    "service_extension" "holidays" "anchor" "short_period" "observed"
    "subject_to" "notwithstanding" "supersedes" "revives" "interprets"
    "delegable_to" "override" "rebuts" "undercuts" "priority" "url" "bearer"
    "trigger" "extend" "waive" "after_expiry" "never" "removes" "elects"
    "extends" "waives" "to" "where" "designated" "interpretation" "formerly"
    "forums" "forward_holidays" "declared_holidays" "quote" "words" "pages"
    "lines" "certificate" "excludes" "ecf" "for" "paper" "delegable" "op"
    "unlock" "characterization" "retracted" "unknown_before" "commenced" "spawn"
    "joins" "anytime" "lock" "against" "Rule" "on" "given" "expect")
  "Clause, field and test keywords.")

(defconst litigate-ts-mode--conditional-keywords
  '("match" "when" "unless" "otherwise")
  "Conditional keywords.")

(defconst litigate-ts-mode--directive-keywords
  '("cite")
  "Directive keywords.")

(provide 'litigate-ts-mode-keywords)
;;; litigate-ts-mode-keywords.el ends here
