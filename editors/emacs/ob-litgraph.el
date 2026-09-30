;;; ob-litgraph.el --- Org Babel backend for litgraph -*- lexical-binding: t; -*-

;; Copyright (C) 2026 David Awad

;; Author: David Awad <me@davidaw.ad>
;; Maintainer: David Awad <me@davidaw.ad>
;; Keywords: tools, convenience, law, literate programming
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

;; Evaluate litgraph requests in Org source blocks:
;;
;;   #+begin_src litgraph :results table
;;   {"scenario": "cofc-1498-patent-case", "op": {"op": "chain"}}
;;   #+end_src
;;
;; The block body is a request (or just a scenario, with the op given as
;; a header argument).  The result is an Org table.
;;
;; Header arguments:
;;
;;   :op        op name (or an op as JSON), for bodies that are only a scenario
;;   :scenario  library scenario name or file, when the body has none
;;   :packs     comma separated pack ids
;;   :table     regexp choosing which table of the result (default: the first)
;;   :format    table (default), summary, json, or org (all tables as Org text)
;;   :var       numbers set scenario parameters: :var rate=900 stakes=2
;;
;; Enable with (org-babel-do-load-languages 'org-babel-load-languages
;;              '((litgraph . t))).
;; Evaluation is synchronous, as Org Babel requires, but goes through the
;; same asynchronous process layer and stays interruptible with C-g.

;;; Code:

(require 'ob)
(require 'cl-lib)
(require 'seq)
(require 'subr-x)
(require 'litgraph)
(require 'litgraph-view)

(defvar org-babel-default-header-args:litgraph
  '((:results . "replace")
    (:exports . "both"))
  "Default header arguments for litgraph source blocks.")

(defvar org-src-lang-modes)
(add-to-list 'org-src-lang-modes '("litgraph" . js-json))

(defun org-babel-expand-body:litgraph (body params &optional _processed-params)
  "Return BODY, the request JSON; PARAMS are unused."
  (ignore params)
  body)

(defun litgraph-ob--op (value)
  "The op object for the :op header argument VALUE (name or JSON)."
  (cond ((null value) nil)
        ((string-match-p "\\`[[:space:]]*{" value) (litgraph-parse-strict value))
        (t (litgraph-obj "op" (string-trim value)))))

(defun litgraph-ob--apply-vars (request vars)
  "Set the scenario parameters of REQUEST from VARS ((NAME . VALUE) ...)."
  (when vars
    (let ((params (mapcar (lambda (var)
                            (cons (format "%s" (car var))
                                  (let ((value (cdr var)))
                                    (if (stringp value)
                                        (string-to-number value)
                                      value))))
                          vars)))
      (puthash "scenario"
               (litgraph--with-params (gethash "scenario" request) params)
               request)))
  request)

(defun litgraph-ob-request (body params)
  "Build the request for the source block BODY under header arguments PARAMS.
A block is self-contained: the session scenario, packs and parameters
of `litgraph-menu' do not leak into it."
  (let* ((text (string-trim body))
         (doc (if (string-empty-p text) (litgraph-obj)
                (litgraph-parse-strict text)))
         (op (or (litgraph-ob--op (cdr (assq :op params)))
                 (litgraph-obj "op" "solve")))
         (scenario (cdr (assq :scenario params)))
         (packs (cdr (assq :packs params)))
         (request
          (cond
           ((not (hash-table-p doc))
            (user-error "A litgraph block must hold a JSON object"))
           ((gethash "op" doc)
            (let ((copy (litgraph-hash-copy doc)))
              (when (assq :op params) (puthash "op" op copy))
              copy))
           ((gethash "scenario" doc)
            (let ((copy (litgraph-hash-copy doc)))
              (puthash "op" op copy)
              copy))
           ((> (hash-table-count doc) 0)
            (litgraph-obj "scenario" doc "op" op))
           (scenario (litgraph-obj "scenario" scenario "op" op))
           (t (user-error "Empty litgraph block: give a request or :scenario")))))
    (when (and scenario (not (gethash "scenario" request)))
      (puthash "scenario" scenario request))
    (when packs
      (puthash "packs" (litgraph-vec (split-string packs "[ ,]+" t)) request))
    (litgraph-ob--apply-vars request (org-babel--get-vars params))))

(defun litgraph-ob--pick-table (tables pattern)
  "The table in TABLES whose title matches PATTERN; the first when PATTERN is nil."
  (if pattern
      (or (seq-find (lambda (table)
                      (string-match-p pattern (or (plist-get table :title) "")))
                    tables)
          (user-error "No table matching `%s'; have: %s" pattern
                      (mapconcat (lambda (table) (or (plist-get table :title) "?"))
                                 tables ", ")))
    (car tables)))

(defun litgraph-ob--summary-table (response)
  "A two-column Org table of RESPONSE's summary."
  (append '(("Key" "Value") hline)
          (mapcar (lambda (pair) (list (car pair) (cdr pair)))
                  (litgraph-view-response-summary response))))

(defun litgraph-ob--format (response raw params)
  "Format RESPONSE (with RAW JSON text) per the :format and :table in PARAMS."
  (let ((format (or (cdr (assq :format params)) "table"))
        (pattern (cdr (assq :table params))))
    (pcase format
      ("json" (with-temp-buffer
                (insert raw)
                (ignore-errors (json-pretty-print-buffer))
                (string-trim-right (buffer-string))))
      ("summary" (litgraph-ob--summary-table response))
      ("org" (mapconcat
              (lambda (table) (litgraph-view-table-to-org table))
              (litgraph-view-response-tables response) "\n"))
      (_ (if-let* ((tables (litgraph-view-response-tables response)))
             (litgraph-view-table-org-rows
              (litgraph-ob--pick-table tables pattern))
           (litgraph-ob--summary-table response))))))

(defun litgraph-ob--report-warnings (response)
  "Tell the user which kinds of warnings RESPONSE carries."
  (when-let* ((warnings (litgraph-get response 'warnings)))
    (message "litgraph: %d warning kind(s): %s (numbers rest on fallbacks)"
             (length warnings)
             (string-join (delete-dups
                           (mapcar (lambda (w) (litgraph-get w 'code)) warnings))
                          ", "))))

(defun org-babel-execute:litgraph (body params)
  "Evaluate the litgraph request BODY under the header arguments PARAMS."
  (let* ((request (litgraph-ob-request body params))
         (response (litgraph-request-sync request)))
    (litgraph-ob--report-warnings response)
    (litgraph-ob--format response litgraph--last-raw params)))

(defun org-babel-prep-session:litgraph (_session _params)
  "Litgraph has no sessions."
  (error "Litgraph does not support sessions"))

(provide 'ob-litgraph)

;;; ob-litgraph.el ends here
