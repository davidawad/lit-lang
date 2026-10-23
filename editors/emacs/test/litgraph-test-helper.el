;;; litgraph-test-helper.el --- Shared helpers for the litgraph tests -*- lexical-binding: t; -*-

;; SPDX-License-Identifier: GPL-3.0-or-later

;;; Commentary:

;; The unit tests never start the real binary.  They replace
;; `litgraph-process-function' with `litgraph-test-stub', which answers from
;; recorded real litgraph output in test/fixtures/.

;;; Code:

(require 'ert)
(require 'cl-lib)
(require 'litgraph)

(defconst litgraph-test-dir
  (file-name-directory (or load-file-name buffer-file-name))
  "Directory of the test files.")

(defun litgraph-test-fixture (name)
  "Contents of the fixture NAME (without extension) as a string."
  (with-temp-buffer
    (insert-file-contents
     (expand-file-name (format "fixtures/%s.json" name) litgraph-test-dir))
    (buffer-string)))

(defun litgraph-test-fixture-response (name)
  "Fixture NAME parsed like a live response."
  (litgraph-parse (litgraph-test-fixture name)))

(defvar litgraph-test-calls nil
  "The (ARGS . INPUT) of every stubbed process call, newest first.")

(defvar litgraph-test-op-fixtures
  '(("describe" . "describe") ("solve" . "solve") ("chain" . "chain")
    ("graph" . "graph-cofc") ("tornado" . "tornado") ("simulate" . "simulate")
    ("path" . "path") ("pareto" . "pareto") ("sweep" . "sweep")
    ("explain" . "explain") ("compare" . "compare") ("batch" . "batch")
    ("structure" . "structure") ("lint" . "lint") ("packs" . "packs")
    ("deadlines" . "deadlines-node") ("calibration" . "calibration")
    ("metric" . "metric"))
  "Op name to the fixture answering it.")

(defun litgraph-test--request-op (input)
  "The op name in the request JSON INPUT."
  (let* ((table (litgraph-parse-strict input))
         (op (gethash "op" table)))
    (if (hash-table-p op) (gethash "op" op) op)))

(defvar litgraph-test-validate-fixture "validate-unknown-field"
  "Fixture returned to `litgraph validate' calls.")

(defvar litgraph-test-timeline-fixture "timeline-ttab-opposition"
  "Fixture returned to `litgraph timeline' calls.")

(defun litgraph-test-stub (args input callback)
  "Stand-in for `litgraph--spawn': answer ARGS/INPUT from fixtures via CALLBACK."
  (push (cons args input) litgraph-test-calls)
  (pcase args
    (`("describe") (funcall callback 0 (litgraph-test-fixture "describe") ""))
    (`("schema" "request")
     (funcall callback 0 (litgraph-test-fixture "schema-request") ""))
    (`("q" "-")
     (let* ((op (litgraph-test--request-op input))
            (fixture (cdr (assoc op litgraph-test-op-fixtures))))
       (if fixture
           (funcall callback 0 (litgraph-test-fixture fixture) "")
         (funcall callback 2 (litgraph-test-fixture "error") ""))))
    (`("timeline" . ,_)
     (funcall callback 0 (litgraph-test-fixture litgraph-test-timeline-fixture) ""))
    ((and `("validate" . ,_) (guard t))
     (funcall callback 2 (litgraph-test-fixture litgraph-test-validate-fixture) ""))
    (_ (funcall callback 1 "" "unexpected call")))
  nil)

(defmacro litgraph-test-with-stub (&rest body)
  "Run BODY with the process layer stubbed and a fresh cache and session."
  (declare (indent 0))
  `(let ((litgraph-process-function #'litgraph-test-stub)
         (litgraph-test-calls nil)
         (litgraph--cache (make-hash-table :test #'equal))
         (litgraph-scenario nil) (litgraph-packs nil) (litgraph-params nil)
         (litgraph-default-scenario nil) (litgraph-default-packs nil)
         (litgraph-last-error nil))
     ,@body))

(defun litgraph-test-real-binary ()
  "Path of the real litgraph binary, or nil.
Set LITGRAPH_SKIP_INTEGRATION=1 to pretend there is none."
  (unless (getenv "LITGRAPH_SKIP_INTEGRATION")
    (or (executable-find "litgraph")
        (let ((path "/opt/homebrew/bin/litgraph"))
          (and (file-executable-p path) path)))))

(provide 'litgraph-test-helper)

;;; litgraph-test-helper.el ends here
