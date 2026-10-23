;;; litgraph-unit-test.el --- Unit tests (process layer stubbed) -*- lexical-binding: t; -*-

;; SPDX-License-Identifier: GPL-3.0-or-later

;;; Code:

(require 'litgraph-test-helper)
(require 'litgraph)
(require 'litgraph-view)
(require 'litgraph-graph)
(require 'litgraph-org)
(require 'litgraph-scenario)
(require 'litgraph-menu)
(require 'ob-litgraph)

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

(ert-deftest litgraph-build-request-named-scenario ()
  (litgraph-test-with-stub
    (let ((r (litgraph-build-request "solve" "cofc-1498-patent-case")))
      (should (equal (gethash "scenario" r) "cofc-1498-patent-case"))
      (should (equal (litgraph-serialize (gethash "op" r)) "{\"op\":\"solve\"}")))))

(ert-deftest litgraph-build-request-params-extend-name ()
  (litgraph-test-with-stub
    (let* ((r (litgraph-build-request "chain" "cofc-1498-patent-case" '("cofc")
                                      '(("rate" . 900))))
           (scenario (gethash "scenario" r)))
      (should (equal (gethash "extends" scenario) "cofc-1498-patent-case"))
      (should (= (gethash "rate" (gethash "params" scenario)) 900))
      (should (equal (append (gethash "packs" r) nil) '("cofc"))))))

(ert-deftest litgraph-build-request-uses-session ()
  (litgraph-test-with-stub
    (setq litgraph-scenario "frcp-civil-case"
          litgraph-packs '("frcp-civil-procedure")
          litgraph-params '(("rate" . 1)))
    (let ((r (litgraph-build-request "solve")))
      (should (equal (gethash "extends" (gethash "scenario" r))
                     "frcp-civil-case"))
      (should (equal (append (gethash "packs" r) nil)
                     '("frcp-civil-procedure"))))))

(ert-deftest litgraph-build-request-from-request-file ()
  (litgraph-test-with-stub
    (let ((file (make-temp-file "lg" nil ".json"
                                "{\"packs\":[\"cofc\"],\"scenario\":{\"params\":{\"rate\":700}},\"op\":{\"op\":\"chain\"}}")))
      (unwind-protect
          (let ((r (litgraph-build-request "solve" file)))
                  (should (equal (litgraph-serialize (gethash "op" r)) "{\"op\":\"solve\"}"))
            (should (equal (append (gethash "packs" r) nil) '("cofc")))
            (should (= 700 (gethash "rate"
                                    (gethash "params"
                                             (gethash "scenario" r))))))
        (delete-file file)))))

(ert-deftest litgraph-build-request-from-named-scenario-doc ()
  (litgraph-test-with-stub
    (let ((doc (litgraph-parse-strict
                "{\"id\":\"x\",\"packs\":[\"cofc\"],\"scenario\":{\"params\":{\"rate\":1}}}")))
      (let ((r (litgraph-build-request "solve" doc)))
        (should (equal (append (gethash "packs" r) nil) '("cofc")))
        (should (= 1 (gethash "rate" (gethash "params" (gethash "scenario" r)))))))))

(ert-deftest litgraph-request-success-and-raw ()
  (litgraph-test-with-stub
    (let (got)
      (litgraph-request (litgraph-build-request "solve" "s")
                        (lambda (response) (setq got response)))
      (should (litgraph-get got 'ok))
      (should (numberp (litgraph-get got 'result 'value)))
      (should (string-match-p "\"best_line\"" litgraph--last-raw))
      (should (equal (cdr (car litgraph-test-calls))
                     (litgraph-serialize (litgraph-build-request "solve" "s")))))))

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
    (litgraph-request "{}" #'ignore)
    (should (string-match-p "exit 1: boom" litgraph-last-error)))
  (let ((litgraph-process-function (lambda (_a _i cb) (funcall cb 'timeout "" "")))
        (litgraph-last-error nil))
    (litgraph-request "{}" #'ignore)
    (should (string-match-p "timed out" litgraph-last-error))))

(ert-deftest litgraph-request-sync-returns-and-signals ()
  (litgraph-test-with-stub
    (should (litgraph-get (litgraph-request-sync
                           (litgraph-build-request "chain" "s"))
                          'ok))
    (should-error (litgraph-request-sync (litgraph-build-request "nonsense" "s"))
                  :type 'error)))

(ert-deftest litgraph-environment-exports-dirs ()
  (let ((litgraph-packs-dir "/tmp/packs") (litgraph-scenarios-dir "/tmp/sc"))
    (let ((env (litgraph--environment)))
      (should (member "LITGRAPH_PACKS=/tmp/packs" env))
      (should (member "LITGRAPH_SCENARIOS=/tmp/sc" env))))
  (let ((litgraph-packs-dir nil) (litgraph-scenarios-dir nil))
    (should-not (seq-some (lambda (e) (string-prefix-p "LITGRAPH_PACKS=" e))
                          (seq-take (litgraph--environment) 1)))))

(ert-deftest litgraph-executable-missing-is-user-error ()
  (let ((litgraph-executable "definitely-not-a-real-binary-xyz")
        (litgraph-fallback-directories nil))
    (should-error (litgraph-executable-path) :type 'user-error)))

;;;; Metadata and completion

(ert-deftest litgraph-metadata-comes-from-describe ()
  (litgraph-test-with-stub
    (should (member "solve" (litgraph-op-names)))     ; fallback list
    (litgraph-with-describe #'ignore)
    (should (member "deadlines" (litgraph-op-names)))
    (should (assoc "cofc" (litgraph-pack-candidates)))
    (should (assoc "cofc-1498-patent-case" (litgraph-scenario-candidates)))
    (should (assoc "rate" (litgraph-param-candidates)))
    (should (assoc "dollars" (litgraph-metric-candidates)))
    (should (litgraph-op-available-p "solve"))
    (should-not (litgraph-op-available-p "posterior"))))

(ert-deftest litgraph-describe-is-cached ()
  (litgraph-test-with-stub
    (litgraph-with-describe #'ignore)
    (litgraph-with-describe #'ignore)
    (should (= 1 (length litgraph-test-calls)))))

(ert-deftest litgraph-graph-candidates ()
  (litgraph-test-with-stub
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

(ert-deftest litgraph-op-arg-specs-from-schema ()
  (let* ((schema (litgraph-test-fixture-response "schema-request"))
         (deadlines (litgraph-op-arg-specs schema "deadlines"))
         (sweep (litgraph-op-arg-specs schema "sweep"))
         (structure (litgraph-op-arg-specs schema "structure")))
    (should (plist-get (seq-find (lambda (s) (equal (plist-get s :name) "trigger"))
                                 deadlines)
                       :required))
    (should (eq 'boolean (plist-get (seq-find (lambda (s)
                                                (equal (plist-get s :name)
                                                       "reachable"))
                                              deadlines)
                                    :type)))
    (should (eq 'number (plist-get (seq-find (lambda (s) (equal (plist-get s :name) "lo"))
                                             sweep)
                                   :type)))
    (let ((what (seq-find (lambda (s) (equal (plist-get s :name) "what")) structure)))
      (should (or (plist-get what :enum) (plist-get what :oneof))))
    (should-not (litgraph-op-arg-specs schema "no-such-op"))))

(ert-deftest litgraph-op-arg-specs-fall-back-to-describe ()
  (let* ((describe (litgraph-get (litgraph-test-fixture-response "describe") 'result))
         (specs (litgraph-op-arg-specs-from-describe describe "deadlines")))
    (should (member "trigger" (mapcar (lambda (s) (plist-get s :name)) specs)))
    (should (plist-get (car specs) :required))))

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
    (let ((schema (litgraph-test-fixture-response "schema-request"))
          (answers '("2026-01-05" "cofc::claim-accrues")))
      (cl-letf (((symbol-function 'org-read-date) (lambda (&rest _) (pop answers)))
                ((symbol-function 'completing-read-multiple) (lambda (&rest _) nil))
                ((symbol-function 'completing-read) (lambda (&rest _) (pop answers))))
        (let ((op (litgraph-read-op-arguments "deadlines" schema nil nil)))
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
    (should (string-match-p "Odds +21\\.7% JUDGMENT FOR PLAINTIFF" text))
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
    (let* ((raw (litgraph-test-fixture "chain"))
           (buffer (litgraph-view-show (litgraph-parse raw)
                                       (litgraph-build-request "chain" "s") raw)))
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
    (litgraph-view--act-on-id "param:stakes")
    (let ((request (litgraph-parse-strict (cdr (car litgraph-test-calls)))))
      (should (equal "sweep" (gethash "op" (gethash "op" request))))
      (should (equal "stakes" (gethash "param" (gethash "op" request)))))
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
    (should (string-match-p ":LITGRAPH_STEPS:\nDue 2026-11-30 (RCFC 6)\\." entry))))

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

;;;; Scenario editing and flymake

(ert-deftest litgraph-scenario-locates-tokens ()
  (with-temp-buffer
    (insert "{\n  \"scenario\": {\"bogus\": 1},\n  \"remove_edges\": [\"nope::x\"]\n}")
    (let ((region (litgraph-scenario--locate
                   "parse: unknown field `bogus`, expected one of `params`")))
      (should (equal (buffer-substring (car region) (cdr region)) "\"bogus\"")))
    (let ((region (litgraph-scenario--locate "not found: edge nope::x")))
      (should (equal (buffer-substring (car region) (cdr region)) "\"nope::x\"")))
    (let ((region (litgraph-scenario--locate "something unlocatable")))
      (should (= (car region) 1)))))

(ert-deftest litgraph-flymake-reports-validate-errors ()
  (litgraph-test-with-stub
    (with-temp-buffer
      (insert "{\"scenario\": {\"bogus\": 1}, \"op\": {\"op\": \"solve\"}}")
      (let (reported)
        (litgraph-flymake (lambda (diags &rest _) (setq reported diags)))
        (should (= 1 (length reported)))
        (let ((d (car reported)))
          (should (eq (flymake-diagnostic-type d) :error))
          (should (string-match-p "unknown field" (flymake-diagnostic-text d)))
          (should (equal (buffer-substring (flymake-diagnostic-beg d)
                                           (flymake-diagnostic-end d))
                         "\"bogus\"")))
        (should (equal (car (car litgraph-test-calls)) '("validate" "-")))))))

(ert-deftest litgraph-flymake-syntax-error-skips-engine ()
  (litgraph-test-with-stub
    (with-temp-buffer
      (insert "{\n  \"a\": 1,\n  \"b\": [1, 2,,]\n}")
      (let (reported)
        (litgraph-flymake (lambda (diags &rest _) (setq reported diags)))
        (should (= 1 (length reported)))
        (should (string-match-p "syntax" (flymake-diagnostic-text (car reported))))
        (should (= (line-number-at-pos (flymake-diagnostic-beg (car reported))) 3))
        (should-not litgraph-test-calls)))))

(ert-deftest litgraph-flymake-valid-document-reports-nothing ()
  (litgraph-test-with-stub
    (let ((litgraph-test-validate-fixture "validate-ok"))
      (with-temp-buffer
        (insert "{\"op\": {\"op\": \"solve\"}}")
        (let ((reported 'unset))
          (litgraph-flymake (lambda (diags &rest _) (setq reported diags)))
          (should (null reported)))))))

(ert-deftest litgraph-flymake-warnings-are-opt-in ()
  (litgraph-test-with-stub
    (let ((litgraph-test-validate-fixture "validate-ok")
          (litgraph-flymake-warnings t))
      (with-temp-buffer
        (insert "{\"op\": {\"op\": \"solve\"}}")
        (let (reported)
          (litgraph-flymake (lambda (diags &rest _) (setq reported diags)))
          (should reported)
          (should (eq :note (flymake-diagnostic-type (car reported)))))))))

(ert-deftest litgraph-flymake-discards-stale-results ()
  (let* ((pending nil)
        (litgraph-process-function (lambda (_a _i cb) (push cb pending) nil))
        (reports 0))
    (with-temp-buffer
      (insert "{\"op\": {\"op\": \"solve\"}}")
      (litgraph-flymake (lambda (&rest _) (cl-incf reports)))
      (litgraph-flymake (lambda (&rest _) (cl-incf reports)))
      (funcall (cadr pending) 0 (litgraph-test-fixture "validate-ok") "")
      (funcall (car pending) 0 (litgraph-test-fixture "validate-ok") "")
      (should (= reports 1)))))

(ert-deftest litgraph-scenario-template-is-a-runnable-request ()
  (litgraph-test-with-stub
    (let* ((text (litgraph-scenario--template "cofc-1498-patent-case"))
           (doc (litgraph-parse-strict text)))
      (should (equal "cofc-1498-patent-case"
                     (gethash "extends" (gethash "scenario" doc))))
      (should (equal "solve" (gethash "op" (gethash "op" doc)))))))

(ert-deftest litgraph-scenario-new-and-open-enable-the-mode ()
  (litgraph-test-with-stub
    (let* ((dir (make-temp-file "lgs" t))
           (file (expand-file-name "m.litgraph.json" dir)))
      (unwind-protect
          (progn
            (litgraph-new-scenario file "cofc-1498-patent-case")
            (should (file-exists-p file))
            (should litgraph-scenario-mode)
            (should (derived-mode-p 'litgraph-scenario-json-mode))
            (should (memq #'litgraph-flymake flymake-diagnostic-functions))
            (kill-buffer (current-buffer)))
        (delete-directory dir t)))))

(ert-deftest litgraph-scenario-run-sends-whole-request ()
  (litgraph-test-with-stub
    (with-temp-buffer
      (insert "{\"scenario\":\"cofc-1498-patent-case\",\"op\":{\"op\":\"chain\"}}")
      (litgraph-scenario-run)
      (should (equal (litgraph--request-op-name
                      (litgraph-parse-strict (cdr (car litgraph-test-calls))))
                     "chain")))
    (when (get-buffer "*litgraph: chain*") (kill-buffer "*litgraph: chain*"))))

(ert-deftest litgraph-scenario-capf-completes-ids ()
  (litgraph-test-with-stub
    (litgraph-with-describe #'ignore)
    (litgraph-with-graph #'ignore)
    (with-temp-buffer
      (js-json-mode)
      (insert "{\"remove_edges\": [\"cofc::e-accr")
      (let ((capf (litgraph-scenario-capf)))
        (should capf)
        (should (member "cofc::e-accrual-to-patentfork"
                        (all-completions "cofc::e-accr" (nth 2 capf)))))
      (erase-buffer)
      (insert "{\"scenario\": {\"extends\": \"cofc-")
      (let ((capf (litgraph-scenario-capf)))
        (should (equal '("cofc-1498-patent-case" "cofc-bid-protest")
                       (all-completions "cofc-" (nth 2 capf))))))))

;;;; Org Babel

(ert-deftest litgraph-ob-request-shapes ()
  (let ((r (litgraph-ob-request "{\"scenario\":\"s\",\"op\":{\"op\":\"chain\"}}" nil)))
    (should (equal "chain" (gethash "op" (gethash "op" r)))))
  (let ((r (litgraph-ob-request "{\"scenario\":\"s\",\"op\":{\"op\":\"chain\"}}"
                                '((:op . "tornado")))))
    (should (equal "tornado" (gethash "op" (gethash "op" r)))))
  (let ((r (litgraph-ob-request "" '((:op . "chain") (:scenario . "s") (:packs . "a,b")))))
    (should (equal "s" (gethash "scenario" r)))
    (should (equal '("a" "b") (append (gethash "packs" r) nil))))
  (let ((r (litgraph-ob-request "{\"params\":{\"rate\":5}}" nil)))
    (should (equal "solve" (gethash "op" (gethash "op" r))))
    (should (= 5 (gethash "rate" (gethash "params" (gethash "scenario" r))))))
  (should-error (litgraph-ob-request "" nil) :type 'user-error)
  (should-error (litgraph-ob-request "[1]" nil) :type 'user-error))

(ert-deftest litgraph-ob-does-not-leak-session ()
  (let ((litgraph-scenario "leaky") (litgraph-params '(("rate" . 1))))
    (let ((r (litgraph-ob-request "{\"scenario\":\"s\",\"op\":{\"op\":\"chain\"}}" nil)))
      (should (equal "s" (gethash "scenario" r))))))

(ert-deftest litgraph-ob-formats ()
  (litgraph-test-with-stub
    (let* ((body "{\"scenario\":\"s\",\"op\":{\"op\":\"chain\"}}")
           (table (org-babel-execute:litgraph body nil))
           (summary (org-babel-execute:litgraph body '((:format . "summary"))))
           (json (org-babel-execute:litgraph body '((:format . "json"))))
           (org (org-babel-execute:litgraph body '((:format . "org")))))
      (should (equal (car table) '("Outcome" "p" "Utility" "Tags")))
      (should (eq (cadr table) 'hline))
      (should (equal (caddr table)
                     '("JUDGMENT FOR PLAINTIFF (Bench Trial Verdict)" 0.217164
                       1000000 "win, judgment, fee-eligible")))
      (should (equal (car summary) '("Key" "Value")))
      (should (assoc "Expected net" (cddr summary)))
      (should (string-match-p "\"absorption\"" json))
      (should (string-match-p "^| Outcome" org)))))

(ert-deftest litgraph-ob-table-selection ()
  (litgraph-test-with-stub
    (let ((body "{\"scenario\":\"s\",\"op\":{\"op\":\"chain\"}}"))
      (should (equal (caar (org-babel-execute:litgraph body '((:table . "visited"))))
                     "Node"))
      (should-error (org-babel-execute:litgraph body '((:table . "nonexistent")))
                    :type 'user-error))))

(ert-deftest litgraph-ob-vars-become-params ()
  (litgraph-test-with-stub
    (let ((request (litgraph-ob-request "{\"scenario\":\"s\",\"op\":{\"op\":\"chain\"}}" nil)))
      (litgraph-ob--apply-vars request '((rate . 900) ("stakes" . "2")))
      (let ((params (gethash "params" (gethash "scenario" request))))
        (should (= 900 (gethash "rate" params)))
        (should (= 2 (gethash "stakes" params)))
        (should (equal "s" (gethash "extends" (gethash "scenario" request))))))))

(ert-deftest litgraph-ob-executes-in-org-buffer ()
  (litgraph-test-with-stub
    (let ((org-confirm-babel-evaluate nil))
      (with-temp-buffer
        (org-mode)
        (insert "#+begin_src litgraph\n{\"scenario\":\"s\",\"op\":{\"op\":\"chain\"}}\n#+end_src\n")
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
  (should (eq (lookup-key litgraph-command-map "d") 'litgraph-deadlines-to-org)))

(ert-deftest litgraph-menu-greys-out-unsupported-ops ()
  (litgraph-test-with-stub
    (should (litgraph-menu--has-op "posterior"))      ; unknown until fetched
    (litgraph-with-describe #'ignore)
    (should-not (litgraph-menu--has-op "posterior"))
    (should (litgraph-menu--has-op "solve"))))

(ert-deftest litgraph-run-op-runs-and-displays ()
  (litgraph-test-with-stub
    (let* ((shown nil)
           (litgraph-display-function (lambda (resp req raw) (setq shown (list resp req raw))))
           (litgraph-scenario "cofc-1498-patent-case"))
      (cl-letf (((symbol-function 'completing-read-multiple) (lambda (&rest _) nil))
                ((symbol-function 'completing-read) (lambda (&rest _) "")))
        (litgraph-run-op "chain"))
      (should shown)
      (should (equal "chain" (litgraph-get (car shown) 'op)))
      (should (stringp (nth 2 shown))))))

(ert-deftest litgraph-graph-command-fetches-graph-and-solve ()
  (litgraph-test-with-stub
    (let ((litgraph-graph-dot-executable "no-such-dot-binary")
          (litgraph-scenario "cofc-1498-patent-case"))
      (litgraph-graph)
      (unwind-protect
          (with-current-buffer "*litgraph-graph*"
            (should (string-match-p "★" (buffer-string)))
            (should (string-match-p "solved value \\$48,423" litgraph-graph--title))
            (should (string-match-p "color=\"#d62728\"" litgraph-graph--dot)))
        (kill-buffer "*litgraph-graph*")))
    (should (equal '("graph" "solve")
                   (mapcar (lambda (call)
                             (litgraph-test--request-op (cdr call)))
                           (seq-filter (lambda (call) (equal (car call) '("q" "-")))
                                       (reverse litgraph-test-calls)))))))

(ert-deftest litgraph-deadlines-to-org-command-end-to-end ()
  (litgraph-test-with-stub
    (let* ((file (make-temp-file "lg-e2e" nil ".org"))
           (litgraph-org-file file) (litgraph-org-jump nil)
           (litgraph-org-offer-agenda nil)
           (litgraph-scenario "cofc-1498-patent-case"))
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
            (let ((request (litgraph-parse-strict
                            (cdr (car (seq-filter
                                       (lambda (c) (string-match-p "deadlines" (or (cdr c) "")))
                                       litgraph-test-calls))))))
              (should (equal "2026-10-01" (gethash "trigger" (gethash "op" request))))
              (should (equal "cofc::government-response-fork"
                             (gethash "node" (gethash "op" request))))))
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
                            "litgraph-open-scenario" "litgraph-new-scenario"
                            "litgraph-scenario-mode" "litgraph-command-map"
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
