;;; litgraph-test-helper.el --- Shared helpers for the litgraph tests -*- lexical-binding: t; -*-

;; SPDX-License-Identifier: GPL-3.0-or-later

;;; Commentary:

;; The unit tests never start the real binary.  They replace
;; `litgraph-process-function' with `litgraph-test-stub', which answers from
;; recorded real engine output in test/fixtures/.  The integration tests
;; run the `litigate' binary on PATH.

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

(defun litgraph-test-skip-or-fail (reason)
  "Skip with REASON, unless LITGRAPH_IT_REQUIRE requests a hard failure."
  (if (equal (getenv "LITGRAPH_IT_REQUIRE") "1")
      (error "%s (LITGRAPH_IT_REQUIRE=1)" reason)
    (ert-skip reason)))

(defvar litgraph-test-calls nil
  "The (ARGS . INPUT) of every stubbed process call, newest first.")

(defvar litgraph-test-commands nil
  "The (EXECUTABLE . PREFIX) of every stubbed process call, newest first.
PREFIX is `litgraph-engine-arguments' at the time of the call.")

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
  "Fixture returned to `litigate engine validate' calls.")

(defvar litgraph-test-timeline-fixture "timeline-ttab-opposition"
  "Fixture returned to `litigate timeline' calls.")

(defun litgraph-test-stub (args input callback)
  "Stand-in for `litgraph--spawn': answer ARGS/INPUT from fixtures via CALLBACK."
  (push (cons args input) litgraph-test-calls)
  (push (cons litgraph-executable litgraph-engine-arguments)
        litgraph-test-commands)
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
    (`("timeline" . ,args)
     ;; Model clap's timeline parser: --format takes json|human, and only
     ;; the date, layer, and world options are accepted.  In particular,
     ;; obsolete or unknown flags must fail instead of returning a fixture.
     (let ((rest args) (valid t) (file nil))
       (while (and rest valid)
         (let ((arg (pop rest)))
           (cond
            ((equal arg "--format")
             (if (cl-member (pop rest) '("json" "human") :test #'equal) nil
               (setq valid nil)))
            ((member arg '("--as-of" "--known-at"))
             (if (and rest (not (string-prefix-p "--" (car rest))))
                 (pop rest) (setq valid nil)))
            ((member arg '("--layers" "--world"))
             (if rest (pop rest) (setq valid nil)))
            ((string-prefix-p "--" arg) (setq valid nil))
            ((null file) (setq file arg))
            (t (setq valid nil)))))
       (if (and valid file)
           (funcall callback 0 (litgraph-test-fixture litgraph-test-timeline-fixture) "")
         (funcall callback 2 "" "error: unexpected argument or invalid timeline arguments"))))
    ((and `("validate" . ,_) (guard t))
     (funcall callback 2 (litgraph-test-fixture litgraph-test-validate-fixture) ""))
    (_ (funcall callback 1 "" "unexpected call")))
  nil)

(defmacro litgraph-test-with-stub (&rest body)
  "Run BODY with the process layer stubbed and a fresh cache and session."
  (declare (indent 0))
  `(let ((litgraph-process-function #'litgraph-test-stub)
         (litgraph-test-calls nil)
         (litgraph-test-commands nil)
         (litgraph--cache (make-hash-table :test #'equal))
         (litgraph-scenario nil) (litgraph-packs nil) (litgraph-params nil)
         (litgraph-default-scenario nil) (litgraph-default-packs nil)
         (litgraph-last-error nil))
     ,@body))

(defun litgraph-test-real-binary ()
  "Path of the real `litigate' binary, or nil.
Set LITGRAPH_SKIP_INTEGRATION=1 to pretend there is none."
  (unless (getenv "LITGRAPH_SKIP_INTEGRATION")
    (or (executable-find "litigate")
        (let ((path "/opt/homebrew/bin/litigate"))
          (and (file-executable-p path) path)))))

(provide 'litgraph-test-helper)

;;; litgraph-test-helper.el ends here
