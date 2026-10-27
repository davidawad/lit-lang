;;; litgraph-integration-test.el --- Tests against the real litigate binary -*- lexical-binding: t; -*-

;; SPDX-License-Identifier: GPL-3.0-or-later

;;; Commentary:

;; Tests run the real `litigate' binary (LITIGATE_BIN, else the one on
;; PATH): `litigate run' for engine ops, `litigate timeline' for the
;; timeline.  They skip when it is missing unless LITGRAPH_IT_REQUIRE=1,
;; as `just emacs-test' sets.  Every op compiles its matter from this
;; repository's `cases/' and `lit/'.

;;; Code:

(require 'litgraph-test-helper)
(require 'litgraph)
(require 'litgraph-view)
(require 'litgraph-graph)
(require 'litgraph-org)
(require 'ob-litgraph)
(require 'litgraph-timeline)

(defconst litgraph-it-repository
  (expand-file-name "../../../" litgraph-test-dir)
  "This repository's root: its `cases/' and `lit/' are the targets.")

(defconst litgraph-it-matter "cofc-1498-patent-case"
  "The matter the engine tests run on: `cases/NAME.case'.")

(defmacro litgraph-it (&rest body)
  "Run BODY against the real binary with a clean session, or skip/fail."
  (declare (indent 0))
  `(let ((binary (litgraph-test-real-binary)))
     (unless binary (litgraph-test-skip-or-fail "litigate binary not installed"))
     (let ((litgraph-executable binary)
           (litgraph--cache (make-hash-table :test #'equal))
           (litgraph-project-directory litgraph-it-repository)
           (litgraph-target nil) (litgraph-params nil)
           (litgraph-default-target nil))
       ,@body)))

(ert-deftest litgraph-it-runs-the-built-litigate ()
  ;; The representative op, end to end: the request becomes `litigate
  ;; run TARGET --op OP --set ...', the envelope is unwrapped, and the
  ;; engine's answer reflects the parameter override.
  (litgraph-it
    (let* ((args nil)
           (litgraph-process-function
            (lambda (a input callback)
              (setq args a)
              (litgraph--spawn a input callback)))
           (base (litgraph-request-sync
                  (litgraph-build-request "solve" litgraph-it-matter)))
           (dear (litgraph-request-sync
                  (litgraph-build-request "solve" litgraph-it-matter
                                          '(("rate" . 5000))))))
      (should (equal (car args) "run"))
      (should (equal (cadr args)
                     (expand-file-name (format "cases/%s.case" litgraph-it-matter)
                                       litgraph-it-repository)))
      (should (member "rate=5000" args))
      (dolist (response (list base dear))
        (should (litgraph-get response 'ok))
        (should (equal (litgraph-get response 'op) "solve"))
        (should (numberp (litgraph-get response 'result 'value)))
        (should (litgraph-get response 'result 'best_line)))
      (should (= 5000 (litgraph-get dear 'provenance 'params 'rate)))
      (should (< (litgraph-get dear 'result 'value)
                 (litgraph-get base 'result 'value)))
      ;; What the view shows is the engine response, not the envelope.
      (should (string-prefix-p "{" litgraph--last-raw))
      (should-not (string-match-p "\"op\":\"run\"" litgraph--last-raw)))))

(ert-deftest litgraph-it-compile-error-is-reported ()
  (litgraph-it
    (let* ((dir (make-temp-file "lg-it" t))
           (file (expand-file-name "broken.case" dir)))
      (unwind-protect
          (progn
            (write-region "case broken\nuses nope ^0.1.0\n" nil file)
            (let ((err (should-error (litgraph-request-sync
                                      (litgraph-build-request "solve" file)))))
              (should (string-match-p "L1006: package `nope' is not loaded\\|L0001"
                                      (error-message-string err)))))
        (delete-directory dir t)))))

(ert-deftest litgraph-it-describe-discovers-ops ()
  (litgraph-it
    (setq litgraph-target litgraph-it-matter)
    (litgraph-with-describe #'ignore)
    ;; Bound the wait by `litgraph-timeout', as `litgraph-request-sync'
    ;; does: a debug build compiles the matter on every call.
    (let ((deadline (+ (float-time) litgraph-timeout)))
      (while (and (< (float-time) deadline) (not (litgraph--cached-describe)))
        (accept-process-output nil 0.05)))
    (should (member "solve" (litgraph-op-names)))
    (should (member "deadlines" (litgraph-op-names)))
    (should (assoc "cofc" (litgraph-pack-candidates)))
    (should (assoc (format "cases/%s.case" litgraph-it-matter)
                   (let ((default-directory litgraph-it-repository))
                     (litgraph-target-candidates))))
    (dolist (op '("chain" "sweep" "deadlines"))
      (should (litgraph-op-arg-specs (litgraph--cached-describe) op)))))

(ert-deftest litgraph-it-solve-chain-render ()
  (litgraph-it
    (dolist (op '("solve" "chain" "tornado" "simulate" "pareto" "structure"))
      (let* ((request (litgraph-build-request op litgraph-it-matter))
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
                                           litgraph-it-matter))
                  :type 'error)))

(ert-deftest litgraph-it-graph-dot-is-valid-graphviz ()
  (litgraph-it
    (let* ((graph (litgraph-get (litgraph-request-sync
                                 (litgraph-build-request "graph" litgraph-it-matter))
                                'result))
           (solve (litgraph-request-sync (litgraph-build-request "solve" litgraph-it-matter)))
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

(ert-deftest litgraph-it-babel-block ()
  (litgraph-it
    (let ((org-confirm-babel-evaluate nil)
          (litgraph-executable (litgraph-test-real-binary)))
      (with-temp-buffer
        (org-mode)
        (insert "#+begin_src litgraph :target " litgraph-it-matter " :var rate=900\nchain\n#+end_src\n")
        (goto-char (point-min))
        (org-babel-execute-src-block)
        (should (string-match-p "| Outcome" (buffer-string)))
        (should (string-match-p "| Utility" (buffer-string)))))))

(ert-deftest litgraph-it-async-does-not-block ()
  (litgraph-it
    (let ((done nil) (ticks 0))
      (litgraph-request (litgraph-build-request "simulate" litgraph-it-matter)
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
                         litgraph-it-matter)
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
                                 (litgraph-build-request "graph" litgraph-it-matter))
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
  ;; `litigate run' finds the packages a matter uses in the project's
  ;; lit/: here the timeline fixture project's lit/examples.
  (litgraph-it
    (let ((litgraph-project-directory
           (expand-file-name "fixtures/timeline-project" litgraph-test-dir)))
      (let ((response (litgraph-request-sync
                       (litgraph-build-request "packs" "examples/acme"))))
        (should (equal (mapcar (lambda (pack) (litgraph-get pack 'id))
                               (litgraph-get response 'result))
                       '("frcp-rule12-example"))))
      (let ((response (litgraph-request-sync
                       (litgraph-build-request "solve" "lit/examples"))))
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
          (litgraph-target nil) (litgraph-params nil)
          (litgraph-default-target nil))
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
