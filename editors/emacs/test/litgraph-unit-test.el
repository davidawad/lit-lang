;;; litgraph-unit-test.el --- Unit tests (process layer stubbed) -*- lexical-binding: t; -*-

;; SPDX-License-Identifier: GPL-3.0-or-later

;;; Code:

(require 'litgraph-test-helper)
(require 'litgraph)
(require 'litgraph-view)
(require 'litgraph-graph)
(require 'litgraph-org)
(require 'litgraph-menu)
(require 'ob-litgraph)

(ert-deftest litgraph-test-skip-or-fail-respects-require-switch ()
  (let ((process-environment (copy-sequence process-environment)))
    (setenv "LITGRAPH_IT_REQUIRE" "1")
    (should-error (litgraph-test-skip-or-fail "tool missing") :type 'error)
    (setenv "LITGRAPH_IT_REQUIRE" nil)
    (should-error (litgraph-test-skip-or-fail "tool missing")
                  :type 'ert-test-skipped)))

;;;; Core: JSON, requests, process layer

(ert-deftest litgraph-get-walks-paths ()
  (let ((r (litgraph-parse "{\"a\": {\"b\": [1, {\"c\": 7}]}}")))
    (should (equal (litgraph-get r 'a 'b 1 'c) 7))
    (should (equal (litgraph-get r "a" 'b 0) 1))
    (should-not (litgraph-get r 'missing 'deeper))))

(ert-deftest litgraph-obj-drops-nil ()
  (let ((o (litgraph-obj "a" 1 "b" nil "c" "x")))
    (should (= (hash-table-count o) 2))
    (should (equal (litgraph-serialize (litgraph-obj "op" "solve"))
                   "{\"op\":\"solve\"}"))))

(ert-deftest litgraph-build-request-shape ()
  (litgraph-test-with-stub
    (let ((r (litgraph-build-request "solve" "cofc-1498-patent-case")))
      (should (equal (gethash "target" r) "cofc-1498-patent-case"))
      (should (equal (litgraph-serialize (gethash "op" r)) "{\"op\":\"solve\"}"))
      (should-not (gethash "params" r)))
    (let* ((op (litgraph-obj "op" "sweep" "param" "rate"))
           (r (litgraph-build-request op "m" '(("rate" . 900)))))
      (should (eq (gethash "op" r) op))
      (should (= 900 (gethash "rate" (gethash "params" r)))))))

(ert-deftest litgraph-build-request-uses-session ()
  (litgraph-test-with-stub
    (setq litgraph-target "frcp-civil-case"
          litgraph-params '(("rate" . 1)))
    (let ((r (litgraph-build-request "solve")))
      (should (equal (gethash "target" r) "frcp-civil-case"))
      (should (= 1 (gethash "rate" (gethash "params" r)))))
    (setq litgraph-target nil)
    (let ((litgraph-default-target "cofc-bid-protest"))
      (should (equal (gethash "target" (litgraph-build-request "solve"))
                     "cofc-bid-protest")))))

(defmacro litgraph-test-with-project (root &rest body)
  "Run BODY with ROOT bound to a fresh directory holding lit/ and cases/."
  (declare (indent 1))
  `(let ((,root (file-name-as-directory (make-temp-file "lg-project" t))))
     (unwind-protect
         (progn
           (make-directory (expand-file-name "lit/pack" ,root) t)
           (make-directory (expand-file-name "cases/sub" ,root) t)
           ,@body)
       (delete-directory ,root t))))

(ert-deftest litgraph-request-args-run-the-target ()
  (litgraph-test-with-project root
    (write-region "" nil (expand-file-name "cases/m.case" root))
    (let ((litgraph-project-directory root))
      (should (equal (litgraph-request-args
                      (litgraph-build-request
                       (litgraph-obj "op" "explain" "node" "a::b") "m"
                       '(("stakes" . 2) ("rate" . 900.0))))
                     (list "run" (expand-file-name "cases/m.case" root)
                           "--op" "{\"op\":\"explain\",\"node\":\"a::b\"}"
                           "--set" "rate=900" "--set" "stakes=2"
                           "--format" "json")))
      (should (equal (litgraph-request-args (litgraph-build-request "solve" "pack"))
                     (list "run" (expand-file-name "lit/pack" root)
                           "--op" "{\"op\":\"solve\"}" "--format" "json"))))
    (should-error (litgraph-request-args (litgraph-obj "op" (litgraph-obj "op" "solve")))
                  :type 'user-error)))

(ert-deftest litgraph-resolve-target-paths-and-names ()
  (litgraph-test-with-project root
    (let ((case-file (expand-file-name "cases/sub/x.case" root)))
      (write-region "" nil case-file)
      (let ((litgraph-project-directory root)
            (default-directory (expand-file-name "lit/" root)))
        ;; A path relative to `default-directory', then to the project root.
        (should (equal (litgraph-resolve-target "pack") (expand-file-name "lit/pack" root)))
        (should (equal (litgraph-resolve-target "cases/sub/x.case") case-file))
        (should (equal (litgraph-resolve-target case-file) case-file))
        ;; Unknown names reach the binary as given, for it to report.
        (should (equal (litgraph-resolve-target "nope") "nope"))
        (should (equal (litgraph-target-candidates)
                       '(("cases/sub/x.case" . "matter") ("lit/pack" . "package"))))))))

(ert-deftest litgraph-target-label-names-matters-and-packages ()
  (should (equal (litgraph-target-label "/r/cases/acme.case") "acme"))
  (should (equal (litgraph-target-label "/r/lit/cofc/") "cofc"))
  (should (equal (litgraph-target-label "frcp-civil-case") "frcp-civil-case"))
  (let ((litgraph-target "/r/cases/acme.case") (litgraph-params '(("rate" . 9))))
    (should (equal (litgraph-context-label) "acme  rate=9"))))

(ert-deftest litgraph-request-success-and-raw ()
  (litgraph-test-with-stub
    (let (got)
      (litgraph-request (litgraph-build-request "solve" "s")
                        (lambda (response) (setq got response)))
      (should (litgraph-get got 'ok))
      (should (equal (litgraph-get got 'op) "solve"))
      (should (numberp (litgraph-get got 'result 'value)))
      ;; The raw text is the engine response, not the envelope around it.
      (should (string-match-p "\"best_line\"" litgraph--last-raw))
      (should (string-prefix-p "{\"ok\":true" litgraph--last-raw))
      (should-not (string-match-p "\"op\":\"run\"" litgraph--last-raw))
      (should (equal (car (car litgraph-test-calls))
                     (litgraph-request-args (litgraph-build-request "solve" "s")))))))

(ert-deftest litgraph-request-engine-error-reaches-errback ()
  (litgraph-test-with-stub
    (let (message)
      (litgraph-request (litgraph-build-request "nonsense" "s")
                        #'ignore
                        (lambda (m &optional _r) (setq message m)))
      (should (string-match-p "not-found" message))
      (should (string-match-p "list ids with" message)))))

(ert-deftest litgraph-request-default-errback-records ()
  (litgraph-test-with-stub
    (litgraph-request (litgraph-build-request "nonsense" "s") #'ignore)
    (should (string-match-p "not-found" litgraph-last-error))))

(ert-deftest litgraph-request-process-failure ()
  (let ((litgraph-process-function
         (lambda (_a _i cb) (funcall cb 1 "" "boom")))
        (litgraph-last-error nil))
    (litgraph-request (litgraph-build-request "solve" "s") #'ignore)
    (should (string-match-p "exit 1: boom" litgraph-last-error)))
  (let ((litgraph-process-function (lambda (_a _i cb) (funcall cb 'timeout "" "")))
        (litgraph-last-error nil))
    (litgraph-request (litgraph-build-request "solve" "s") #'ignore)
    (should (string-match-p "timed out" litgraph-last-error))))

(ert-deftest litgraph-request-sync-returns-and-signals ()
  (litgraph-test-with-stub
    (should (litgraph-get (litgraph-request-sync
                           (litgraph-build-request "chain" "s"))
                          'ok))
    (should-error (litgraph-request-sync (litgraph-build-request "nonsense" "s"))
                  :type 'error)))

(ert-deftest litgraph-compile-error-reports-its-diagnostic ()
  ;; A matter that does not compile never reaches the engine: the
  ;; envelope is not ok and carries only diagnostics.
  (let ((decoded (litgraph--decode 2 (litgraph-test-fixture "compile-error") "")))
    (should-not (car decoded))
    (should (string-match-p "\\`L0001: case is missing" (cadr decoded)))
    (should (string-match-p "(broken.case:1)" (cadr decoded)))
    (should-not (cddr decoded))))

(ert-deftest litgraph-engine-calls-go-through-litigate-run ()
  (should (equal (default-value 'litgraph-executable) "litigate"))
  (should-not (boundp 'litgraph-engine-arguments))
  (litgraph-test-with-stub
    (litgraph-request-sync (litgraph-build-request "chain" "s"))
    (should (equal (car litgraph-test-commands) "litigate"))
    (should (equal (seq-take (car (car litgraph-test-calls)) 4)
                   '("run" "s" "--op" "{\"op\":\"chain\"}")))))

(ert-deftest litgraph-project-root-is-nearest-lit-directory ()
  (litgraph-test-with-project root
    (let ((litgraph-project-directory nil))
      (let ((default-directory (expand-file-name "cases/sub/" root)))
        (should (file-equal-p (litgraph-project-root) root)))
      (let ((default-directory (expand-file-name "lit/pack/" root)))
        ;; lit/pack has no lit/ of its own; lit/ sits one level higher.
        (should (file-equal-p (litgraph-project-root) root)))
      (let ((default-directory (expand-file-name "no-such-dir/" root)))
        (should-not (litgraph-project-root))))
    (let ((litgraph-project-directory (expand-file-name "cases" root))
          (default-directory root))
      (should (equal (litgraph-project-root)
                     (file-name-as-directory (expand-file-name "cases" root)))))))

(ert-deftest litgraph-spawn-runs-litigate-in-project-root ()
  ;; A stand-in executable that echoes its arguments and directory.
  (litgraph-test-with-project root
    (let* ((script (expand-file-name "fake-litigate" root))
           (litgraph-executable script)
           (litgraph-project-directory nil)
           (litgraph-timeout 30)
           (default-directory (expand-file-name "cases/sub/" root))
           (result nil))
      (with-temp-file script
        (insert "#!/bin/sh\n"
                "printf '{\"ok\":true,\"result\":{\"args\":\"%s\",\"pwd\":\"%s\"}}' \"$*\" \"$(pwd -P)\"\n"))
      (set-file-modes script #o755)
      (litgraph--spawn '("run" "m.case" "--op" "solve") nil
                       (lambda (exit stdout _stderr)
                         (setq result (cons exit (litgraph-parse stdout)))))
      (let ((deadline (+ (float-time) 30)))
        (while (and (not result) (< (float-time) deadline))
          (accept-process-output nil 0.05)))
      (should (equal (car result) 0))
      (should (equal (litgraph-get (cdr result) 'result 'args) "run m.case --op solve"))
      (should (file-equal-p (litgraph-get (cdr result) 'result 'pwd) root)))))

(ert-deftest litgraph-cache-key-names-the-engine ()
  (litgraph-test-with-project root
    (let ((litgraph-project-directory nil))
      (should-not (equal (let ((default-directory root))
                           (litgraph--cache-key 'describe))
                         (let ((default-directory temporary-file-directory))
                           (litgraph--cache-key 'describe)))))))

(ert-deftest litgraph-executable-missing-is-user-error ()
  (let ((litgraph-executable "definitely-not-a-real-binary-xyz")
        (litgraph-fallback-directories nil))
    (should-error (litgraph-executable-path) :type 'user-error)))

;;;; Metadata and completion

(ert-deftest litgraph-metadata-comes-from-describe ()
  (litgraph-test-with-stub
    (setq litgraph-target "cofc-1498-patent-case")
    (should (member "solve" (litgraph-op-names)))     ; fallback list
    (litgraph-with-describe #'ignore)
    (should (equal (litgraph-test-call-op-name (car litgraph-test-calls)) "describe"))
    (should (member "deadlines" (litgraph-op-names)))
    (should (member "settle" (litgraph-op-names)))
    (should (assoc "cofc" (litgraph-pack-candidates)))
    (should (assoc "rate" (litgraph-param-candidates)))
    (should (assoc "dollars" (litgraph-metric-candidates)))
    (should (litgraph-op-available-p "solve"))
    (should-not (litgraph-op-available-p "frobnicate"))))

(ert-deftest litgraph-describe-is-cached-per-target ()
  (litgraph-test-with-stub
    (setq litgraph-target "a")
    (litgraph-with-describe #'ignore)
    (litgraph-with-describe #'ignore)
    (should (= 1 (length litgraph-test-calls)))
    (setq litgraph-target "b")
    (should-not (litgraph--cached-describe))
    (litgraph-with-describe #'ignore)
    (should (= 2 (length litgraph-test-calls)))))

(ert-deftest litgraph-metadata-needs-a-target ()
  (litgraph-test-with-stub
    (let (message)
      (litgraph-with-describe #'ignore (lambda (m &optional _r) (setq message m)))
      (should (string-match-p "no matter selected" message))
      (should-not litgraph-test-calls)
      (should-error (litgraph-run-op "solve") :type 'user-error))))

(ert-deftest litgraph-graph-candidates ()
  (litgraph-test-with-stub
    (setq litgraph-target "cofc-1498-patent-case")
    (let (graph)
      (litgraph-with-graph (lambda (g) (setq graph g)))
      (should (assoc "cofc::claim-accrues" (litgraph-node-candidates graph)))
      (should (assoc "cofc::e-accrual-to-patentfork"
                     (litgraph-edge-candidates graph)))
      (let ((targets (litgraph-terminal-candidates graph)))
        (should (assoc "terminals" targets))
        (should (assoc "cofc::judgment-for-plaintiff" targets))
        (should (assoc "tag:win" targets))))))

(ert-deftest litgraph-completion-table-annotates ()
  (let* ((table (litgraph--completion-table '(("a" . "note a") "b") 'litgraph))
         (metadata (funcall table "" nil 'metadata))
         (annotate (alist-get 'annotation-function (cdr metadata))))
    (should (equal (all-completions "" table) '("a" "b")))
    (should (string-match-p "note a" (funcall annotate "a")))
    (should-not (funcall annotate "b"))))

;;;; Op arguments

(ert-deftest litgraph-op-arg-specs-from-describe ()
  (let* ((describe (litgraph-get (litgraph-test-fixture-response "describe") 'result))
         (spec (lambda (op name)
                 (seq-find (lambda (s) (equal (plist-get s :name) name))
                           (litgraph-op-arg-specs describe op)))))
    (should (plist-get (funcall spec "deadlines" "trigger") :required))
    (should-not (plist-get (funcall spec "deadlines" "reachable") :required))
    (should-not (plist-get (funcall spec "sweep" "lo") :required))
    (should (plist-get (funcall spec "sweep" "param") :required))
    (let ((what (funcall spec "structure" "what")))
      (should (eq 'enum (plist-get what :type)))
      (should (member "mincut" (plist-get what :enum))))
    (should-not (litgraph-op-arg-specs describe "describe"))
    (should-not (litgraph-op-arg-specs describe "no-such-op"))))

(ert-deftest litgraph-coerce-values ()
  (should (= 12 (litgraph-coerce-value 'integer " 12 ")))
  (should (= 0.5 (litgraph-coerce-value 'number "0.5")))
  (should-error (litgraph-coerce-value 'number "abc") :type 'user-error)
  (should (eq t (litgraph-coerce-value 'boolean "yes")))
  (should (eq :false (litgraph-coerce-value 'boolean "no")))
  (should (equal (litgraph-coerce-value 'strings "a, b c") ["a" "b" "c"]))
  (should (hash-table-p (litgraph-coerce-value 'json "{\"a\":1}")))
  (should (= 3 (litgraph-coerce-value 'guess "3")))
  (should (equal "x" (litgraph-coerce-value 'guess "x"))))

(ert-deftest litgraph-read-op-arguments-prompts-required ()
  (litgraph-test-with-stub
    (let ((describe (litgraph-get (litgraph-test-fixture-response "describe") 'result))
          (answers '("2026-01-05")))
      (cl-letf (((symbol-function 'org-read-date) (lambda (&rest _) (pop answers)))
                ((symbol-function 'completing-read-multiple) (lambda (&rest _) nil))
                ((symbol-function 'completing-read) (lambda (&rest _) (pop answers))))
        (let ((op (litgraph-read-op-arguments "deadlines" describe nil)))
          (should (equal (gethash "op" op) "deadlines"))
          (should (equal (gethash "trigger" op) "2026-01-05")))))))

;;;; Formatting and tables

(ert-deftest litgraph-view-number-formatting ()
  (should (equal (litgraph-view-format-number 1234567.4) "1,234,567"))
  (should (equal (litgraph-view-format-number -48423.07) "-48,423"))
  (should (equal (litgraph-view-format-number 0.217164 t) "21.7%"))
  (should (equal (litgraph-view-format-number 60.0) "60"))
  (should (equal (litgraph-view-format-number 2.3456) "2.35"))
  (should (equal (litgraph-view-format-number 0.0123456) "0.01235"))
  (should (equal (litgraph-view-format-number 1000) "1,000")))

(ert-deftest litgraph-view-table-org-output ()
  (let* ((table (litgraph-view-table "T" '(("A" . left) ("B" . right))
                                     (list (list "x|y" (cons "1.0K" 1000.123456789))
                                           (list "z" (cons "2" 2)))))
         (org (litgraph-view-table-to-org table))
         (rows (litgraph-view-table-org-rows table)))
    (should (string-match-p "^#\\+CAPTION: T" org))
    (should (string-match-p "| A " org))
    (should (string-match-p "\\\\vert{}" org))
    (should (equal (car rows) '("A" "B")))
    (should (eq (cadr rows) 'hline))
    (should (equal (nth 2 rows) '("x|y" 1000.12)))
    (should (equal (nth 3 rows) '("z" 2)))))

(ert-deftest litgraph-view-auto-table-picks-scalars ()
  (let ((table (litgraph-view-auto-table
                "t" '(((id . "a") (n . 1) (nested . ((x . 1))))
                      ((id . "b") (n . 2.5))) 'id)))
    (should (equal (mapcar #'car (plist-get table :columns)) '("Id" "N")))
    (should (equal (plist-get table :ids) '("a" "b")))
    (should (= 2 (length (plist-get table :rows))))))

;;;; Result buffers

(defun litgraph-test--render (fixture)
  "Render FIXTURE into a temporary result buffer; return its text."
  (let ((response (litgraph-test-fixture-response fixture)))
    (with-temp-buffer
      (litgraph-view-mode)
      (litgraph-view--render response nil)
      (buffer-substring-no-properties (point-min) (point-max)))))

(ert-deftest litgraph-view-solve ()
  (let ((text (litgraph-test--render "solve")))
    (should (string-match-p "^\\* Summary" text))
    (should (string-match-p "Value +48,423" text))
    (should (string-match-p "Best line +35 steps" text))
    (should (string-match-p "^\\* Best line (35 steps)" text))))

(ert-deftest litgraph-view-chain-has-outcome-odds ()
  (let ((text (litgraph-test--render "chain")))
    (should (string-match-p "Odds .*21\\.7% JUDGMENT FOR PLAINTIFF" text))
    (should (string-match-p "^\\* Outcomes" text))
    (should (string-match-p "Expected dollars" text))))

(ert-deftest litgraph-view-warnings-grouped-and-provenance-last ()
  (let ((text (litgraph-test--render "chain")))
    (should (string-match-p "^\\* Warnings (3 kinds)" text))
    (should (string-match-p "fact-unset x2 -- a matter fact is not set" text))
    (should (string-match-p "probability-fill x27" text))
    (should (< (string-match "\\* Warnings" text)
               (string-match "\\* Provenance" text)))))

(ert-deftest litgraph-view-provenance-is-folded ()
  (let ((response (litgraph-test-fixture-response "chain")))
    (with-temp-buffer
      (litgraph-view-mode)
      (litgraph-view--render response nil)
      (goto-char (point-min))
      (re-search-forward "^\\* Provenance")
      (forward-line 1)
      (should (invisible-p (point)))
      (goto-char (point-min))
      (re-search-forward "^\\* Summary")
      (forward-line 1)
      (should-not (invisible-p (point))))))

(ert-deftest litgraph-view-tornado-path-sweep-explain ()
  (should (string-match-p "param:stakes +0\\.75" (litgraph-test--render "tornado")))
  (should (string-match-p "Most sensitive +param:stakes" (litgraph-test--render "tornado")))
  (should (string-match-p "^\\* Lines" (litgraph-test--render "path")))
  (should (string-match-p "^\\* Curve" (litgraph-test--render "sweep")))
  (should (string-match-p "Regret" (litgraph-test--render "explain")))
  (should (string-match-p "^\\* Distributions" (litgraph-test--render "simulate"))))

(ert-deftest litgraph-view-deadlines-sorted ()
  (let ((text (litgraph-test--render "deadlines-node")))
    (should (string-match-p "Deadlines +2" text))
    (should (string-match-p "2026-11-30" text))
    (should (string-match-p "RCFC 6" text))
    (should (string-match-p "Press d" text))))

(ert-deftest litgraph-view-batch-and-compare-nest ()
  (let ((batch (litgraph-test--render "batch")))
    (should (string-match-p "^\\* 1 solve: Summary" batch))
    (should (string-match-p "^\\* 2 chain: Summary" batch)))
  (let ((compare (litgraph-test--render "compare")))
    (should (string-match-p "^\\* Delta" compare))
    (should (string-match-p "^\\* Base: Summary" compare))
    (should (string-match-p "^\\* Variant: Summary" compare))))

(ert-deftest litgraph-view-generic-fallback-for-unknown-ops ()
  (let ((response (litgraph-parse
                   "{\"ok\":true,\"op\":\"posterior\",\"result\":{\"n\":3,\"intervals\":[{\"edge\":\"a\",\"lo\":0.1},{\"edge\":\"b\",\"lo\":0.2}],\"note\":\"hi\"},\"warnings\":[],\"provenance\":{},\"elapsed_ms\":1.0}")))
    (with-temp-buffer
      (litgraph-view-mode)
      (litgraph-view--render response nil)
      (let ((text (buffer-string)))
        (should (string-match-p "^\\* Summary" text))
        (should (string-match-p "Note +hi" text))
        (should (string-match-p "^\\* Intervals" text))
        (should (string-match-p "Edge +Lo" text))))))

(ert-deftest litgraph-view-renderer-error-falls-back ()
  (let ((litgraph-view-renderers
         (list (cons "solve" (lambda (_) (error "broken renderer")))))
        (response (litgraph-test-fixture-response "solve")))
    (with-temp-buffer
      (litgraph-view-mode)
      (litgraph-view--render response nil)
      (should (string-match-p "Best line" (buffer-string))))))

(ert-deftest litgraph-view-show-and-raw-json-and-table-buffer ()
  (litgraph-test-with-stub
    (let* ((engine (litgraph--engine-response (litgraph-test-fixture "chain")))
           (buffer (litgraph-view-show (car engine)
                                       (litgraph-build-request "chain" "s")
                                       (cdr engine))))
      (unwind-protect
          (with-current-buffer buffer
            (should (derived-mode-p 'litgraph-view-mode))
            (should (equal (buffer-name) "*litgraph: chain*"))
            (goto-char (point-min))
            (re-search-forward "JUDGMENT FOR PLAINTIFF")
            (litgraph-view-open-table)
            (with-current-buffer "*litgraph-table: Outcomes*"
              (should (derived-mode-p 'litgraph-table-mode))
              (should (= 7 (length tabulated-list-entries)))
              ;; sort by p, numerically
              (goto-char (point-min))
              (tabulated-list-sort 1)
              (let ((first (aref (cadr (car tabulated-list-entries)) 1)))
                (should (equal first "4.8%"))))
            (with-current-buffer buffer (litgraph-view-raw-json))
            (with-current-buffer "*litgraph-json: chain*"
              (should (string-match-p "\"absorption\"" (buffer-string)))
              (should (> (count-lines (point-min) (point-max)) 20))))
        (dolist (name '("*litgraph: chain*" "*litgraph-table: Outcomes*"
                        "*litgraph-json: chain*"))
          (when (get-buffer name) (kill-buffer name)))))))

(ert-deftest litgraph-view-act-on-parameter-row-runs-sweep ()
  (litgraph-test-with-stub
    (setq litgraph-target "session")
    (with-temp-buffer
      (litgraph-view-mode)
      (setq litgraph-view--request (litgraph-build-request "tornado" "shown"))
      (litgraph-view--act-on-id "param:stakes"))
    (let ((call (car litgraph-test-calls)))
      ;; The drill-down runs on the target the result came from.
      (should (equal "shown" (cadr (car call))))
      (should (equal "sweep" (litgraph-test-call-op-name call)))
      (should (equal "stakes" (gethash "param" (litgraph-test-call-op call)))))
    (when (get-buffer "*litgraph: sweep*") (kill-buffer "*litgraph: sweep*"))))

;;;; Graph

(ert-deftest litgraph-graph-dot-highlights-best-line-and-roles ()
  (let* ((graph (litgraph-get (litgraph-test-fixture-response "graph-cofc") 'result))
         (solve (litgraph-test-fixture-response "solve"))
         (best (litgraph-graph--best-edge-ids solve))
         (dot (litgraph-graph-dot graph best "My title")))
    (should (string-prefix-p "digraph litgraph {" dot))
    (should (string-match-p "label=\"My title\"" dot))
    (should (= 35 (length best)))
    (should (string-match-p "color=\"#d62728\", penwidth=2.5" dot))
    (should (string-match-p "fillcolor=\"#cfe4ff\"" dot))      ; you
    (should (string-match-p "fillcolor=\"#ffd9c7\"" dot))      ; opponent
    (should (string-match-p "fillcolor=\"#c9f2c7\"" dot))      ; win
    (should (string-match-p "fillcolor=\"#f6c9c9\"" dot))      ; loss
    (should (string-match-p "peripheries=2" dot))
    (should (string-match-p "label=\"95%\"" dot))
    ;; multi-line labels use dot's \n, never a literal backslash-n typo
    (should (string-match-p "\\\\n" dot))
    (should-not (string-match-p "\\\\\\\\n" dot))))

(ert-deftest litgraph-graph-quote-escapes ()
  (should (equal (litgraph-graph--quote "a\"b\\c\nd") "\"a\\\"b\\\\c\\nd\"")))

(ert-deftest litgraph-graph-text-fallback ()
  (let* ((graph (litgraph-get (litgraph-test-fixture-response "graph-cofc") 'result))
         (best (litgraph-graph--best-edge-ids (litgraph-test-fixture-response "solve")))
         (lines (litgraph-graph-text graph best)))
    (should (string-match-p "Claim Against the United States Accrues" (caar lines)))
    (should (seq-some (lambda (l) (string-match-p "★" (car l))) lines))
    (should (seq-some (lambda (l) (string-match-p "■ .*\\[\\$1,000,000\\]" (car l)))
                      lines))
    (should (cdr (cadr lines)))))

(ert-deftest litgraph-graph-text-truncates ()
  (let* ((graph (litgraph-get (litgraph-test-fixture-response "graph-cofc") 'result))
         (litgraph-graph-text-max-lines 5)
         (lines (litgraph-graph-text graph nil)))
    (should (= 6 (length lines)))
    (should (string-match-p "more lines not shown" (car (car (last lines)))))))

(ert-deftest litgraph-graph-buffer-shows-text-without-graphviz ()
  (litgraph-test-with-stub
    (let ((litgraph-graph-dot-executable "no-such-dot-binary"))
      (let ((buffer (litgraph-graph-show
                     (litgraph-get (litgraph-test-fixture-response "graph-cofc")
                                   'result)
                     nil "title")))
        (unwind-protect
            (with-current-buffer buffer
              (should (derived-mode-p 'litgraph-graph-mode))
              (should (string-match-p "Claim Against" (buffer-string)))
              (should (string-match-p "digraph" litgraph-graph--dot)))
          (kill-buffer buffer))))))

;;;; Org deadlines

(defun litgraph-test--deadlines ()
  (litgraph-get (litgraph-test-fixture-response "deadlines-node")
                'result 'deadlines))

(ert-deftest litgraph-org-timestamp ()
  (should (string-match-p "\\`<2026-11-30 Mon>\\'" (litgraph-org-timestamp "2026-11-30")))
  (let ((litgraph-org-warning-period "-7d"))
    (should (equal (litgraph-org-timestamp "2026-11-30") "<2026-11-30 Mon -7d>")))
  (should-error (litgraph-org-timestamp "30/11/2026")))

(ert-deftest litgraph-org-entry-string-shape ()
  (let ((entry (litgraph-org-entry-string (car (litgraph-test--deadlines)) "M")))
    (should (string-match-p "\\`\\*\\* TODO United States answers (RCFC 12(a)(1)(A)) +:deadline:\n" entry))
    (should (string-match-p "^DEADLINE: <2026-11-30 Mon>\n:PROPERTIES:" entry))
    (should (string-match-p ":LITGRAPH_ID: M|cofc::e-responsefork-answer" entry))
    (should (string-match-p ":LITGRAPH_STEPS:\nDue 2026-11-30 (RCFC 6(a))\\." entry))))

(defmacro litgraph-test-with-org-buffer (&rest body)
  (declare (indent 0))
  `(with-temp-buffer
     (org-mode)
     (let ((litgraph-org-warning-period nil))
       ,@body)))

(ert-deftest litgraph-org-files-deadlines-into-agenda-shape ()
  (litgraph-test-with-org-buffer
    (let ((summary (litgraph-org-file-deadlines (litgraph-test--deadlines) "Acme")))
      (should (= 2 (plist-get summary :added)))
      (should (= 0 (plist-get summary :updated)))
      (should (string-match-p "^\\* Matter: Acme +:litgraph:" (buffer-string)))
      (should (string-match-p ":CATEGORY: Acme" (buffer-string)))
      (let ((found (org-map-entries
                    (lambda () (org-entry-get nil "DEADLINE")) "LEVEL=2")))
        (should (equal (length found) 2))
        (should (string-match-p "2026-11-30" (car found)))))))

(ert-deftest litgraph-org-refiling-is-idempotent-and-keeps-user-edits ()
  (litgraph-test-with-org-buffer
    (litgraph-org-file-deadlines (litgraph-test--deadlines) "Acme")
    ;; The user marks the first deadline done and adds a note.
    (goto-char (point-min))
    (re-search-forward "^\\*\\* TODO")
    (org-todo "DONE")
    (org-back-to-heading t)
    (org-end-of-subtree t t)
    (insert "My own note.\n")
    ;; Re-file with a corrected trigger date.
    (let* ((moved (mapcar (lambda (d)
                            (let ((copy (copy-tree d)))
                              (setf (alist-get 'due_date copy) "2026-12-14")
                              (setf (alist-get 'trigger copy) "2026-10-15")
                              copy))
                          (litgraph-test--deadlines)))
           (summary (litgraph-org-file-deadlines moved "Acme")))
      (should (= 0 (plist-get summary :added)))
      (should (= 2 (plist-get summary :updated))))
    (let ((text (buffer-string)))
      (should (= 2 (length (org-map-entries #'point "LEVEL=2"))))
      (should (string-match-p "^\\*\\* DONE United States answers" text))
      (should (string-match-p "My own note\\." text))
      (should (= 2 (with-temp-buffer (insert text)
                                     (how-many "DEADLINE: <2026-12-14 Mon>"
                                               (point-min) (point-max)))))
      (should (string-match-p ":LITGRAPH_TRIGGER: 2026-10-15" text))
      (should (= 2 (with-temp-buffer (insert text)
                                     (how-many "^:LITGRAPH_STEPS:"
                                               (point-min) (point-max))))))))

(ert-deftest litgraph-org-different-matters-do-not-collide ()
  (litgraph-test-with-org-buffer
    (litgraph-org-file-deadlines (litgraph-test--deadlines) "A")
    (let ((summary (litgraph-org-file-deadlines (litgraph-test--deadlines) "B")))
      (should (= 2 (plist-get summary :added))))
    (should (= 4 (length (org-map-entries #'point "LEVEL=2"))))
    (should (= 2 (length (org-map-entries #'point "LEVEL=1"))))))

(ert-deftest litgraph-org-import-writes-file ()
  (litgraph-test-with-stub
    (let* ((file (make-temp-file "lg-dl" nil ".org"))
           (litgraph-org-file file) (litgraph-org-jump nil)
           (litgraph-org-offer-agenda nil)
           (response (litgraph-test-fixture-response "deadlines-node")))
      (unwind-protect
          (progn
            (delete-file file)
            (should (= 2 (plist-get (litgraph-org-import response nil "Acme")
                                    :added)))
            (let ((text (with-temp-buffer (insert-file-contents file)
                                          (buffer-string))))
              (should (string-match-p "#\\+title: Litigation deadlines" text))
              (should (string-match-p "DEADLINE: <2026-11-30" text))))
        (when-let* ((buffer (find-buffer-visiting file))) (kill-buffer buffer))
        (delete-file file)))))

(ert-deftest litgraph-org-import-rejects-empty ()
  (should-error (litgraph-org-import (litgraph-parse "{\"result\":{\"deadlines\":[]}}")
                                     nil "M")
                :type 'user-error))

;;;; Org Babel

(ert-deftest litgraph-ob-request-shapes ()
  (let ((r (litgraph-ob-request "{\"op\":\"chain\",\"top\":3}" '((:target . "m")))))
    (should (equal "m" (gethash "target" r)))
    (should (equal "chain" (gethash "op" (gethash "op" r))))
    (should (= 3 (gethash "top" (gethash "op" r))))
    (should-not (gethash "params" r)))
  (let ((r (litgraph-ob-request "chain" '((:target . "m") (:op . "tornado")))))
    (should (equal "tornado" (gethash "op" (gethash "op" r)))))
  (let ((r (litgraph-ob-request "  " '((:target . "m")))))
    (should (equal "solve" (gethash "op" (gethash "op" r)))))
  (should-error (litgraph-ob-request "chain" nil) :type 'user-error)
  (should-error (litgraph-ob-request "[1]" '((:target . "m"))) :type 'user-error)
  (should-error (litgraph-ob-request "{\"top\":3}" '((:target . "m"))) :type 'user-error))

(ert-deftest litgraph-ob-does-not-leak-session ()
  (let ((litgraph-target "leaky") (litgraph-params '(("rate" . 1))))
    (let ((r (litgraph-ob-request "chain" '((:target . "m")))))
      (should (equal "m" (gethash "target" r)))
      (should-not (gethash "params" r)))))

(ert-deftest litgraph-ob-formats ()
  (litgraph-test-with-stub
    (let* ((params '((:target . "m")))
           (table (org-babel-execute:litgraph "chain" params))
           (summary (org-babel-execute:litgraph "chain" (cons '(:format . "summary") params)))
           (json (org-babel-execute:litgraph "chain" (cons '(:format . "json") params)))
           (org (org-babel-execute:litgraph "chain" (cons '(:format . "org") params))))
      (should (equal (car table) '("Outcome" "p" "Utility" "Tags")))
      (should (eq (cadr table) 'hline))
      (should (member '("JUDGMENT FOR PLAINTIFF (Bench Trial Verdict)" 0.217164
                        1000000 "win, judgment, fee-eligible")
                      (cddr table)))
      (should (equal (car summary) '("Key" "Value")))
      (should (assoc "Expected net" (cddr summary)))
      (should (string-match-p "\"absorption\"" json))
      (should (string-match-p "^| Outcome" org)))))

(ert-deftest litgraph-ob-table-selection ()
  (litgraph-test-with-stub
    (let ((params '((:target . "m"))))
      (should (equal (caar (org-babel-execute:litgraph
                            "chain" (cons '(:table . "visited") params)))
                     "Node"))
      (should-error (org-babel-execute:litgraph
                     "chain" (cons '(:table . "nonexistent") params))
                    :type 'user-error))))

(ert-deftest litgraph-ob-vars-become-params ()
  (litgraph-test-with-stub
    (with-temp-buffer
      (org-mode)
      (insert "#+begin_src litgraph :target m :var rate=900 stakes=\"2\"\nchain\n#+end_src\n")
      (goto-char (point-min))
      (let* ((info (org-babel-get-src-block-info))
             (request (litgraph-ob-request (nth 1 info) (nth 2 info)))
             (params (gethash "params" request)))
        (should (= 900 (gethash "rate" params)))
        (should (= 2 (gethash "stakes" params)))
        (should (equal (litgraph-request-args request)
                       '("run" "m" "--op" "{\"op\":\"chain\"}"
                         "--set" "rate=900" "--set" "stakes=2" "--format" "json")))))))

(ert-deftest litgraph-ob-executes-in-org-buffer ()
  (litgraph-test-with-stub
    (let ((org-confirm-babel-evaluate nil))
      (with-temp-buffer
        (org-mode)
        (insert "#+begin_src litgraph :target m\n{\"op\": \"chain\"}\n#+end_src\n")
        (goto-char (point-min))
        (org-babel-execute-src-block)
        (should (string-match-p "^#\\+RESULTS:" (buffer-string)))
        (should (string-match-p "| Outcome +| +p +|" (buffer-string)))))))

;;;; Menu and autoloads

(ert-deftest litgraph-menu-defined ()
  (should (fboundp 'litgraph-menu-transient))
  (should (fboundp 'litgraph-menu))
  (should (commandp 'litgraph-solve))
  (should (commandp 'litgraph-run-op))
  (should (keymapp litgraph-command-map))
  (should (eq (lookup-key litgraph-command-map "d") 'litgraph-deadlines-to-org))
  (should (eq (lookup-key litgraph-command-map "C") 'litgraph-set-target)))

(ert-deftest litgraph-menu-greys-out-unsupported-ops ()
  (litgraph-test-with-stub
    (setq litgraph-target "m")
    (should (litgraph-menu--has-op "frobnicate"))     ; unknown until fetched
    (litgraph-with-describe #'ignore)
    (should-not (litgraph-menu--has-op "frobnicate"))
    (should (litgraph-menu--has-op "posterior"))
    (should (litgraph-menu--has-op "solve"))))

(ert-deftest litgraph-run-op-runs-and-displays ()
  (litgraph-test-with-stub
    (let* ((shown nil)
           (litgraph-display-function (lambda (resp req raw) (setq shown (list resp req raw))))
           (litgraph-target "cofc-1498-patent-case"))
      (cl-letf (((symbol-function 'completing-read-multiple) (lambda (&rest _) nil))
                ((symbol-function 'completing-read) (lambda (&rest _) "")))
        (litgraph-run-op "chain"))
      (should shown)
      (should (equal "chain" (litgraph-get (car shown) 'op)))
      (should (stringp (nth 2 shown))))))

(ert-deftest litgraph-graph-command-fetches-graph-and-solve ()
  (litgraph-test-with-stub
    (let ((litgraph-graph-dot-executable "no-such-dot-binary")
          (litgraph-target "cofc-1498-patent-case"))
      (litgraph-graph)
      (unwind-protect
          (with-current-buffer "*litgraph-graph*"
            (should (string-match-p "★" (buffer-string)))
            (should (string-match-p "solved value \\$48,423" litgraph-graph--title))
            (should (string-match-p "color=\"#d62728\"" litgraph-graph--dot)))
        (kill-buffer "*litgraph-graph*")))
    (should (equal '("graph" "solve")
                   (mapcar #'litgraph-test-call-op-name (litgraph-test-run-calls))))))

(ert-deftest litgraph-graph-text-root-ignores-inactive-parent-filter ()
  (let* ((graph (litgraph-parse
                 "{\"nodes\":[{\"id\":\"branch\",\"label\":\"Inactive branch\"},{\"id\":\"root\",\"label\":\"Real root\"},{\"id\":\"leaf\",\"label\":\"Leaf\"}],\"edges\":[{\"id\":\"inactive\",\"from\":\"leaf\",\"to\":\"branch\",\"active\":false},{\"id\":\"active\",\"from\":\"root\",\"to\":\"leaf\",\"active\":true}]}"))
         (litgraph-graph-show-inactive nil)
         (lines (litgraph-graph-text graph)))
    (should (equal (cdar lines) "root"))
    (should (string-match-p "Real root" (caar lines)))))

(ert-deftest litgraph-graph-text-prefers-declared-start ()
  (let* ((graph (litgraph-parse
                 "{\"start\":{\"id\":\"root\"},\"nodes\":[{\"id\":\"first\",\"label\":\"First root\"},{\"id\":\"root\",\"label\":\"Declared root\"}],\"edges\":[]}"))
         (lines (litgraph-graph-text graph)))
    (should (equal (cdar lines) "root"))))

(ert-deftest litgraph-deadlines-to-org-command-end-to-end ()
  (litgraph-test-with-stub
    (let* ((file (make-temp-file "lg-e2e" nil ".org"))
           (litgraph-org-file file) (litgraph-org-jump nil)
           (litgraph-org-offer-agenda nil)
           (litgraph-target "cofc-1498-patent-case"))
      (unwind-protect
          (progn
            (delete-file file)
            (cl-letf (((symbol-function 'org-read-date) (lambda (&rest _) "2026-10-01"))
                      ((symbol-function 'completing-read)
                       (lambda (&rest _) "cofc::government-response-fork"))
                      ((symbol-function 'read-string) (lambda (&rest _) "Acme v. US")))
              (litgraph-deadlines-to-org))
            (let ((text (with-temp-buffer (insert-file-contents file) (buffer-string))))
              (should (string-match-p "^\\* Matter: Acme v. US" text))
              (should (string-match-p "DEADLINE: <2026-11-30 Mon>" text)))
            (let ((op (litgraph-test-call-op
                       (seq-find (lambda (c)
                                   (equal (litgraph-test-call-op-name c) "deadlines"))
                                 litgraph-test-calls))))
              (should (equal "2026-10-01" (gethash "trigger" op)))
              (should (equal "cofc::government-response-fork" (gethash "node" op)))))
        (when-let* ((buffer (find-buffer-visiting file))) (kill-buffer buffer))
        (delete-file file)))))

(ert-deftest litgraph-menu-layout-parses ()
  (should (transient-get-suffix 'litgraph-menu-transient "s"))
  (should (transient-get-suffix 'litgraph-menu-transient "d"))
  (should (transient-get-suffix 'litgraph-menu-transient "r")))

(ert-deftest litgraph-autoloads-cover-entry-points ()
  (let* ((dir (make-temp-file "lg-al" t))
         (file (expand-file-name "litgraph-autoloads.el" dir)))
    (unwind-protect
        (progn
          (loaddefs-generate (file-name-directory (locate-library "litgraph.el"))
                             file)
          (let ((text (with-temp-buffer (insert-file-contents file) (buffer-string))))
            (dolist (name '("litgraph-menu" "litgraph-run-op" "litgraph-solve"
                            "litgraph-deadlines-to-org" "litgraph-graph"
                            "litgraph-set-target" "litgraph-command-map"
                            "litgraph-timeline"))
              (should (string-match-p (format "\\(?:autoload\\|defun\\).?'?%s\\b" name)
                                      text)))
            (should (string-match-p "auto-mode-alist" text))))
      (delete-directory dir t))))

(ert-deftest litgraph-graph-zoom-without-image-is-a-user-error ()
  (with-temp-buffer
    (litgraph-graph-mode)
    (let ((inhibit-read-only t)) (insert "just text"))
    (should-error (litgraph-graph-zoom-in) :type 'user-error)))

(provide 'litgraph-unit-test)

;;; litgraph-unit-test.el ends here
