;;; litgraph-integration-test.el --- Tests against the real litigate binary -*- lexical-binding: t; -*-

;; SPDX-License-Identifier: GPL-3.0-or-later

;;; Commentary:

;; Tests run the `litigate' on PATH (`litigate engine' for engine
;; calls, `litigate timeline' for the timeline) and skip unavailable
;; tools unless LITGRAPH_IT_REQUIRE=1.  The engine compiles its packs
;; from the nearest `lit/' (this repository's) on every call unless
;; LITGRAPH_PACKS and LITGRAPH_SCENARIOS name a prebuilt set, as `just
;; emacs-test' does (target/lit-build from `just lit-sync').

;;; Code:

(require 'litgraph-test-helper)
(require 'litgraph)
(require 'litgraph-view)
(require 'litgraph-graph)
(require 'litgraph-org)
(require 'litgraph-scenario)
(require 'ob-litgraph)
(require 'litgraph-timeline)

(defmacro litgraph-it (&rest body)
  "Run BODY against the real binary with a clean session, or skip/fail."
  (declare (indent 0))
  `(let ((binary (litgraph-test-real-binary)))
     (unless binary (litgraph-test-skip-or-fail "litigate binary not installed"))
     (let ((litgraph-executable binary)
           (litgraph--cache (make-hash-table :test #'equal))
           (litgraph-scenario nil) (litgraph-packs nil) (litgraph-params nil)
           (litgraph-default-scenario nil) (litgraph-default-packs nil))
       ,@body)))

(ert-deftest litgraph-it-describe-discovers-ops ()
  (litgraph-it
    (let ((response (litgraph-request-sync (litgraph-obj "op" (litgraph-obj "op" "describe")))))
      (should (litgraph-get response 'ok))
      (litgraph-with-describe #'ignore)
      (litgraph-with-schema #'ignore)
      ;; Bound the wait by `litgraph-timeout', as `litgraph-request-sync'
      ;; does: without prebuilt packs a debug build compiles lit/ on every
      ;; call, which takes longer than any fixed few seconds.
      (let ((deadline (+ (float-time) litgraph-timeout)))
        (while (and (< (float-time) deadline)
                    (not (and (litgraph--cached-describe)
                              (gethash (litgraph--cache-key 'schema)
                                       litgraph--cache))))
          (accept-process-output nil 0.05)))
      (should (member "solve" (litgraph-op-names)))
      (should (member "deadlines" (litgraph-op-names)))
      (should (assoc "cofc" (litgraph-pack-candidates)))
      (should (assoc "cofc-1498-patent-case" (litgraph-scenario-candidates)))
      (let ((schema (gethash (litgraph--cache-key 'schema) litgraph--cache)))
        (dolist (op (litgraph-op-names))
          (when (member op '("solve" "chain" "sweep" "deadlines"))
            (should (litgraph-op-arg-specs schema op))))))))

(ert-deftest litgraph-it-solve-chain-render ()
  (litgraph-it
    (dolist (op '("solve" "chain" "tornado" "simulate" "pareto" "structure"))
      (let* ((request (litgraph-build-request op "cofc-1498-patent-case"))
             (response (litgraph-request-sync request)))
        (should (litgraph-get response 'ok))
        (with-temp-buffer
          (litgraph-view-mode)
          (litgraph-view--render response request)
          (should (string-match-p "^\\* Summary" (buffer-string)))
          (should (string-match-p "cofc-1498-patent-case" (buffer-string))))))))

(ert-deftest litgraph-it-engine-error-is-reported ()
  (litgraph-it
    (should-error (litgraph-request-sync
                   (litgraph-build-request (litgraph-obj "op" "explain" "node" "nope")
                                           "cofc-1498-patent-case"))
                  :type 'error)))

(ert-deftest litgraph-it-graph-dot-is-valid-graphviz ()
  (litgraph-it
    (let* ((graph (litgraph-get (litgraph-request-sync
                                 (litgraph-build-request "graph" "cofc-1498-patent-case"))
                                'result))
           (solve (litgraph-request-sync (litgraph-build-request "solve" "cofc-1498-patent-case")))
           (dot (litgraph-graph-dot graph (litgraph-graph--best-edge-ids solve) "t")))
      (should (> (length (litgraph-graph-text graph nil)) 20))
      (if (executable-find "dot")
          (with-temp-buffer
            (insert dot)
            (should (zerop (call-process-region (point-min) (point-max) "dot" t nil nil
                                                "-Tsvg" "-o" null-device))))
        (litgraph-test-skip-or-fail "graphviz not installed")))))

(ert-deftest litgraph-it-deadlines-to-org ()
  (litgraph-it
    (let* ((file (make-temp-file "lg-it" nil ".org"))
           (litgraph-org-file file) (litgraph-org-jump nil)
           (litgraph-org-offer-agenda nil)
           (request (litgraph-build-request
                     (litgraph-obj "op" "deadlines" "trigger" "2026-01-05"
                                   "node" "frcp-civil-procedure::service-completed")
                     "frcp-civil-case"))
           (response (litgraph-request-sync request)))
      (unwind-protect
          (progn
            (delete-file file)
            (should (>= (plist-get (litgraph-org-import response request "IT") :added) 1))
            (let ((text (with-temp-buffer (insert-file-contents file) (buffer-string))))
              (should (string-match-p "DEADLINE: <2026-01-26 Mon>" text))
              (should (string-match-p "FRCP 6" text))))
        (when-let* ((buffer (find-buffer-visiting file))) (kill-buffer buffer))
        (delete-file file)))))

(ert-deftest litgraph-it-flymake-validates-real-documents ()
  (litgraph-it
    (dolist (case '(("{\"scenario\": {\"bogus\": 1}, \"op\": {\"op\": \"solve\"}}" . t)
                    ("{\"scenario\": \"cofc-1498-patent-case\", \"op\": {\"op\": \"solve\"}}" . nil)
                    ("{\"scenario\": \"cofc-1498-patent-case\", \"remove_edges\": [\"nope::x\"], \"op\": {\"op\": \"solve\"}}" . t)))
      (with-temp-buffer
        (insert (car case))
        (let ((reported 'unset) (litgraph-process-function #'litgraph--spawn))
          (litgraph-flymake (lambda (diags &rest _) (setq reported diags)))
          (let ((deadline (+ (float-time) litgraph-timeout)))
            (while (and (eq reported 'unset) (< (float-time) deadline))
              (accept-process-output nil 0.05)))
          (should-not (eq reported 'unset))
          (if (cdr case)
              (should (seq-some (lambda (d) (eq :error (flymake-diagnostic-type d)))
                                reported))
            (should-not reported)))))))

(ert-deftest litgraph-it-babel-block ()
  (litgraph-it
    (let ((org-confirm-babel-evaluate nil)
          (litgraph-executable (litgraph-test-real-binary)))
      (with-temp-buffer
        (org-mode)
        (insert "#+begin_src litgraph :var rate=900\n{\"scenario\":\"cofc-1498-patent-case\",\"op\":{\"op\":\"chain\"}}\n#+end_src\n")
        (goto-char (point-min))
        (org-babel-execute-src-block)
        (should (string-match-p "| Outcome" (buffer-string)))
        (should (string-match-p "| Utility" (buffer-string)))))))

(ert-deftest litgraph-it-async-does-not-block ()
  (litgraph-it
    (let ((done nil) (ticks 0))
      (litgraph-request (litgraph-build-request "simulate" "cofc-1498-patent-case")
                        (lambda (_) (setq done t)))
      ;; The call returned immediately; we can do other work until it lands.
      (should-not done)
      (let ((deadline (+ (float-time) 30)))
        (while (and (not done) (< (float-time) deadline))
          (cl-incf ticks)
          (accept-process-output nil 0.01)))
      (should done))))

(ert-deftest litgraph-it-timeout-kills-process ()
  (litgraph-it
    (let ((litgraph-timeout 0.001) (result nil))
      (litgraph-request (litgraph-build-request
                         (litgraph-obj "op" "simulate" "runs" 50000000)
                         "cofc-1498-patent-case")
                        #'ignore
                        (lambda (message &optional _r) (setq result message)))
      (let ((deadline (+ (float-time) 15)))
        (while (and (not result) (< (float-time) deadline))
          (accept-process-output nil 0.05)))
      (should (and result (string-match-p "timed out" result))))))

(ert-deftest litgraph-it-graphviz-renders-svg-asynchronously ()
  (litgraph-it
    (unless (executable-find "dot")
      (litgraph-test-skip-or-fail "graphviz not installed"))
    (let* ((graph (litgraph-get (litgraph-request-sync
                                 (litgraph-build-request "graph" "cofc-1498-patent-case"))
                                'result))
           (dot (litgraph-graph-dot graph nil "t"))
           (result 'pending))
      (litgraph-graph--render-image dot (lambda (data) (setq result data)))
      (should (eq result 'pending))
      (let ((deadline (+ (float-time) 30)))
        (while (and (eq result 'pending) (< (float-time) deadline))
          (accept-process-output nil 0.05)))
      (should (stringp result))
      (should (string-match-p "<svg" result)))))

(ert-deftest litgraph-it-project-directory-compiles-lit ()
  ;; With no prebuilt packs, `litigate engine' compiles lit/ in the
  ;; project directory: here the timeline fixture's lit/examples.
  (litgraph-it
    (let ((process-environment
           (seq-remove (lambda (e) (string-match-p "\\`LITGRAPH_\\(PACKS\\|SCENARIOS\\)=" e))
                       process-environment))
          (litgraph-packs-dir nil) (litgraph-scenarios-dir nil)
          (litgraph-project-directory
           (expand-file-name "fixtures/timeline-project" litgraph-test-dir)))
      (let ((response (litgraph-request-sync
                       (litgraph-obj "op" (litgraph-obj "op" "describe")))))
        (should (equal (mapcar (lambda (pack) (litgraph-get pack 'id))
                               (litgraph-get response 'result 'packs))
                       '("frcp-rule12-example"))))
      (let ((response (litgraph-request-sync
                       (litgraph-obj "packs" ["frcp-rule12-example"]
                                     "op" (litgraph-obj "op" "solve")))))
        (should (litgraph-get response 'ok))))))

(defun litgraph-it--timeline-binary ()
  "Return the installed `litigate' executable, or nil if it is absent."
  (executable-find (or (plist-get litgraph-timeline-command :program)
                       "litigate")))

(ert-deftest litgraph-it-timeline-binary-default-is-litigation-engine ()
  (should (equal (plist-get litgraph-timeline-command :program) "litigate")))

(ert-deftest litgraph-it-timeline-binary-presence-checks-configured-program ()
  (let ((calls nil))
    (cl-letf (((symbol-function 'executable-find)
               (lambda (program) (push program calls) nil)))
      (should-not (litgraph-it--timeline-binary))
      (should (equal calls '("litigate"))))))

(ert-deftest litgraph-it-timeline-renders-real-op ()
  ;; The example project comes from litgraph's cases/examples and
  ;; lit/examples.  LITGRAPH_TIMELINE_FILE names another
  ;; .case or .lit file to try.
  (let ((binary (litgraph-it--timeline-binary)))
    (unless binary (litgraph-test-skip-or-fail "litigate binary not installed"))
    (let ((litgraph-executable binary)
          (litgraph--cache (make-hash-table :test #'equal))
          (litgraph-scenario nil) (litgraph-packs nil) (litgraph-params nil)
          (litgraph-default-scenario nil) (litgraph-default-packs nil))
    (let* ((file (or (getenv "LITGRAPH_TIMELINE_FILE")
                     (expand-file-name "fixtures/timeline-project/cases/examples/acme.case"
                                       litgraph-test-dir)))
           (buffer (litgraph-timeline file)))
      (unwind-protect
          (with-current-buffer buffer
            (let ((deadline (+ (float-time) 30)))
              (while (and (not litgraph-timeline--doc)
                          (not litgraph-timeline--error)
                          (< (float-time) deadline))
                (accept-process-output nil 0.05)))
            (should-not litgraph-timeline--error)
            (should (equal (litgraph-get litgraph-timeline--doc 'schema)
                           litgraph-timeline-schema))
            (should (string-match-p "^Timeline  " (buffer-string)))
            ;; Every state and layer class the op emits is one the schema has.
            (dolist (row (litgraph-get litgraph-timeline--doc 'rows))
              (should (member (litgraph-get row 'status 'state)
                              litgraph-timeline-states))
              (should (member (litgraph-get row 'layer)
                              litgraph-timeline-layer-classes))
              (when (equal (litgraph-get row 'status 'state) "unknown")
                (should (litgraph-get row 'status 'reason))))
            (dolist (layer (litgraph-get litgraph-timeline--doc 'layers))
              (should (member (litgraph-get layer 'class)
                              litgraph-timeline-layer-classes))))
        (kill-buffer buffer))))))

(provide 'litgraph-integration-test)

;;; litgraph-integration-test.el ends here
