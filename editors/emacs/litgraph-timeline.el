;;; litgraph-timeline.el --- Timeline of a .case or .lit file, magit-log style -*- lexical-binding: t; -*-

;; Copyright (C) 2026 David Awad

;; Author: David Awad <me@davidaw.ad>
;; Maintainer: David Awad <me@davidaw.ad>
;; Keywords: tools, convenience, law
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

;; `litgraph-timeline' shows the timeline of a `.case' or `.lit' file the
;; way `magit-log --graph' shows history: one row per event, deadline or
;; fork in date order, and a graph column with one lane per world
;; (alternate timeline).  A fork row is where worlds diverge: an assumed
;; order, a grant or denial, a hypothetical decision.  Each row shows the
;; date, the kind, the status (`met', `open', `late', `unknown' with its
;; reason, ...), the class of the authority layer that decided it, the
;; label and the rule cite.
;;
;; The buffer only renders the JSON of the litgraph `timeline' op
;; (schema `litigate.timeline/1', see docs/litigate/TIMELINE.md in
;; litgraph); `litgraph-timeline-command' says how to call it.
;;
;; Keys in the timeline buffer:
;;
;;   RET  jump to the row's source span, or to its cited text
;;   =    diff two worlds
;;   l    toggle a layer on or off
;;   t    set as_of / known_at
;;   w    show one world only (or all again)
;;   g    refresh (also done after saving the file)
;;   j    raw JSON
;;
;; In `litigate-mode' (github.com/davidawad/litigate-mode.el) the key in
;; `litgraph-timeline-litigate-key' opens the timeline of the buffer.

;;; Code:

(require 'cl-lib)
(require 'seq)
(require 'subr-x)
(require 'text-property-search)
(require 'litgraph)
(require 'litgraph-view)

(declare-function json-pretty-print-buffer "json")
(defvar litigate-mode-map)
(defvar litigate-ts-mode-map)


;;;; Customization

(defgroup litgraph-timeline nil
  "Timeline buffers for .case and .lit files."
  :group 'litgraph
  :prefix "litgraph-timeline-")

(defcustom litgraph-timeline-command
  '(:program "litigate"
    :args ("timeline" "--format" "json" file)
    :as-of "--as-of"
    :known-at "--known-at"
    :layers "--layers"
    :world "--world")
  "How to call the timeline op.
A plist:

  :program   the executable (defaults to `litigate').
  :args      the arguments; the symbol `file' stands for the absolute
             name of the .case or .lit file.
  :as-of     flag followed by the as_of date, when one is set.
  :known-at  flag followed by the known_at date, when one is set.
  :layers    flag followed by the comma-separated ids of the enabled
             layers, passed only once a layer has been toggled.
  :world     flag followed by the world id, when one world is shown.

A flag set to nil is never passed."
  :type '(plist :key-type symbol :value-type sexp))

(defcustom litgraph-timeline-display-action nil
  "Action passed to `pop-to-buffer' when showing a timeline buffer."
  :type 'sexp)

(defcustom litgraph-timeline-refresh-on-save t
  "Non-nil means saving a file refreshes the timelines built from it."
  :type 'boolean)

;;;###autoload
(defcustom litgraph-timeline-litigate-key "C-c C-t"
  "Key that opens the timeline in `litigate-mode' buffers, or nil for none.
A string accepted by `keymap-set'.  It takes effect when
`litigate-mode' (or the older `litigate-ts-mode') is loaded."
  :type '(choice (const :tag "None" nil) key)
  :group 'litgraph-timeline)

(defcustom litgraph-timeline-glyphs
  '(("event" . "*") ("deadline" . "o") ("fork" . "+"))
  "Graph glyph of each row kind.  Other kinds use \"*\"."
  :type '(alist :key-type string :value-type string))

(defcustom litgraph-timeline-cite-function nil
  "Function that shows the text a cite refers to, or nil.
Called with the cite string and the row (an alist) when RET is pressed
on a row without a source span.  It returns non-nil when it showed
something; otherwise the cite is searched for in the timeline's file."
  :type '(choice (const :tag "Search the file" nil) function))

(defface litgraph-timeline-met '((t :inherit litgraph-view-good))
  "Face of the `met' status.")

(defface litgraph-timeline-open '((t :inherit font-lock-function-name-face))
  "Face of the `open' status.")

(defface litgraph-timeline-late '((t :inherit litgraph-view-bad))
  "Face of the `late', `lapsed' and `breached' statuses.")

(defface litgraph-timeline-unknown '((t :inherit warning))
  "Face of the `unknown' status and of reasons.")

(defface litgraph-timeline-superseded
  '((t :inherit litgraph-view-note :strike-through t))
  "Face of the `superseded' status.")

(defface litgraph-timeline-none '((t :inherit litgraph-view-note))
  "Face of the `none' status (events).")

(defface litgraph-timeline-date '((t :inherit font-lock-constant-face))
  "Face of row dates.")

(defface litgraph-timeline-fork '((t :inherit font-lock-keyword-face :weight bold))
  "Face of fork rows' labels.")

(defcustom litgraph-timeline-layer-faces
  '(("base_rule" . font-lock-constant-face)
    ("local_rule" . font-lock-type-face)
    ("standing_order" . font-lock-variable-name-face)
    ("general_order" . font-lock-builtin-face)
    ("emergency_declaration" . warning)
    ("case_order" . font-lock-function-name-face)
    ("decision" . font-lock-keyword-face))
  "Face of each layer class."
  :type '(alist :key-type string :value-type face))

(defcustom litgraph-timeline-lane-faces
  '(font-lock-keyword-face font-lock-function-name-face font-lock-string-face
    font-lock-type-face font-lock-constant-face font-lock-variable-name-face)
  "Faces of the graph lanes, one per world, reused in turn."
  :type '(repeat face))


;;;; The schema's closed sets

(defconst litgraph-timeline-schema "litigate.timeline/1"
  "The `schema' value of the timeline JSON this file renders.")

(defconst litgraph-timeline-states
  '("met" "open" "late" "lapsed" "breached" "unknown" "superseded" "none")
  "Every `status.state' of the timeline schema.")

(defconst litgraph-timeline-layer-classes
  '("base_rule" "local_rule" "standing_order" "general_order"
    "emergency_declaration" "case_order" "decision")
  "Every layer `class' of the timeline schema.")

(defconst litgraph-timeline--state-faces
  '(("met" . litgraph-timeline-met)
    ("open" . litgraph-timeline-open)
    ("late" . litgraph-timeline-late)
    ("lapsed" . litgraph-timeline-late)
    ("breached" . litgraph-timeline-late)
    ("unknown" . litgraph-timeline-unknown)
    ("superseded" . litgraph-timeline-superseded)
    ("none" . litgraph-timeline-none))
  "Face of each status.")

(defun litgraph-timeline--state-face (state)
  "Face of the status STATE (a string the schema may not know yet)."
  (or (cdr (assoc state litgraph-timeline--state-faces))
      'litgraph-timeline-unknown))

(defun litgraph-timeline--layer-face (class)
  "Face of the layer CLASS."
  (or (cdr (assoc class litgraph-timeline-layer-faces)) 'default))


;;;; Reading the document

(defun litgraph-timeline--rows-by-id (doc)
  "Hash table of DOC's rows by id."
  (let ((table (make-hash-table :test #'equal)))
    (dolist (row (litgraph-get doc 'rows) table)
      (puthash (litgraph-get row 'id) row table))))

(defun litgraph-timeline--world (doc id)
  "The world of DOC with ID, or nil."
  (seq-find (lambda (w) (equal (litgraph-get w 'id) id))
            (litgraph-get doc 'worlds)))

(defun litgraph-timeline--world-ids (doc)
  "Ids of DOC's worlds, then of any world only rows mention."
  (let ((ids (mapcar (lambda (w) (litgraph-get w 'id))
                     (litgraph-get doc 'worlds))))
    (dolist (row (litgraph-get doc 'rows))
      (let ((id (litgraph-get row 'world)))
        (unless (member id ids) (setq ids (append ids (list id))))))
    ids))

(defun litgraph-timeline--world-ancestors (doc id)
  "Ids of the worlds above world ID in DOC, nearest first."
  (let ((seen nil)
        (parent (litgraph-get (litgraph-timeline--world doc id) 'parent)))
    (while (and parent (not (member parent seen)))
      (push parent seen)
      (setq parent (litgraph-get (litgraph-timeline--world doc parent)
                                 'parent)))
    (nreverse seen)))

(defun litgraph-timeline--world-depth (doc id)
  "Number of worlds above world ID in DOC."
  (length (litgraph-timeline--world-ancestors doc id)))

(defun litgraph-timeline-row-date (row)
  "The date shown for ROW: when it happened, or when it is due."
  (or (litgraph-get row 'date) (litgraph-get row 'due)))

(defun litgraph-timeline-sorted-rows (doc)
  "DOC's rows in date order.
A row without a date sorts with the row that triggered it, else with the
row before it in the document.  No row of a world sorts before the
world's fork row; on equal dates parent worlds come first, then document
order."
  (let* ((rows (litgraph-get doc 'rows))
         (by-id (litgraph-timeline--rows-by-id doc))
         (keys (make-hash-table :test #'equal))
         (previous "")
         (key-of nil))
    (setq key-of
          (lambda (row seen)
            (let ((id (litgraph-get row 'id)))
              (or (gethash id keys)
                  (litgraph-timeline-row-date row)
                  (when-let* ((trigger (litgraph-get row 'trigger))
                              (parent (gethash trigger by-id))
                              ((not (member trigger seen))))
                    (funcall key-of parent (cons id seen)))))))
    (dolist (row rows)
      (let ((key (or (funcall key-of row nil) previous)))
        (puthash (litgraph-get row 'id) key keys)
        (setq previous key)))
    ;; A world's rows never come before its fork row.
    (let ((fork-key
           (lambda (world-id)
             (let ((best ""))
               (dolist (id (cons world-id
                                 (litgraph-timeline--world-ancestors
                                  doc world-id)))
                 (when-let* ((fork (litgraph-get
                                    (litgraph-timeline--world doc id)
                                    'fork_row))
                             (key (gethash fork keys)))
                   (when (string< best key) (setq best key))))
               best))))
      (dolist (row rows)
        (let ((id (litgraph-get row 'id))
              (floor (funcall fork-key (litgraph-get row 'world))))
          (when (string< (gethash id keys) floor)
            (puthash id floor keys)))))
    (let ((index 0) decorated)
      (dolist (row rows)
        (push (list (gethash (litgraph-get row 'id) keys)
                    (litgraph-timeline--world-depth
                     doc (litgraph-get row 'world))
                    index row)
              decorated)
        (cl-incf index))
      (mapcar
       #'cadddr
       (sort (nreverse decorated)
             (lambda (a b)
               (cond ((not (equal (car a) (car b))) (string< (car a) (car b)))
                     ((/= (cadr a) (cadr b)) (< (cadr a) (cadr b)))
                     (t (< (caddr a) (caddr b))))))))))

(defun litgraph-timeline--predecessors (doc row-id worlds)
  "Ids of rows reaching ROW-ID through DOC's edges in WORLDS, with ROW-ID."
  (let ((found (list row-id))
        (queue (list row-id)))
    (while queue
      (let ((target (pop queue)))
        (dolist (edge (litgraph-get doc 'edges))
          (when (and (equal (litgraph-get edge 'to) target)
                     (member (litgraph-get edge 'world) worlds))
            (let ((from (litgraph-get edge 'from)))
              (unless (member from found)
                (push from found)
                (push from queue)))))))
    found))

(defun litgraph-timeline-world-rows (doc world-id &optional sorted)
  "Rows of world WORLD-ID in DOC: its own, plus its ancestors' up to the fork.
The inherited rows are those that lead to the fork row through the
ancestors' edges, the fork row included; when the ancestors have no
edges, those not after the fork row in date order.  SORTED is DOC's rows
in date order, when already computed.  Return them in date order."
  (let* ((sorted (or sorted (litgraph-timeline-sorted-rows doc)))
         (world (litgraph-timeline--world doc world-id))
         (parent (litgraph-get world 'parent))
         (fork (litgraph-get world 'fork_row))
         (inherited
          (when (and parent fork)
            (let* ((parent-rows (litgraph-timeline-world-rows doc parent sorted))
                   (parent-ids (mapcar (lambda (r) (litgraph-get r 'id))
                                       parent-rows))
                   (worlds (cons parent (litgraph-timeline--world-ancestors
                                         doc parent)))
                   (reach (litgraph-timeline--predecessors doc fork worlds)))
              (if (cdr reach)
                  (seq-filter (lambda (id) (member id reach)) parent-ids)
                (let ((before t) ids)
                  (dolist (row sorted (nreverse ids))
                    (let ((id (litgraph-get row 'id)))
                      (when (and before (member id parent-ids)) (push id ids))
                      (when (equal id fork) (setq before nil))))))))))
    (seq-filter (lambda (row)
                  (or (equal (litgraph-get row 'world) world-id)
                      (member (litgraph-get row 'id) inherited)))
                sorted)))


;;;; Diffing worlds

(defun litgraph-timeline--match-key (row)
  "Key matching ROW across two worlds: its kind and span, else its label."
  (let ((span (litgraph-get row 'source)))
    (list (litgraph-get row 'kind)
          (if span
              (list (litgraph-get span 'file) (litgraph-get span 'start)
                    (litgraph-get span 'end))
            (litgraph-get row 'label)))))

(defun litgraph-timeline-diff-worlds (doc a b)
  "Compare worlds A and B of DOC.
Return a plist: :common, the rows both have; :changed, pairs
\(ROW-A . ROW-B) of rows for the same entry (same kind and source
span, or same label); :only-a and :only-b, the rest.  Rows are in date order."
  (let* ((sorted (litgraph-timeline-sorted-rows doc))
         (rows-a (litgraph-timeline-world-rows doc a sorted))
         (rows-b (litgraph-timeline-world-rows doc b sorted))
         (ids-b (mapcar (lambda (r) (litgraph-get r 'id)) rows-b))
         (common (seq-filter (lambda (r) (member (litgraph-get r 'id) ids-b))
                             rows-a))
         (rest-a (seq-difference rows-a common))
         (rest-b (seq-difference rows-b common))
         changed only-a)
    (dolist (row rest-a)
      (if-let* ((key (litgraph-timeline--match-key row))
                (other (seq-find (lambda (r)
                                   (equal (litgraph-timeline--match-key r) key))
                                 rest-b)))
          (progn (push (cons row other) changed)
                 (setq rest-b (delq other rest-b)))
        (push row only-a)))
    (list :common common :changed (nreverse changed)
          :only-a (nreverse only-a) :only-b rest-b)))


;;;; Graph column

(defun litgraph-timeline--lane-face (lane)
  "Face of graph LANE."
  (if litgraph-timeline-lane-faces
      (nth (mod lane (length litgraph-timeline-lane-faces))
           litgraph-timeline-lane-faces)
    'default))

(defun litgraph-timeline-graph (doc rows)
  "Graph column for ROWS (DOC's rows to show, in order).
Return a list with one entry per row: (GRAPH . CONNECTOR), where GRAPH
is the row's graph string and CONNECTOR is the line drawn under a fork
row toward the worlds that leave it (nil for other rows)."
  (let* ((world-ids (litgraph-timeline--world-ids doc))
         (lanes (length world-ids))
         (width (max 1 (1- (* 2 lanes))))
         (positions (make-hash-table :test #'equal))
         (spans nil))
    (seq-do-indexed (lambda (row i) (puthash (litgraph-get row 'id) i positions))
                    rows)
    ;; Each world's lane is active from its first row (or just after its
    ;; fork row) to its last row.
    (setq spans
          (seq-map-indexed
           (lambda (id _lane)
             (let* ((world (litgraph-timeline--world doc id))
                    (own (delq nil (mapcar
                                    (lambda (row)
                                      (and (equal (litgraph-get row 'world) id)
                                           (gethash (litgraph-get row 'id)
                                                    positions)))
                                    rows)))
                    (fork (and (litgraph-get world 'parent)
                               (gethash (litgraph-get world 'fork_row)
                                        positions))))
               (if own
                   (cons (if fork (1+ fork) (apply #'min own))
                         (apply #'max own))
                 (cons 1 0))))
           world-ids))
    (cl-labels
        ((lane-of (id) (or (seq-position world-ids id) 0))
         (active (lane pos) (let ((span (nth lane spans)))
                              (<= (car span) pos (cdr span))))
         (cell (text lane) (propertize text 'face
                                       (litgraph-timeline--lane-face lane))))
      (seq-map-indexed
       (lambda (row pos)
         (let ((graph (make-vector width " "))
               (own (lane-of (litgraph-get row 'world)))
               (connector nil))
           (dotimes (lane lanes)
             (cond ((= lane own)
                    (aset graph (* 2 lane)
                          (cell (or (cdr (assoc (litgraph-get row 'kind)
                                                litgraph-timeline-glyphs))
                                    "*")
                                lane)))
                   ((active lane pos) (aset graph (* 2 lane) (cell "|" lane)))))
           (let ((children
                  (seq-filter
                   (lambda (lane)
                     (let ((world (litgraph-timeline--world
                                   doc (nth lane world-ids))))
                       (and (equal (litgraph-get world 'fork_row)
                                   (litgraph-get row 'id))
                            (litgraph-get world 'parent)
                            (= (car (nth lane spans)) (1+ pos))
                            (<= (car (nth lane spans)) (cdr (nth lane spans))))))
                   (number-sequence 0 (1- lanes)))))
             (when children
               (let ((line (make-vector width " ")))
                 (dotimes (lane lanes)
                   (when (or (and (active lane pos) (active lane (1+ pos)))
                             (and (= lane own) (active lane (1+ pos))))
                     (aset line (* 2 lane) (cell "|" lane))))
                 (when (equal (aref line (* 2 own)) " ")
                   (aset line (* 2 own) (cell "|" own)))
                 (dolist (child children)
                   (let* ((right (> child own))
                          (slash (if right (1- (* 2 child)) (1+ (* 2 child)))))
                     (aset line slash (cell (if right "\\" "/") child))
                     (let ((from (min (* 2 own) slash))
                           (to (max (* 2 own) slash)))
                       (cl-loop for col from (1+ from) below to
                                when (equal (aref line col) " ")
                                do (aset line col (cell "-" child))))))
                 (setq connector (apply #'concat (append line nil))))))
           (cons (apply #'concat (append graph nil)) connector)))
       rows))))


;;;; Rendering

(defvar-local litgraph-timeline--file nil
  "Absolute name of the .case or .lit file this timeline is built from.")
(defvar-local litgraph-timeline--doc nil "The parsed timeline shown.")
(defvar-local litgraph-timeline--raw nil "The raw JSON of the timeline shown.")
(defvar-local litgraph-timeline--error nil "The last error, or nil.")
(defvar-local litgraph-timeline--as-of nil "The as_of date asked for, or nil.")
(defvar-local litgraph-timeline--known-at nil
  "The known_at date asked for, or nil.")
(defvar-local litgraph-timeline--world-filter nil
  "Id of the only world shown, or nil for all.")
(defvar-local litgraph-timeline--disabled-layers t
  "Ids of the layers switched off, or t when no layer was toggled yet.")

(defun litgraph-timeline--disabled (doc)
  "Ids of DOC's layers that are off, here or in the op's answer."
  (if (listp litgraph-timeline--disabled-layers)
      litgraph-timeline--disabled-layers
    (delq nil (mapcar (lambda (layer)
                        (unless (litgraph-get layer 'enabled)
                          (litgraph-get layer 'id)))
                      (litgraph-get doc 'layers)))))

(defun litgraph-timeline-visible-rows (doc &optional disabled world)
  "DOC's rows to show, in date order.
Leave out rows decided by a layer in DISABLED (ids), and, when WORLD is
non-nil, rows outside that world."
  (let* ((sorted (litgraph-timeline-sorted-rows doc))
         (rows (if world (litgraph-timeline-world-rows doc world sorted) sorted)))
    (seq-remove (lambda (row)
                  (member (car (last (litgraph-get row 'provenance))) disabled))
                rows)))

(defun litgraph-timeline--pad (string width)
  "STRING padded with spaces to WIDTH."
  (concat string (make-string (max 0 (- width (string-width string))) ?\s)))

(defun litgraph-timeline--row-text (row layer-width)
  "One line of text (no graph) for ROW, layer column LAYER-WIDTH wide."
  (let* ((state (or (litgraph-get row 'status 'state) "none"))
         (reason (litgraph-get row 'status 'reason))
         (kind (or (litgraph-get row 'kind) ""))
         (class (or (litgraph-get row 'layer) ""))
         (cite (litgraph-get row 'cite))
         (date (litgraph-timeline-row-date row)))
    (concat
     (propertize (litgraph-timeline--pad (or date "—") 10)
                 'face (if date 'litgraph-timeline-date 'litgraph-view-note))
     "  " (litgraph-timeline--pad kind 8)
     "  " (propertize (litgraph-timeline--pad state 10)
                      'face (litgraph-timeline--state-face state))
     "  " (propertize (litgraph-timeline--pad class layer-width)
                      'face (litgraph-timeline--layer-face class))
     "  " (propertize (or (litgraph-get row 'label) "")
                      'face (if (equal kind "fork") 'litgraph-timeline-fork
                              'default))
     (if reason
         (concat " — " (propertize reason 'face
                                   (if (equal state "unknown")
                                       'litgraph-timeline-unknown
                                     (litgraph-timeline--state-face state))))
       "")
     (if cite (concat "  " (propertize (format "[%s]" cite)
                                       'face 'litgraph-view-note))
       ""))))

(defun litgraph-timeline--layer-width (doc)
  "Width of the layer column for DOC."
  (apply #'max 5 (mapcar (lambda (row) (string-width (or (litgraph-get row 'layer) "")))
                         (litgraph-get doc 'rows))))

(defun litgraph-timeline--choice-text (choice)
  "Describe a world's CHOICE."
  (if (not choice) ""
    (concat (propertize (or (litgraph-get choice 'kind) "") 'face 'bold)
            ": " (or (litgraph-get choice 'label) "")
            (if-let* ((p (litgraph-get choice 'probability)))
                (format "  p=%s" (litgraph-view-format-number p))
              "")
            (if-let* ((cite (litgraph-get choice 'cite)))
                (propertize (format "  [%s]" cite) 'face 'litgraph-view-note)
              ""))))

(defun litgraph-timeline--span-text (span)
  "FILE:LINE:COLUMN for SPAN."
  (format "%s:%s:%s" (litgraph-get span 'file) (litgraph-get span 'line)
          (litgraph-get span 'column)))

(defun litgraph-timeline--heading (text)
  "Insert the heading TEXT."
  (insert (propertize text 'face 'litgraph-view-title) "\n"))

(defun litgraph-timeline--insert-header (doc disabled)
  "Insert DOC's file, dates, worlds, layers and diagnostics.
DISABLED is the ids of the layers switched off."
  (let ((by-id (litgraph-timeline--rows-by-id doc))
        (world-ids (litgraph-timeline--world-ids doc)))
    (insert (propertize "Timeline  " 'face 'litgraph-view-title)
            (propertize (or (litgraph-get doc 'source 'file) "?") 'face 'bold)
            (format "  (%s)\n" (or (litgraph-get doc 'source 'kind) "?")))
    (insert (propertize "As of " 'face 'litgraph-view-label)
            (or (litgraph-get doc 'as_of) "?")
            (propertize "   Known at " 'face 'litgraph-view-label)
            (or (litgraph-get doc 'known_at) "everything recorded")
            (if litgraph-timeline--world-filter
                (concat (propertize "   World " 'face 'litgraph-view-label)
                        litgraph-timeline--world-filter)
              "")
            "\n\n")
    (litgraph-timeline--heading "Worlds")
    (dolist (world (litgraph-get doc 'worlds))
      (let* ((id (litgraph-get world 'id))
             (lane (or (seq-position world-ids id) 0))
             (fork (litgraph-get world 'fork_row)))
        (insert
         (propertize
          (concat "  " (propertize "|" 'face (litgraph-timeline--lane-face lane))
                  " " (litgraph-timeline--pad id 4) " "
                  (propertize (or (litgraph-get world 'label) "") 'face 'bold)
                  (if-let* ((choice (litgraph-get world 'choice)))
                      (concat "  " (litgraph-timeline--choice-text choice))
                    "")
                  (if fork
                      (propertize
                       (format "  forks from %s at %s %s"
                               (litgraph-get world 'parent) fork
                               (or (litgraph-get (gethash fork by-id) 'label) ""))
                       'face 'litgraph-view-note)
                    ""))
          'litgraph-timeline-world id)
         "\n")))
    (insert "\n")
    (litgraph-timeline--heading "Layers")
    (dolist (layer (litgraph-get doc 'layers))
      (let* ((id (litgraph-get layer 'id))
             (on (not (member id disabled)))
             (class (or (litgraph-get layer 'class) ""))
             (from (litgraph-get layer 'effective 'from))
             (to (litgraph-get layer 'effective 'to)))
        (insert
         (propertize
          (concat "  " (if on "[x] " "[ ] ")
                  (litgraph-timeline--pad id 14) " "
                  (propertize (litgraph-timeline--pad class 21)
                              'face (litgraph-timeline--layer-face class))
                  " " (litgraph-timeline--pad (or (litgraph-get layer 'authority) "")
                                              13)
                  " " (or (litgraph-get layer 'label) "")
                  (if-let* ((cite (litgraph-get layer 'cite)))
                      (propertize (format "  [%s]" cite) 'face 'litgraph-view-note)
                    "")
                  (if (or from to)
                      (propertize (format "  %s → %s" (or from "…") (or to "…"))
                                  'face 'litgraph-view-note)
                    ""))
          'litgraph-timeline-layer id
          'face (unless on 'litgraph-view-note))
         "\n")))
    (when-let* ((diagnostics (litgraph-get doc 'diagnostics)))
      (insert "\n")
      (litgraph-timeline--heading "Diagnostics")
      (dolist (diagnostic diagnostics)
        (let ((severity (or (litgraph-get diagnostic 'severity) "note"))
              (span (litgraph-get diagnostic 'source)))
          (insert
           (propertize
            (concat "  "
                    (propertize (litgraph-timeline--pad severity 8)
                                'face (pcase severity
                                        ("error" 'litgraph-view-bad)
                                        ("warning" 'warning)
                                        (_ 'litgraph-view-note)))
                    (or (litgraph-get diagnostic 'code) "") ": "
                    (or (litgraph-get diagnostic 'message) "")
                    (if span
                        (propertize (concat "  " (litgraph-timeline--span-text span))
                                    'face 'litgraph-view-note)
                      ""))
            'litgraph-timeline-span span)
           "\n"))))
    (insert "\n")))

(defun litgraph-timeline--world-labels (doc row)
  "Text naming the worlds that leave the fork ROW of DOC."
  (mapconcat (lambda (world)
               (format "%s %s" (litgraph-get world 'id)
                       (or (litgraph-get world 'label) "")))
             (seq-filter (lambda (world)
                           (and (litgraph-get world 'parent)
                                (equal (litgraph-get world 'fork_row)
                                       (litgraph-get row 'id))))
                         (litgraph-get doc 'worlds))
             ", "))

(defun litgraph-timeline-render (doc)
  "Render the timeline DOC into the current buffer."
  (let* ((inhibit-read-only t)
         (disabled (litgraph-timeline--disabled doc))
         (rows (litgraph-timeline-visible-rows
                doc disabled litgraph-timeline--world-filter))
         (graphs (litgraph-timeline-graph doc rows))
         (graph-width (if graphs (string-width (caar graphs)) 1))
         (layer-width (litgraph-timeline--layer-width doc)))
    (erase-buffer)
    (litgraph-timeline--insert-header doc disabled)
    (insert (propertize
             (concat (make-string graph-width ?\s) "  "
                     (litgraph-timeline--pad "Date" 10) "  "
                     (litgraph-timeline--pad "Kind" 8) "  "
                     (litgraph-timeline--pad "Status" 10) "  "
                     (litgraph-timeline--pad "Layer" layer-width) "  "
                     "Event or deadline")
             'face 'litgraph-view-header)
            "\n")
    (if (not rows)
        (insert (propertize "  (no rows)\n" 'face 'litgraph-view-note))
      (cl-mapc
       (lambda (row graph)
         (insert (propertize
                  (concat (car graph) "  "
                          (litgraph-timeline--row-text row layer-width))
                  'litgraph-timeline-row row)
                 "\n")
         (when (cdr graph)
           (insert (litgraph-timeline--pad (cdr graph) graph-width) "  "
                   (propertize (litgraph-timeline--world-labels doc row)
                               'face 'litgraph-view-note)
                   "\n")))
       rows graphs))
    (set-buffer-modified-p nil)))


;;;; Running the op

(defun litgraph-timeline--args (file &optional as-of known-at layers world)
  "Arguments of the timeline op for FILE.
AS-OF, KNOWN-AT and WORLD are strings or nil.  LAYERS is the list of
enabled layer ids (or that list already joined with commas), or nil or t
to leave layers to the op."
  (let ((spec litgraph-timeline-command))
    (cl-flet ((option (key value)
                (when-let* ((flag (plist-get spec key))
                            ((stringp value)))
                  (list flag value))))
      (append (mapcar (lambda (arg) (if (eq arg 'file) file arg))
                      (plist-get spec :args))
              (option :as-of as-of)
              (option :known-at known-at)
              (option :layers (if (consp layers) (string-join layers ",") layers))
              (option :world world)))))

(defun litgraph-timeline--decode (exit stdout stderr)
  "Turn a finished run (EXIT, STDOUT, STDERR) into (OK . DOC-OR-MESSAGE)."
  (let ((parsed (and (not (memq exit '(timeout signal)))
                     (not (string-empty-p (string-trim stdout)))
                     (ignore-errors (litgraph-parse stdout)))))
    (cond
     ((and (consp parsed) (equal (litgraph-get parsed 'schema)
                                 litgraph-timeline-schema))
      (cons t parsed))
     ((and (consp parsed) (assq 'schema parsed))
      (cons nil (format "unsupported timeline schema %S (this buffer reads %s)"
                        (litgraph-get parsed 'schema) litgraph-timeline-schema)))
     ((and (consp parsed) (assq 'ok parsed))
      (cons nil (cadr (litgraph--decode exit stdout stderr))))
     (t (cons nil
              (concat (litgraph--describe-failure exit stdout stderr)
                      (if (integerp exit)
                          (format " (does the binary have the timeline op? \
See `litgraph-timeline-command': %s)"
                                  (string-join
                                   (seq-filter #'stringp
                                               (plist-get litgraph-timeline-command
                                                          :args))
                                   " "))
                        "")))))))

(defun litgraph-timeline--layers-arg (doc)
  "Enabled layer ids for the op, joined with commas, or t.
t means no layer was toggled.  DOC is the timeline shown, if any."
  (if (listp litgraph-timeline--disabled-layers)
      (string-join
       (seq-remove (lambda (id) (member id litgraph-timeline--disabled-layers))
                   (mapcar (lambda (layer) (litgraph-get layer 'id))
                           (litgraph-get doc 'layers)))
       ",")
    t))

(defun litgraph-timeline--row-id-at-point ()
  "Id of the row at point, or nil."
  (litgraph-get (get-text-property (point) 'litgraph-timeline-row) 'id))

(defun litgraph-timeline--goto-row (id)
  "Move to the line of row ID; return non-nil when found."
  (let ((match (save-excursion
                 (goto-char (point-min))
                 (text-property-search-forward
                  'litgraph-timeline-row id
                  (lambda (value row) (equal value (litgraph-get row 'id)))))))
    (when match (goto-char (prop-match-beginning match)))))

(defun litgraph-timeline--show (buffer doc raw)
  "Show DOC (with RAW JSON text) in the timeline BUFFER, keeping point."
  (with-current-buffer buffer
    (let ((row (litgraph-timeline--row-id-at-point))
          (line (line-number-at-pos))
          (windows (get-buffer-window-list buffer nil t)))
      (setq litgraph-timeline--doc doc
            litgraph-timeline--raw raw
            litgraph-timeline--error nil)
      (litgraph-timeline-render doc)
      (unless (and row (litgraph-timeline--goto-row row))
        (goto-char (point-min))
        (forward-line (1- line)))
      (dolist (window windows) (set-window-point window (point))))))

(defun litgraph-timeline--fail (buffer message)
  "Report MESSAGE for the timeline BUFFER."
  (when (buffer-live-p buffer)
    (with-current-buffer buffer
      (setq litgraph-timeline--error message)
      (unless litgraph-timeline--doc
        (let ((inhibit-read-only t))
          (erase-buffer)
          (insert (propertize "Timeline  " 'face 'litgraph-view-title)
                  (or litgraph-timeline--file "") "\n\n"
                  (propertize message 'face 'litgraph-view-bad) "\n")))))
  (litgraph--report-error message))

(defun litgraph-timeline-refresh ()
  "Run the timeline op again and redraw this buffer."
  (interactive)
  (unless litgraph-timeline--file (user-error "Not in a timeline buffer"))
  (let* ((buffer (current-buffer))
         (program (plist-get litgraph-timeline-command :program))
         (litgraph-executable (or program litgraph-executable))
         (default-directory (file-name-directory litgraph-timeline--file))
         (args (litgraph-timeline--args
                litgraph-timeline--file litgraph-timeline--as-of
                litgraph-timeline--known-at
                (litgraph-timeline--layers-arg litgraph-timeline--doc)
                litgraph-timeline--world-filter)))
    (setq mode-line-process ":run")
    (funcall litgraph-process-function args nil
             (lambda (exit stdout stderr)
               (when (buffer-live-p buffer)
                 (with-current-buffer buffer (setq mode-line-process nil))
                 (let ((decoded (litgraph-timeline--decode exit stdout stderr)))
                   (if (car decoded)
                       (litgraph-timeline--show buffer (cdr decoded) stdout)
                     (litgraph-timeline--fail buffer (cdr decoded)))))))))

(defun litgraph-timeline--buffers (&optional file)
  "Live timeline buffers, only those built from FILE when it is non-nil."
  (seq-filter (lambda (buffer)
                (with-current-buffer buffer
                  (and (derived-mode-p 'litgraph-timeline-mode)
                       litgraph-timeline--file
                       (or (not file)
                           (file-equal-p litgraph-timeline--file file)))))
              (buffer-list)))

(defun litgraph-timeline--mentions-p (file)
  "Non-nil if this timeline was built from FILE or has a row from it."
  (or (file-equal-p litgraph-timeline--file file)
      (seq-some (lambda (row)
                  (when-let* ((span (litgraph-get row 'source))
                              (name (litgraph-timeline--resolve
                                     (litgraph-get span 'file))))
                    (file-equal-p name file)))
                (litgraph-get litgraph-timeline--doc 'rows))))

(defun litgraph-timeline--after-save ()
  "Refresh the timelines that rest on the file just saved."
  (when (and litgraph-timeline-refresh-on-save buffer-file-name)
    (let ((file buffer-file-name))
      (dolist (buffer (litgraph-timeline--buffers))
        (with-current-buffer buffer
          (when (litgraph-timeline--mentions-p file)
            (litgraph-timeline-refresh)))))))

;;;###autoload
(defun litgraph-timeline (file)
  "Show the timeline of the .case or .lit FILE.
Interactively, FILE is the current buffer's file, else read."
  (interactive
   (list (if (and buffer-file-name
                  (string-match-p "\\.\\(case\\|lit\\)\\'" buffer-file-name))
             buffer-file-name
           (read-file-name "Timeline of .case/.lit file: " nil nil t))))
  (setq file (expand-file-name file))
  (when-let* ((source (find-buffer-visiting file)))
    (when (and (buffer-modified-p source)
               (y-or-n-p (format "Save %s first? " (buffer-name source))))
      (with-current-buffer source (save-buffer))))
  (add-hook 'after-save-hook #'litgraph-timeline--after-save)
  (let ((buffer (or (car (litgraph-timeline--buffers file))
                    (generate-new-buffer
                     (format "*litgraph-timeline: %s*"
                             (file-name-nondirectory file))))))
    (with-current-buffer buffer
      (unless (derived-mode-p 'litgraph-timeline-mode)
        (litgraph-timeline-mode)
        (let ((inhibit-read-only t))
          (insert "Running the timeline op on " file "…\n")))
      (setq litgraph-timeline--file file)
      (litgraph-timeline-refresh))
    (pop-to-buffer buffer litgraph-timeline-display-action)
    buffer))


;;;; Commands

(defun litgraph-timeline--require ()
  "Signal unless a timeline is shown."
  (unless litgraph-timeline--doc
    (user-error "No timeline shown%s"
                (if litgraph-timeline--error
                    (concat ": " litgraph-timeline--error)
                  ""))))

(defun litgraph-timeline--resolve (file)
  "Absolute name of FILE, a span's file, or nil when it does not exist.
A relative name is tried against the timeline's directory and its parents."
  (cond
   ((not file) nil)
   ((file-name-absolute-p file) (and (file-exists-p file) file))
   (t (let ((dir (and litgraph-timeline--file
                      (file-name-directory litgraph-timeline--file)))
            found)
        (while (and dir (not found))
          (let ((candidate (expand-file-name file dir)))
            (if (file-exists-p candidate)
                (setq found candidate)
              (let ((up (file-name-directory (directory-file-name dir))))
                (setq dir (unless (equal up dir) up))))))
        found))))

(defun litgraph-timeline--visit-span (span)
  "Show the source SPAN in another window."
  (let ((file (litgraph-timeline--resolve (litgraph-get span 'file))))
    (unless file
      (user-error "Cannot find %s" (litgraph-get span 'file)))
    (pop-to-buffer (find-file-noselect file) t)
    (goto-char (point-min))
    (forward-line (1- (or (litgraph-get span 'line) 1)))
    (move-to-column (1- (or (litgraph-get span 'column) 1)))
    (let ((start (litgraph-get span 'start))
          (end (litgraph-get span 'end)))
      (when (and start end (> end start) (fboundp 'pulse-momentary-highlight-region))
        (pulse-momentary-highlight-region
         (point) (min (point-max) (+ (point) (- end start))))))))

(defun litgraph-timeline--visit-cite (cite row)
  "Show the text CITE (of ROW) refers to."
  (or (and litgraph-timeline-cite-function
           (funcall litgraph-timeline-cite-function cite row))
      (let* ((buffer (find-file-noselect litgraph-timeline--file))
             (position (with-current-buffer buffer
                         (save-excursion
                           (goto-char (point-min))
                           (search-forward cite nil t)))))
        (when position
          (pop-to-buffer buffer t)
          (goto-char (- position (length cite)))
          t))
      (progn (kill-new cite)
             (message "No source span; cite copied: %s" cite))))

(defun litgraph-timeline-visit ()
  "Jump to the source of the row at point, or to its cited text.
On a world line, go to the world's fork row; on a diagnostic, to its span;
on a layer, to its cite."
  (interactive)
  (let ((row (get-text-property (point) 'litgraph-timeline-row))
        (world (get-text-property (point) 'litgraph-timeline-world))
        (layer (get-text-property (point) 'litgraph-timeline-layer))
        (span (get-text-property (point) 'litgraph-timeline-span)))
    (cond
     (row (cond ((litgraph-get row 'source)
                 (litgraph-timeline--visit-span (litgraph-get row 'source)))
                ((litgraph-get row 'cite)
                 (litgraph-timeline--visit-cite (litgraph-get row 'cite) row))
                (t (user-error "Row %s has no source span and no cite"
                               (litgraph-get row 'id)))))
     (world (let ((fork (litgraph-get (litgraph-timeline--world
                                       litgraph-timeline--doc world)
                                      'fork_row)))
              (unless (and fork (litgraph-timeline--goto-row fork))
                (user-error "World %s has no fork row shown" world))))
     (layer (let ((cite (litgraph-get
                         (seq-find (lambda (l) (equal (litgraph-get l 'id) layer))
                                   (litgraph-get litgraph-timeline--doc 'layers))
                         'cite)))
              (if cite (litgraph-timeline--visit-cite cite nil)
                (user-error "Layer %s has no cite" layer))))
     (span (litgraph-timeline--visit-span span))
     (t (user-error "Nothing to visit here")))))

(defun litgraph-timeline--read-layer ()
  "The layer on this line, else one read with completion."
  (or (get-text-property (point) 'litgraph-timeline-layer)
      (let ((choices (mapcar (lambda (layer)
                               (cons (format "%s  %s  %s" (litgraph-get layer 'id)
                                             (litgraph-get layer 'class)
                                             (or (litgraph-get layer 'label) ""))
                                     (litgraph-get layer 'id)))
                             (litgraph-get litgraph-timeline--doc 'layers))))
        (unless choices (user-error "This timeline has no layers"))
        (cdr (assoc (completing-read "Toggle layer: " choices nil t) choices)))))

(defun litgraph-timeline-toggle-layer (layer)
  "Switch LAYER (an id) on or off and refresh."
  (interactive (progn (litgraph-timeline--require)
                      (list (litgraph-timeline--read-layer))))
  (let ((disabled (litgraph-timeline--disabled litgraph-timeline--doc)))
    (setq litgraph-timeline--disabled-layers
          (if (member layer disabled)
              (remove layer disabled)
            (cons layer (copy-sequence disabled))))
    (litgraph-timeline-render litgraph-timeline--doc)
    (litgraph-timeline-refresh)
    (message "Layer %s %s" layer
             (if (member layer litgraph-timeline--disabled-layers) "off" "on"))))

(defun litgraph-timeline--read-date (prompt current)
  "Read a date (YYYY-MM-DD) with PROMPT; CURRENT is the default.
An empty answer means none."
  (let ((answer (string-trim
                 (read-string (format "%s (YYYY-MM-DD, empty for none): " prompt)
                              current))))
    (cond ((string-empty-p answer) nil)
          ((string-match-p "\\`[0-9]\\{4\\}-[0-9]\\{2\\}-[0-9]\\{2\\}\\'" answer)
           answer)
          (t (user-error "Not a date: %s" answer)))))

(defun litgraph-timeline-set-dates (as-of known-at)
  "Judge statuses at AS-OF, with knowledge up to KNOWN-AT; refresh.
Either may be nil: the op's default as_of (today), or everything recorded."
  (interactive
   (list (litgraph-timeline--read-date "as_of" litgraph-timeline--as-of)
         (litgraph-timeline--read-date "known_at" litgraph-timeline--known-at)))
  (unless litgraph-timeline--file (user-error "Not in a timeline buffer"))
  (setq litgraph-timeline--as-of as-of
        litgraph-timeline--known-at known-at)
  (litgraph-timeline-refresh))

(defun litgraph-timeline--world-candidates ()
  "Completion candidates for the worlds shown: (TEXT . ID)."
  (mapcar (lambda (world)
            (cons (format "%s  %s" (litgraph-get world 'id)
                          (or (litgraph-get world 'label) ""))
                  (litgraph-get world 'id)))
          (litgraph-get litgraph-timeline--doc 'worlds)))

(defun litgraph-timeline--read-world (prompt &optional default)
  "Read a world id with PROMPT; DEFAULT is an id."
  (let* ((choices (litgraph-timeline--world-candidates))
         (default-text (car (rassoc default choices))))
    (cdr (assoc (completing-read prompt choices nil t nil nil default-text)
                choices))))

(defun litgraph-timeline--world-at-point ()
  "Id of the world of the line at point, or nil."
  (or (get-text-property (point) 'litgraph-timeline-world)
      (litgraph-get (get-text-property (point) 'litgraph-timeline-row) 'world)))

(defun litgraph-timeline-set-world (world)
  "Show only WORLD (an id), or every world when WORLD is nil; refresh.
Interactively, with a prefix argument or when one world is shown, show all."
  (interactive
   (progn (litgraph-timeline--require)
          (list (unless (or current-prefix-arg litgraph-timeline--world-filter)
                  (litgraph-timeline--read-world
                   "Show world: " (litgraph-timeline--world-at-point))))))
  (setq litgraph-timeline--world-filter world)
  (litgraph-timeline-render litgraph-timeline--doc)
  (litgraph-timeline-refresh))

(defun litgraph-timeline-diff (a b)
  "Show how worlds A and B (ids) differ."
  (interactive
   (progn
     (litgraph-timeline--require)
     (let* ((doc litgraph-timeline--doc)
            (here (litgraph-timeline--world-at-point))
            (a (litgraph-timeline--read-world "Diff world: " here))
            (parent (litgraph-get (litgraph-timeline--world doc a) 'parent))
            (other (or parent
                       (seq-find (lambda (id) (not (equal id a)))
                                 (mapcar (lambda (w) (litgraph-get w 'id))
                                         (litgraph-get doc 'worlds))))))
       (list a (litgraph-timeline--read-world
                (format "Diff %s against world: " a) other)))))
  (litgraph-timeline--require)
  (let ((doc litgraph-timeline--doc)
        (file litgraph-timeline--file)
        (buffer (get-buffer-create
                 (format "*litgraph-timeline-diff: %s*"
                         (file-name-nondirectory
                          (or litgraph-timeline--file "timeline"))))))
    (with-current-buffer buffer
      (litgraph-timeline-diff-mode)
      (setq litgraph-timeline--doc doc
            litgraph-timeline--file file)
      (litgraph-timeline-render-diff doc a b))
    (pop-to-buffer buffer litgraph-timeline-display-action)
    buffer))

(defun litgraph-timeline-render-diff (doc a b)
  "Render the difference of worlds A and B of DOC into this buffer."
  (let* ((inhibit-read-only t)
         (diff (litgraph-timeline-diff-worlds doc a b))
         (layer-width (litgraph-timeline--layer-width doc))
         (name (lambda (id)
                 (format "%s %s" id (or (litgraph-get
                                         (litgraph-timeline--world doc id)
                                         'label)
                                        ""))))
         (row-line (lambda (prefix face row)
                     (insert (propertize
                              (concat (propertize prefix 'face face) " "
                                      (litgraph-timeline--row-text row layer-width))
                              'litgraph-timeline-row row)
                             "\n")))
         (section (lambda (title rows prefix face)
                    (litgraph-timeline--heading (format "%s (%d)" title (length rows)))
                    (dolist (row rows) (funcall row-line prefix face row))
                    (insert "\n"))))
    (erase-buffer)
    (insert (propertize "Diff  " 'face 'litgraph-view-title)
            (propertize (funcall name a) 'face 'diff-removed)
            "  ⟷  "
            (propertize (funcall name b) 'face 'diff-added) "\n"
            (propertize "As of " 'face 'litgraph-view-label)
            (or (litgraph-get doc 'as_of) "?") "\n\n")
    (funcall section "Common" (plist-get diff :common) " " 'default)
    (litgraph-timeline--heading
     (format "Changed (%d)" (length (plist-get diff :changed))))
    (dolist (pair (plist-get diff :changed))
      (funcall row-line "-" 'diff-removed (car pair))
      (funcall row-line "+" 'diff-added (cdr pair)))
    (insert "\n")
    (funcall section (format "Only in %s" a) (plist-get diff :only-a)
             "-" 'diff-removed)
    (funcall section (format "Only in %s" b) (plist-get diff :only-b)
             "+" 'diff-added)
    (goto-char (point-min))
    (set-buffer-modified-p nil)))

(defun litgraph-timeline-next-row (n)
  "Move to the Nth next row line (previous if N is negative)."
  (interactive "p")
  (let ((step (if (< n 0) -1 1)))
    (dotimes (_ (abs n))
      (let ((start (point)))
        (forward-line step)
        (while (and (not (get-text-property (point) 'litgraph-timeline-row))
                    (zerop (forward-line step))
                    (not (if (> step 0) (eobp) (bobp)))))
        (unless (get-text-property (point) 'litgraph-timeline-row)
          (goto-char start))))))

(defun litgraph-timeline-previous-row (n)
  "Move to the Nth previous row line."
  (interactive "p")
  (litgraph-timeline-next-row (- n)))

(defun litgraph-timeline-raw-json ()
  "Show the raw JSON of this timeline, pretty-printed."
  (interactive)
  (litgraph-timeline--require)
  (let ((raw litgraph-timeline--raw)
        (doc litgraph-timeline--doc)
        (buffer (get-buffer-create "*litgraph-timeline-json*")))
    (with-current-buffer buffer
      (let ((inhibit-read-only t))
        (erase-buffer)
        (if raw
            (progn (insert raw)
                   (require 'json)
                   (ignore-errors (json-pretty-print-buffer)))
          (pp doc (current-buffer)))
        (goto-char (point-min)))
      (cond ((fboundp 'js-json-mode) (js-json-mode))
            ((fboundp 'json-mode) (json-mode))
            (t (fundamental-mode)))
      (setq buffer-read-only t)
      (local-set-key "q" #'quit-window))
    (pop-to-buffer buffer)))


;;;; Modes

(defvar-keymap litgraph-timeline-mode-map
  :doc "Keymap of `litgraph-timeline-mode'."
  :parent special-mode-map
  "RET" #'litgraph-timeline-visit
  "=" #'litgraph-timeline-diff
  "l" #'litgraph-timeline-toggle-layer
  "t" #'litgraph-timeline-set-dates
  "w" #'litgraph-timeline-set-world
  "g" #'litgraph-timeline-refresh
  "j" #'litgraph-timeline-raw-json
  "n" #'litgraph-timeline-next-row
  "p" #'litgraph-timeline-previous-row)

(define-derived-mode litgraph-timeline-mode special-mode "litgraph-timeline"
  "Major mode of litgraph timeline buffers.
One row per event, deadline or fork; the graph column has one lane per
world.

\\{litgraph-timeline-mode-map}"
  (setq-local truncate-lines t)
  (setq-local revert-buffer-function
              (lambda (&rest _) (litgraph-timeline-refresh))))

(defvar-keymap litgraph-timeline-diff-mode-map
  :doc "Keymap of `litgraph-timeline-diff-mode'."
  :parent special-mode-map
  "RET" #'litgraph-timeline-visit
  "n" #'litgraph-timeline-next-row
  "p" #'litgraph-timeline-previous-row)

(define-derived-mode litgraph-timeline-diff-mode special-mode
  "litgraph-timeline-diff"
  "Major mode of the difference between two timeline worlds.

\\{litgraph-timeline-diff-mode-map}"
  (setq-local truncate-lines t))


;;;; litigate-mode

;;;###autoload
(defun litgraph-timeline-bind-litigate-key (map)
  "Bind `litgraph-timeline-litigate-key' to `litgraph-timeline' in MAP."
  (when (and litgraph-timeline-litigate-key (keymapp map))
    (keymap-set map litgraph-timeline-litigate-key #'litgraph-timeline)))

;;;###autoload
(with-eval-after-load 'litigate-mode
  (litgraph-timeline-bind-litigate-key
   (and (boundp 'litigate-mode-map) litigate-mode-map)))

;;;###autoload
(with-eval-after-load 'litigate-ts-mode
  (litgraph-timeline-bind-litigate-key
   (and (boundp 'litigate-ts-mode-map) litigate-ts-mode-map)))

(provide 'litgraph-timeline)

;;; litgraph-timeline.el ends here
