;;; litgraph-test-helper.el --- Shared helpers for the litgraph tests -*- lexical-binding: t; -*-

;; SPDX-License-Identifier: GPL-3.0-or-later

;;; Commentary:

;; The unit tests never start the real binary.  They replace
;; `litgraph-process-function' with `litgraph-test-stub', which answers from
;; recorded real `litigate run' output in test/fixtures/ (`make
;; fixtures' regenerates it).  The integration tests run the real
;; binary: LITIGATE_BIN, else the `litigate' on PATH.

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
  "Fixture NAME parsed like a live response.
An engine fixture is the envelope `litigate run' prints: this is the
engine response inside it.  Any other fixture is parsed whole."
  (let ((text (litgraph-test-fixture name)))
    (or (car (litgraph--engine-response text))
        (litgraph-parse text))))

(defun litgraph-test-skip-or-fail (reason)
  "Skip with REASON, unless LITGRAPH_IT_REQUIRE requests a hard failure."
  (if (equal (getenv "LITGRAPH_IT_REQUIRE") "1")
      (error "%s (LITGRAPH_IT_REQUIRE=1)" reason)
    (ert-skip reason)))

(defvar litgraph-test-calls nil
  "The (ARGS . INPUT) of every stubbed process call, newest first.")

(defvar litgraph-test-commands nil
  "The executable of every stubbed process call, newest first.")

(defvar litgraph-test-op-fixtures
  '(("describe" . "describe") ("solve" . "solve") ("chain" . "chain")
    ("graph" . "graph-cofc") ("tornado" . "tornado") ("simulate" . "simulate")
    ("path" . "path") ("pareto" . "pareto") ("sweep" . "sweep")
    ("explain" . "explain") ("compare" . "compare") ("batch" . "batch")
    ("structure" . "structure") ("lint" . "lint") ("packs" . "packs")
    ("deadlines" . "deadlines-node") ("calibration" . "calibration")
    ("metric" . "metric"))
  "Op name to the fixture answering it; any other op gets `error'.")

(defun litgraph-test-call-op (call)
  "The op object (a hash table) of the stubbed `litigate run' CALL."
  (let ((args (car call)))
    (when (equal (car args) "run")
      (litgraph-parse-strict (cadr (member "--op" args))))))

(defun litgraph-test-call-op-name (call)
  "The op name of the stubbed `litigate run' CALL, or nil."
  (when-let* ((op (litgraph-test-call-op call))) (gethash "op" op)))

(defun litgraph-test-run-calls ()
  "The stubbed `litigate run' calls, oldest first."
  (seq-filter (lambda (call) (equal (car (car call)) "run"))
              (reverse litgraph-test-calls)))

(defvar litgraph-test-timeline-fixture "timeline-ttab-opposition"
  "Fixture returned to `litigate timeline' calls.")

(defun litgraph-test--run-args-valid-p (args)
  "Non-nil when ARGS (after `run') parse as `litigate run' would.
That is a target, then `--op JSON', `--set NAME=NUMBER' and `--format
json' options."
  (let ((rest args) (valid t) (target nil))
    (while (and rest valid)
      (let ((arg (pop rest)))
        (cond
         ((equal arg "--op")
          (setq valid (and rest (ignore-errors (litgraph-parse-strict (pop rest))))))
         ((equal arg "--set")
          (setq valid (and rest (string-match-p "\\`[a-z_]+=-?[0-9.e]+\\'" (pop rest)))))
         ((equal arg "--format")
          (setq valid (equal (pop rest) "json")))
         ((string-prefix-p "--" arg) (setq valid nil))
         ((null target) (setq target arg))
         (t (setq valid nil)))))
    (and valid target)))

(defun litgraph-test-stub (args input callback)
  "Stand-in for `litgraph--spawn': answer ARGS/INPUT from fixtures via CALLBACK."
  (push (cons args input) litgraph-test-calls)
  (push litgraph-executable litgraph-test-commands)
  (pcase args
    (`("run" . ,rest)
     (if (not (litgraph-test--run-args-valid-p rest))
         (funcall callback 1 "" "error: invalid `litigate run' arguments")
       (let* ((op (litgraph-test-call-op (cons args input)))
              (fixture (cdr (assoc (gethash "op" op) litgraph-test-op-fixtures))))
         (if fixture
             (funcall callback 0 (litgraph-test-fixture fixture) "")
           (funcall callback 2 (litgraph-test-fixture "error") "")))))
    (`("--version") (funcall callback 0 "litigate 0.6.0 (commit x, clean)\n" ""))
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
    (_ (funcall callback 1 "" "unexpected call")))
  nil)

(defmacro litgraph-test-with-stub (&rest body)
  "Run BODY with the process layer stubbed and a fresh cache and session."
  (declare (indent 0))
  `(let ((litgraph-process-function #'litgraph-test-stub)
         (litgraph-test-calls nil)
         (litgraph-test-commands nil)
         (litgraph--cache (make-hash-table :test #'equal))
         (litgraph-target nil) (litgraph-params nil)
         (litgraph-default-target nil)
         (litgraph-last-error nil))
     ,@body))

(defun litgraph-test-real-binary ()
  "Path of the real `litigate' binary, or nil.
LITIGATE_BIN names it; else it is the `litigate' on PATH.  Set
LITGRAPH_SKIP_INTEGRATION=1 to pretend there is none."
  (unless (getenv "LITGRAPH_SKIP_INTEGRATION")
    (or (let ((bin (getenv "LITIGATE_BIN")))
          (and bin (not (string-empty-p bin)) (file-executable-p bin) bin))
        (executable-find "litigate")
        (let ((path "/opt/homebrew/bin/litigate"))
          (and (file-executable-p path) path)))))

(provide 'litgraph-test-helper)

;;; litgraph-test-helper.el ends here
