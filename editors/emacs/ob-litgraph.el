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

;; Run litgraph ops on a matter from Org source blocks:
;;
;;   #+begin_src litgraph :target cofc-1498-patent-case
;;   {"op": "deadlines", "trigger": "2026-10-01"}
;;   #+end_src
;;
;; The block body is the op: an op object as JSON, or just its name
;; (empty means `solve').  It runs as `litigate run TARGET --op OP'.
;; The result is an Org table.
;;
;; Header arguments:
;;
;;   :target    the `.case' matter, `.lit' file or package directory
;;              (required); a bare name is `cases/NAME.case' (else
;;              `lit/NAME') in the project
;;   :op        op name (or an op as JSON), replacing the body's
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
  "Return BODY, the op; PARAMS are unused."
  (ignore params)
  body)

(defun litgraph-ob--op (value)
  "The op object for VALUE, an op name or an op object as JSON."
  (let ((value (and value (string-trim (format "%s" value)))))
    (cond ((null value) nil)
          ((string-prefix-p "{" value) (litgraph-parse-strict value))
          ((string-match-p "\\`[a-z][a-z_-]*\\'" value) (litgraph-obj "op" value))
          (t (user-error "Not an op name or op object: %s" value)))))

(defun litgraph-ob--params (vars)
  "Scenario parameters ((NAME . NUMBER) ...) from the Babel VARS."
  (mapcar (lambda (var)
            (cons (format "%s" (car var))
                  (let ((value (cdr var)))
                    (if (stringp value) (string-to-number value) value))))
          vars))

(defun litgraph-ob-request (body params)
  "Build the request for the source block BODY under header arguments PARAMS.
A block is self-contained: the session target and parameters of
`litgraph-menu' do not leak into it."
  (let* ((target (cdr (assq :target params)))
         (op (or (litgraph-ob--op (cdr (assq :op params)))
                 (let ((text (string-trim body)))
                   (if (string-empty-p text)
                       (litgraph-obj "op" "solve")
                     (litgraph-ob--op text))))))
    (unless (and (hash-table-p op) (stringp (gethash "op" op)))
      (user-error "A litgraph block holds an op: a name or a JSON object with \"op\""))
    (unless target
      (user-error "A litgraph block needs :target (a .case matter or package)"))
    (litgraph-obj "target" (format "%s" target)
                  "op" op
                  "params" (litgraph--params-object
                            (litgraph-ob--params (org-babel--get-vars params))))))

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
  "Run the litgraph op BODY under the header arguments PARAMS."
  (let* ((request (litgraph-ob-request body params))
         (response (litgraph-request-sync request)))
    (litgraph-ob--report-warnings response)
    (litgraph-ob--format response litgraph--last-raw params)))

(defun org-babel-prep-session:litgraph (_session _params)
  "Litgraph has no sessions."
  (error "Litgraph does not support sessions"))

(provide 'ob-litgraph)

;;; ob-litgraph.el ends here
