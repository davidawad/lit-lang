;;; litgraph-timeline-test.el --- Timeline buffer tests (process layer stubbed) -*- lexical-binding: t; -*-

;; SPDX-License-Identifier: GPL-3.0-or-later

;;; Commentary:

;; The timeline JSON contract is frozen in litgraph (docs/litigate/
;; timeline.v1.schema.json, TIMELINE.md).  test/fixtures/ holds copies of
;; the schema and of its example document, taken from litgraph main
;; 76005c3; these tests render the example and check the renderer against
;; the schema's enums.

;;; Code:

(require 'litgraph-test-helper)
(require 'litgraph-timeline)
(require 'litgraph-menu)

(defvar litigate-mode-map)
(defvar litigate-ts-mode-map)

(defun litgraph-timeline-test-doc ()
  "The example timeline, parsed."
  (litgraph-test-fixture-response "timeline-ttab-opposition"))

(defun litgraph-timeline-test-ids (rows)
  "Ids of ROWS."
  (mapcar (lambda (row) (litgraph-get row 'id)) rows))

(defun litgraph-timeline-test-line (pattern)
  "The text of the first line of the buffer matching PATTERN."
  (save-excursion
    (goto-char (point-min))
    (re-search-forward pattern)
    (buffer-substring-no-properties (line-beginning-position)
                                    (line-end-position))))

(defun litgraph-timeline-test-schema-enum (&rest path)
  "The enum at PATH (strings) under the schema's $defs."
  (let ((schema (litgraph-parse-strict (litgraph-test-fixture "timeline.v1.schema"))))
    (append (gethash "enum" (seq-reduce (lambda (table key) (gethash key table))
                                        path (gethash "$defs" schema)))
            nil)))

(defmacro litgraph-timeline-test-in-dir (var &rest body)
  "Run BODY with VAR bound to a fresh directory holding the example's files."
  (declare (indent 1))
  `(let ((,var (file-name-as-directory (make-temp-file "lg-tl" t))))
     (unwind-protect
         (progn
           (make-directory (expand-file-name "cases" ,var))
           (make-directory (expand-file-name "lit/ttab" ,var) t)
           (with-temp-file (expand-file-name
                            "cases/example-ttab-opposition.case" ,var)
             (dotimes (i 12) (insert (format "line %d\n" (1+ i))))
             (insert "// rests on 37 C.F.R. § 2.106(a)\n"))
           (with-temp-file (expand-file-name "lit/ttab/answer.lit" ,var)
             (dotimes (i 45) (insert (format "rule line %d\n" (1+ i)))))
           ,@body)
       (dolist (buffer (buffer-list))
         (when (string-prefix-p ,var (or (buffer-file-name buffer) ""))
           (with-current-buffer buffer (set-buffer-modified-p nil))
           (kill-buffer buffer)))
       (dolist (buffer (litgraph-timeline--buffers)) (kill-buffer buffer))
       (delete-directory ,var t))))

(defun litgraph-timeline-test-open (dir)
  "Open the timeline of the example case under DIR; return its buffer."
  (litgraph-timeline (expand-file-name "cases/example-ttab-opposition.case" dir)))


;;;; Contract

(ert-deftest litgraph-timeline-states-match-schema ()
  (should (equal litgraph-timeline-states
                 (litgraph-timeline-test-schema-enum "status" "properties" "state")))
  (should (equal (sort (mapcar #'car litgraph-timeline--state-faces) #'string<)
                 (sort (copy-sequence litgraph-timeline-states) #'string<))))

(ert-deftest litgraph-timeline-layer-classes-match-schema ()
  (should (equal litgraph-timeline-layer-classes
                 (litgraph-timeline-test-schema-enum "layer_class")))
  (should (equal (mapcar #'car litgraph-timeline-layer-faces)
                 litgraph-timeline-layer-classes)))

(ert-deftest litgraph-timeline-schema-version-matches ()
  (let ((schema (litgraph-parse-strict (litgraph-test-fixture "timeline.v1.schema"))))
    (should (equal litgraph-timeline-schema
                   (gethash "const" (gethash "schema" (gethash "properties" schema)))))
    (should (equal litgraph-timeline-schema
                   (litgraph-get (litgraph-timeline-test-doc) 'schema)))))


;;;; Order, worlds, graph

(ert-deftest litgraph-timeline-rows-in-date-order ()
  ;; r5 (a fork with no date) sorts with r4, which triggered it, and r9
  ;; (an event with no date) with r5; on 2026-04-13 the docket's r3 comes
  ;; before the alternate world's r7.
  (should (equal (litgraph-timeline-test-ids
                  (litgraph-timeline-sorted-rows (litgraph-timeline-test-doc)))
                 '("r1" "r2" "r4" "r5" "r9" "r6" "r3" "r7" "r8"))))

(ert-deftest litgraph-timeline-world-rows-inherit-up-to-fork ()
  (let ((doc (litgraph-timeline-test-doc)))
    (should (equal (litgraph-timeline-test-ids (litgraph-timeline-world-rows doc "w0"))
                   '("r1" "r2" "r4" "r5" "r3")))
    ;; r7 re-emits r3 (same key): it replaces r3 in w1's view.
    (should (equal (litgraph-timeline-test-ids (litgraph-timeline-world-rows doc "w1"))
                   '("r1" "r2" "r4" "r5" "r6" "r7" "r8")))
    (should (equal (litgraph-timeline-test-ids (litgraph-timeline-world-rows doc "w2"))
                   '("r1" "r2" "r4" "r5" "r9" "r3")))))

(defun litgraph-timeline-test-strip (doc field)
  "DOC with FIELD removed from every row."
  (cons (cons 'rows (mapcar (lambda (row) (assq-delete-all field (copy-sequence row)))
                            (litgraph-get doc 'rows)))
        (assq-delete-all 'rows (copy-sequence doc))))

(ert-deftest litgraph-timeline-world-rows-replace-by-key-only ()
  (let* ((doc (litgraph-timeline-test-doc))
         ;; r7 keeps r3's key but names another span and label: still a
         ;; re-emission.
         (doc (cons (cons 'rows
                          (mapcar (lambda (row)
                                    (if (equal (litgraph-get row 'id) "r7")
                                        (append '((source) (label . "x"))
                                                (assq-delete-all
                                                 'label
                                                 (assq-delete-all
                                                  'source (copy-sequence row))))
                                      row))
                                  (litgraph-get doc 'rows)))
                    (assq-delete-all 'rows (copy-sequence doc)))))
    (should (equal (litgraph-timeline-test-ids (litgraph-timeline-world-rows doc "w1"))
                   '("r1" "r2" "r4" "r5" "r6" "r7" "r8")))))

(ert-deftest litgraph-timeline-world-rows-without-keys-match-spans ()
  ;; A document without keys: r7 has r3's source span, so replaces it.
  (let ((doc (litgraph-timeline-test-strip (litgraph-timeline-test-doc) 'key)))
    (should (equal (litgraph-timeline-test-ids (litgraph-timeline-world-rows doc "w1"))
                   '("r1" "r2" "r4" "r5" "r6" "r7" "r8")))
    ;; Without spans either, kind and label: r7's label differs from r3's.
    (should (equal (litgraph-timeline-test-ids
                    (litgraph-timeline-world-rows
                     (litgraph-timeline-test-strip doc 'source) "w1"))
                   '("r1" "r2" "r4" "r5" "r6" "r3" "r7" "r8")))))

(ert-deftest litgraph-timeline-graph-has-one-lane-per-world ()
  (let* ((doc (litgraph-timeline-test-doc))
         (graph (litgraph-timeline-graph doc (litgraph-timeline-sorted-rows doc))))
    (should (equal (mapcar (lambda (g) (substring-no-properties (car g))) graph)
                   '("*    " "*    " "*    " "+    " "| | *"
                     "| *  " "o |  " "  o  " "  o  ")))
    ;; Only the fork row has a connector, branching to both worlds.
    (should (equal (mapcar (lambda (g) (and (cdr g) (substring-no-properties (cdr g))))
                           graph)
                   '(nil nil nil "|\\-\\ " nil nil nil nil nil)))))

(ert-deftest litgraph-timeline-graph-left-branch ()
  ;; A world listed before its parent branches to the left.
  (let* ((doc (litgraph-timeline-test-doc))
         (worlds (litgraph-get doc 'worlds))
         (doc (cons (cons 'worlds (list (nth 1 worlds) (nth 0 worlds)))
                    (assq-delete-all 'worlds (copy-sequence doc))))
         (rows (seq-remove (lambda (r) (equal (litgraph-get r 'world) "w2"))
                           (litgraph-timeline-sorted-rows doc)))
         (graph (litgraph-timeline-graph doc rows)))
    (should (equal (substring-no-properties (cdr (nth 3 graph))) " /|  "))))


;;;; Diff

(defun litgraph-timeline-test-pairs (diff)
  "Ids of the :changed pairs of DIFF."
  (mapcar (lambda (p) (list (litgraph-get (car p) 'id) (litgraph-get (cdr p) 'id)))
          (plist-get diff :changed)))

(ert-deftest litgraph-timeline-diff-pairs-rows-by-key ()
  (let ((diff (litgraph-timeline-diff-worlds (litgraph-timeline-test-doc) "w1" "w2")))
    (should (equal (litgraph-timeline-test-ids (plist-get diff :common))
                   '("r1" "r2" "r4" "r5")))
    ;; r7 re-emits r3: one entry, changed.
    (should (equal (litgraph-timeline-test-pairs diff) '(("r7" "r3"))))
    (should (equal (litgraph-timeline-test-ids (plist-get diff :only-a)) '("r6" "r8")))
    (should (equal (litgraph-timeline-test-ids (plist-get diff :only-b)) '("r9")))))

(ert-deftest litgraph-timeline-diff-pairs-by-key-not-span ()
  ;; Same span, different key: two entries, not one changed.
  (let* ((doc (litgraph-timeline-test-doc))
         ;; Use two independent worlds so parent inheritance cannot mask a
         ;; bad span-based pairing.  r7 and r3 have the same kind and source
         ;; span, but are distinct causes with distinct stable keys.
         (worlds (mapcar (lambda (world)
                           (append '((parent . nil) (fork_row . nil))
                                   (assq-delete-all 'fork_row
                                     (assq-delete-all 'parent
                                       (copy-sequence world)))))
                         (seq-filter (lambda (world)
                                       (member (litgraph-get world 'id) '("w1" "w2")))
                                     (litgraph-get doc 'worlds))))
         (rows (seq-filter (lambda (row)
                             (member (litgraph-get row 'id) '("r3" "r7")))
                           (litgraph-get doc 'rows)))
         (rows (mapcar (lambda (row)
                         (cond
                          ((equal (litgraph-get row 'id) "r3")
                           (cons '(world . "w2")
                                 (assq-delete-all 'world (copy-sequence row))))
                          ((equal (litgraph-get row 'id) "r7")
                           (cons '(key . "other")
                                 (assq-delete-all 'key (copy-sequence row))))
                          (t row)))
                       rows))
         (doc (cons (cons 'rows rows)
                    (cons (cons 'worlds worlds)
                          (assq-delete-all 'rows
                            (assq-delete-all 'worlds (copy-sequence doc))))))
         (diff (litgraph-timeline-diff-worlds doc "w1" "w2")))
    (should-not (plist-get diff :changed))
    (should (equal (litgraph-timeline-test-ids (plist-get diff :only-a))
                   '("r7")))
    (should (equal (litgraph-timeline-test-ids (plist-get diff :only-b)) '("r3")))))

(ert-deftest litgraph-timeline-diff-with-parent ()
  (let ((diff (litgraph-timeline-diff-worlds (litgraph-timeline-test-doc) "w0" "w2")))
    (should (equal (length (plist-get diff :common)) 5))
    (should-not (plist-get diff :only-a))
    (should (equal (litgraph-timeline-test-ids (plist-get diff :only-b)) '("r9"))))
  (let ((diff (litgraph-timeline-diff-worlds (litgraph-timeline-test-doc) "w1" "w0")))
    (should (equal (litgraph-timeline-test-pairs diff) '(("r7" "r3"))))
    (should (equal (litgraph-timeline-test-ids (plist-get diff :only-a)) '("r6" "r8")))
    (should-not (plist-get diff :only-b))))


;;;; Rendering

(ert-deftest litgraph-timeline-render-shows-rows ()
  (with-temp-buffer
    (litgraph-timeline-mode)
    (litgraph-timeline-render (litgraph-timeline-test-doc))
    (let ((text (buffer-substring-no-properties (point-min) (point-max))))
      (should (string-match-p "^Timeline  cases/example-ttab-opposition.case  (case)" text))
      (should (string-match-p "As of 2026-04-08   Known at everything recorded" text))
      (should (string-match-p "w1   reset granted  grant: Board resets the answer date" text))
      (should (string-match-p "\\[x\\] ttab +base_rule +Regulation" text))
      (should (string-match-p "\\[x\\] order-reset-1 +case_order +CaseOrder" text)))
    ;; Each row: date, kind, status, layer class, label, cite.
    (should (equal (litgraph-timeline-test-line "Notice of opposition")
                   (concat "*      2026-02-20  event     none        base_rule   "
                           "Notice of opposition filed  [37 C.F.R. § 2.101]")))
    ;; A fork with no ruling: unknown, with the reason.
    (should (equal (litgraph-timeline-test-line "fork +unknown")
                   (concat "+      —           fork      unknown     base_rule   "
                           "Ruling on the motion — no ruling recorded as of 2026-04-08")))
    (should (string-match-p "^|\\\\-\\\\   +w1 reset granted, w2 reset denied$"
                            (litgraph-timeline-test-line "reset granted, w2")))
    (should (string-match-p "^  o    2026-04-13  deadline  superseded  base_rule   Answer due (original designation) — reset by order-reset-1"
                            (litgraph-timeline-test-line "original designation")))
    (should (string-match-p "^  o    2026-05-13  deadline  open        case_order  Answer due (as reset)"
                            (litgraph-timeline-test-line "as reset")))
    (should (equal (litgraph-get (get-text-property
                                  (save-excursion (goto-char (point-min))
                                                  (search-forward "as reset")
                                                  (point))
                                  'litgraph-timeline-row)
                                 'id)
                   "r8"))))

(ert-deftest litgraph-timeline-render-uncomputable-deadline ()
  ;; A deadline the engine cannot compute is a row with due null and
  ;; state unknown; it is shown with its reason, never left out.
  (let* ((doc (litgraph-timeline-test-doc))
         (row (copy-tree (nth 2 (litgraph-get doc 'rows)))))
    (setf (alist-get 'due row) nil
          (alist-get 'id row) "r10"
          (alist-get 'label row) "Reply due"
          (alist-get 'status row) '((state . "unknown")
                                    (reason . "trigger date not recorded")))
    (setf (alist-get 'rows doc) (append (litgraph-get doc 'rows) (list row)))
    (with-temp-buffer
      (litgraph-timeline-mode)
      (litgraph-timeline-render doc)
      (should (string-match-p
               "—           deadline  unknown     base_rule   Reply due — trigger date not recorded"
               (litgraph-timeline-test-line "Reply due"))))))

(ert-deftest litgraph-timeline-render-every-state-and-class ()
  ;; Statuses and layer classes print exactly as the schema spells them,
  ;; and every state with a reason shows it.
  (let* ((doc (litgraph-timeline-test-doc))
         (template (nth 2 (litgraph-get doc 'rows)))
         (rows (cl-loop for state in litgraph-timeline-states
                        for class in (append litgraph-timeline-layer-classes
                                             litgraph-timeline-layer-classes)
                        for i from 0
                        collect (let ((row (copy-tree template)))
                                  (setf (alist-get 'id row) (format "s%d" i)
                                        (alist-get 'label row) (format "row-%s" state)
                                        (alist-get 'layer row) class
                                        (alist-get 'status row)
                                        `((state . ,state)
                                          (reason . ,(format "why-%s" state))))
                                  row))))
    (setf (alist-get 'rows doc) rows)
    (with-temp-buffer
      (litgraph-timeline-mode)
      (litgraph-timeline-render doc)
      (cl-loop for state in litgraph-timeline-states
               for class in (append litgraph-timeline-layer-classes
                                    litgraph-timeline-layer-classes)
               do (let ((line (litgraph-timeline-test-line
                               (format "row-%s\\b" state))))
                    (should (string-match-p (format "  %s +%s +row-%s — why-%s"
                                                    state class state state)
                                            line)))))))

(ert-deftest litgraph-timeline-render-diagnostics ()
  (let ((doc (litgraph-timeline-test-doc)))
    (setf (alist-get 'diagnostics doc)
          '(((code . "unknown-trigger") (severity . "warning")
             (message . "no date for Served(answer)")
             (source . ((file . "cases/x.case") (start . 1) (end . 2)
                        (line . 3) (column . 4))))))
    (with-temp-buffer
      (litgraph-timeline-mode)
      (litgraph-timeline-render doc)
      (should (string-match-p "warning +unknown-trigger: no date for Served(answer)  cases/x.case:3:4"
                              (litgraph-timeline-test-line "unknown-trigger"))))))

(ert-deftest litgraph-timeline-diff-buffer ()
  (let ((doc (litgraph-timeline-test-doc)))
    (with-temp-buffer
      (litgraph-timeline-diff-mode)
      (litgraph-timeline-render-diff doc "w1" "w2")
      (let ((text (buffer-substring-no-properties (point-min) (point-max))))
        (should (string-match-p "^Diff  w1 reset granted  ⟷  w2 reset denied" text))
        (should (string-match-p "^Common (4)" text))
        (should (string-match-p "^Changed (1)\n- 2026-04-13  deadline  superseded .*\n\\+ 2026-04-13  deadline  open " text))
        (should (string-match-p "^Only in w1 (2)" text))
        (should (string-match-p "^Only in w2 (1)\n\\+ —           event     none .*Board denies" text))))))


;;;; Running the op

(ert-deftest litgraph-timeline-args ()
  (should (equal (litgraph-timeline--args "/x/a.case")
                 '("timeline" "--format" "json" "/x/a.case")))
  (should (equal (litgraph-timeline--args "/x/a.case" "2026-04-08" "2026-01-01"
                                          '("ttab" "o1") "w1")
                 '("timeline" "--format" "json" "/x/a.case" "--as-of" "2026-04-08"
                   "--known-at" "2026-01-01" "--layers" "ttab,o1" "--world" "w1")))
  (should (equal (litgraph-timeline--args "/x/a.case" nil nil '("-ttab" "-o1") nil)
                 '("timeline" "--format" "json" "/x/a.case" "--layers" "-ttab,-o1")))
  ;; No layer off: no --layers at all (the op rejects an empty one).
  (should (equal (litgraph-timeline--args "/x/a.case" nil nil nil nil)
                 '("timeline" "--format" "json" "/x/a.case")))
  (let ((litgraph-timeline-command
         '(:args ("tl" "--format" "json" file) :as-of "--at" :world nil)))
    (should (equal (litgraph-timeline--args "/x/a.case" "2026-04-08" nil t "w1")
                   '("tl" "--format" "json" "/x/a.case" "--at" "2026-04-08")))))

(ert-deftest litgraph-timeline-decode ()
  (should (car (litgraph-timeline--decode
                0 (litgraph-test-fixture "timeline-ttab-opposition") "")))
  (should (car (litgraph-timeline--decode
                0 (concat "{\"ok\":true,\"result\":"
                          (litgraph-test-fixture "timeline-ttab-opposition") "}") "")))
  (let ((no-op (litgraph-timeline--decode 2 "" "error: unrecognized subcommand 'timeline'")))
    (should-not (car no-op))
    (should (string-match-p "unrecognized subcommand" (cdr no-op)))
    (should (string-match-p "litgraph-timeline-command" (cdr no-op))))
  (let ((engine (litgraph-timeline--decode 2 (litgraph-test-fixture "error") "")))
    (should-not (car engine))
    (should (stringp (cdr engine))))
  (should (string-match-p "unsupported timeline schema"
                          (cdr (litgraph-timeline--decode
                                0 "{\"schema\": \"litigate.timeline/2\"}" ""))))
  (should (string-match-p "timed out" (cdr (litgraph-timeline--decode 'timeout "" "")))))

(ert-deftest litgraph-timeline-opens-and-calls-op ()
  (litgraph-test-with-stub
    (litgraph-timeline-test-in-dir dir
      (let* ((file (expand-file-name "cases/example-ttab-opposition.case" dir))
             (buffer (litgraph-timeline-test-open dir)))
        (should (equal (car (car litgraph-test-calls))
                       (list "timeline" "--format" "json" file)))
        ;; `litigate timeline', not `litigate engine timeline'.
        (should (equal (car litgraph-test-commands) '("litigate")))
        (with-current-buffer buffer
          (should (derived-mode-p 'litgraph-timeline-mode))
          (should (string-match-p "Ruling on the motion" (buffer-string)))
          (should (eq (keymap-lookup nil "RET") #'litgraph-timeline-visit))
          (should (eq (keymap-lookup nil "=") #'litgraph-timeline-diff))
          (should (eq (keymap-lookup nil "l") #'litgraph-timeline-toggle-layer))
          (should (eq (keymap-lookup nil "t") #'litgraph-timeline-set-dates))
          (should (eq (keymap-lookup nil "g") #'litgraph-timeline-refresh)))
        ;; Opening it again reuses the buffer.
        (should (eq (litgraph-timeline-test-open dir) buffer))))))

(ert-deftest litgraph-timeline-failure-is-shown ()
  (litgraph-test-with-stub
    (litgraph-timeline-test-in-dir dir
      (let ((litgraph-timeline-command '(:args ("no-such-op" file))))
        (with-current-buffer (litgraph-timeline-test-open dir)
          (should (string-match-p "unexpected call" (buffer-string)))
          (should-error (litgraph-timeline-diff "w0" "w1") :type 'user-error))))))

(ert-deftest litgraph-timeline-toggle-layer-reruns ()
  (litgraph-test-with-stub
    (litgraph-timeline-test-in-dir dir
      (with-current-buffer (litgraph-timeline-test-open dir)
        (litgraph-timeline-toggle-layer "order-reset-1")
        (should (equal (last (car (car litgraph-test-calls)) 2)
                       '("--layers" "-order-reset-1")))
        ;; The stub ignores --layers; the rows the layer decided are hidden.
        (should-not (string-match-p "as reset" (buffer-string)))
        (should (string-match-p "\\[ \\] order-reset-1" (buffer-string)))
        (litgraph-timeline-toggle-layer "order-reset-1")
        ;; Every layer on again: no --layers.
        (should (equal (length (car (car litgraph-test-calls))) 4))
        (should (string-match-p "as reset" (buffer-string)))
        ;; On a layer line, l toggles that layer.
        (goto-char (point-min))
        (search-forward "[x] ttab")
        (call-interactively #'litgraph-timeline-toggle-layer)
        (should (equal (last (car (car litgraph-test-calls)) 2)
                       '("--layers" "-ttab")))
        (should (string-match-p "\\[ \\] ttab" (buffer-string)))
        (litgraph-timeline-toggle-layer "order-reset-1")
        (should (member (car (last (car (car litgraph-test-calls))))
                        '("-ttab,-order-reset-1" "-order-reset-1,-ttab")))))))

(ert-deftest litgraph-timeline-set-dates-and-world ()
  (litgraph-test-with-stub
    (litgraph-timeline-test-in-dir dir
      (with-current-buffer (litgraph-timeline-test-open dir)
        (litgraph-timeline-set-dates "2026-05-01" "2026-04-01")
        (should (equal (cddddr (car (car litgraph-test-calls)))
                       '("--as-of" "2026-05-01" "--known-at" "2026-04-01")))
        (cl-letf (((symbol-function 'read-string)
                   (lambda (&rest _) "2026-05-02")))
          (call-interactively #'litgraph-timeline-set-dates))
        (should (equal (cddddr (car (car litgraph-test-calls)))
                       '("--as-of" "2026-05-02" "--known-at" "2026-05-02")))
        (cl-letf (((symbol-function 'read-string) (lambda (&rest _) "")))
          (call-interactively #'litgraph-timeline-set-dates))
        (should (equal (length (car (car litgraph-test-calls))) 4))
        (cl-letf (((symbol-function 'read-string) (lambda (&rest _) "May 2")))
          (should-error (call-interactively #'litgraph-timeline-set-dates)
                        :type 'user-error))
        (litgraph-timeline-set-world "w2")
        (should (equal (last (car (car litgraph-test-calls)) 2) '("--world" "w2")))
        (should-not (string-match-p "as reset" (buffer-string)))
        (should (string-match-p "deadline  open .*  Answer due (designated" (buffer-string)))
        (should (string-match-p "Board denies the motion (hypothetical)" (buffer-string)))
        (litgraph-timeline-set-world nil)
        (should (string-match-p "as reset" (buffer-string)))))))

(ert-deftest litgraph-timeline-diff-command ()
  (litgraph-test-with-stub
    (litgraph-timeline-test-in-dir dir
      (with-current-buffer (litgraph-timeline-test-open dir)
        (goto-char (point-min))
        (search-forward "original designation")
        ;; Point is on a w1 row: the defaults are w1 and its parent, w0.
        (cl-letf (((symbol-function 'completing-read)
                   (lambda (_prompt _choices _pred _req _init _hist default)
                     default)))
          (let ((diff (call-interactively #'litgraph-timeline-diff)))
            (with-current-buffer diff
              (should (derived-mode-p 'litgraph-timeline-diff-mode))
              (should (string-match-p "^Diff  w1 reset granted  ⟷  w0 docket"
                                      (buffer-string)))
              (should (string-match-p "^Changed (1)" (buffer-string)))
              (should (string-match-p "^Only in w1 (2)" (buffer-string)))
              ;; RET works in the diff buffer too.
              (goto-char (point-min))
              (search-forward "Notice of opposition")
              (litgraph-timeline-visit)
              (should (string-suffix-p "cases/example-ttab-opposition.case"
                                       buffer-file-name))
              (should (= (line-number-at-pos) 9)))))))))


;;;; Visiting sources

(ert-deftest litgraph-timeline-visit-source-span ()
  (litgraph-test-with-stub
    (litgraph-timeline-test-in-dir dir
      (with-current-buffer (litgraph-timeline-test-open dir)
        (goto-char (point-min))
        (search-forward "Answer due (designated")
        (litgraph-timeline-visit)
        (should (equal buffer-file-name (expand-file-name "lit/ttab/answer.lit" dir)))
        (should (= (line-number-at-pos) 41))))))

(ert-deftest litgraph-timeline-visit-cite-without-span ()
  (litgraph-test-with-stub
    (litgraph-timeline-test-in-dir dir
      (let ((timeline (litgraph-timeline-test-open dir)))
        (with-current-buffer timeline
          ;; r6 has no span; its cite is found in the case file.
          (goto-char (point-min))
          (search-forward "Board order resets")
          (litgraph-timeline-visit)
          (should (string-suffix-p "example-ttab-opposition.case" buffer-file-name))
          (should (looking-at-p "37 C\\.F\\.R\\. § 2\\.106(a)")))
        (with-current-buffer timeline
          ;; A cite-function, when set, is asked first.
          (let* ((seen nil)
                 (litgraph-timeline-cite-function
                  (lambda (cite row) (setq seen (list cite (litgraph-get row 'id))))))
            (goto-char (point-min))
            (search-forward "Board order resets")
            (litgraph-timeline-visit)
            (should (equal seen '("37 C.F.R. § 2.106(a)" "r6"))))
          ;; A world line goes to its fork row.
          (goto-char (point-min))
          (search-forward "reset denied")
          (litgraph-timeline-visit)
          (should (string-match-p "Ruling on the motion"
                                  (buffer-substring (line-beginning-position)
                                                    (line-end-position)))))))))

(ert-deftest litgraph-timeline-row-navigation ()
  (with-temp-buffer
    (litgraph-timeline-mode)
    (litgraph-timeline-render (litgraph-timeline-test-doc))
    (goto-char (point-min))
    (litgraph-timeline-next-row 1)
    (should (equal (litgraph-timeline--row-id-at-point) "r1"))
    (litgraph-timeline-next-row 4)
    (should (equal (litgraph-timeline--row-id-at-point) "r9"))
    (litgraph-timeline-previous-row 1)
    (should (equal (litgraph-timeline--row-id-at-point) "r5"))
    (litgraph-timeline-next-row 99)
    (should (equal (litgraph-timeline--row-id-at-point) "r8"))))

(ert-deftest litgraph-timeline-refreshes-after-save ()
  (litgraph-test-with-stub
    (litgraph-timeline-test-in-dir dir
      (litgraph-timeline-test-open dir)
      (let ((calls (length litgraph-test-calls)))
        ;; Saving the case file, or a .lit file a row comes from, refreshes.
        (with-current-buffer (find-file-noselect
                              (expand-file-name "cases/example-ttab-opposition.case" dir))
          (insert "x")
          (save-buffer))
        (should (= (length litgraph-test-calls) (1+ calls)))
        (with-current-buffer (find-file-noselect
                              (expand-file-name "lit/ttab/answer.lit" dir))
          (insert "x")
          (save-buffer))
        (should (= (length litgraph-test-calls) (+ 2 calls)))
        (let ((litgraph-timeline-refresh-on-save nil))
          (with-current-buffer (find-file-noselect
                                (expand-file-name "lit/ttab/answer.lit" dir))
            (insert "x")
            (save-buffer)))
        (should (= (length litgraph-test-calls) (+ 2 calls)))))))


;;;; Entry points

(ert-deftest litgraph-timeline-bound-in-litigate-mode ()
  (let ((litigate-mode-map (make-sparse-keymap))
        (litigate-ts-mode-map (make-sparse-keymap)))
    (unwind-protect
        (progn
          (provide 'litigate-mode)
          (should (eq (keymap-lookup litigate-mode-map "C-c C-t")
                      #'litgraph-timeline))
          (provide 'litigate-ts-mode)
          (should (eq (keymap-lookup litigate-ts-mode-map "C-c C-t")
                      #'litgraph-timeline)))
      (setq features (delq 'litigate-mode (delq 'litigate-ts-mode features)))))
  (let ((map (make-sparse-keymap))
        (litgraph-timeline-litigate-key nil))
    (litgraph-timeline-bind-litigate-key map)
    (should-not (keymap-lookup map "C-c"))))

(ert-deftest litgraph-timeline-in-menu-and-command-map ()
  (should (transient-get-suffix 'litgraph-menu-transient "T"))
  (should (eq (keymap-lookup litgraph-command-map "T") #'litgraph-timeline)))

(provide 'litgraph-timeline-test)

;;; litgraph-timeline-test.el ends here
