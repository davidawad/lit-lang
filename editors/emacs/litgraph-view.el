;;; litgraph-view.el --- Result buffers for litgraph -*- lexical-binding: t; -*-

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

;; Turns a litgraph response into a readable buffer:
;;
;;   * Summary       value, best line, outcome odds
;;   * Tables        chain outcomes, tornado, paths, deadlines, ...
;;   * Warnings      grouped by code, with what each one means
;;   * Provenance    folded
;;
;; Raw JSON is one key (j) away, any table opens as a sortable
;; `tabulated-list' (T) or exports as an Org table (o).  Each op has a
;; renderer in `litgraph-view-renderers'; unknown ops (new ones in a newer
;; litgraph) fall back to a generic renderer, so nothing is ever hidden.
;;
;; Every renderer builds the same data model: a list of sections, each of
;; blocks (key/value lists, tables, text).  A table is a plist with
;; :title, :columns ((HEADER . ALIGN) ...) and :rows; a cell is a string
;; or (STRING . RAW) where RAW is the sortable value.

;;; Code:

(require 'cl-lib)
(require 'seq)
(require 'subr-x)
(require 'outline)
(require 'tabulated-list)
(require 'litgraph)

(declare-function litgraph-graph "litgraph-graph")
(declare-function litgraph-menu "litgraph-menu")
(declare-function litgraph-org-import "litgraph-org")
(declare-function json-pretty-print-buffer "json")

(defgroup litgraph-view nil
  "Result buffers for litgraph."
  :group 'litgraph
  :prefix "litgraph-view-")

(defcustom litgraph-view-max-column-width 48
  "Widest a table column may grow in a result buffer."
  :type 'natnum)

(defcustom litgraph-view-display-action nil
  "Action passed to `pop-to-buffer' when showing a result buffer."
  :type 'sexp)

(defface litgraph-view-title '((t :inherit font-lock-keyword-face :weight bold))
  "Face of the result buffer's title line.")

(defface litgraph-view-label '((t :inherit font-lock-doc-face))
  "Face of key/value labels.")

(defface litgraph-view-header
  '((t :inherit header-line :weight bold :underline t))
  "Face of table header rows.")

(defface litgraph-view-good '((t :inherit success))
  "Face for favourable values.")

(defface litgraph-view-bad '((t :inherit error))
  "Face for unfavourable values.")

(defface litgraph-view-note '((t :inherit shadow))
  "Face of secondary text.")


;;;; Formatting

(defconst litgraph-view--percent-keys
  '(p probability step_p p_loss mass sink_mass interrupt_mass)
  "Response keys whose numbers are probabilities.")

(defun litgraph-view--commas (n)
  "Format the integer N with thousands separators."
  (concat (if (< n 0) "-" "")
          (litgraph-view--group-digits (number-to-string (abs n)))))

(defun litgraph-view--group-digits (digits)
  "Insert commas into the digit string DIGITS."
  (let ((out nil) (count 0))
    (dolist (char (nreverse (string-to-list digits)))
      (when (and (> count 0) (zerop (% count 3))) (push ?, out))
      (push char out)
      (cl-incf count))
    (concat out)))

(defun litgraph-view-format-number (x &optional percent)
  "Format the number X for humans; as a percentage if PERCENT."
  (cond
   (percent (format "%.1f%%" (* 100.0 x)))
   ((and (integerp x)) (litgraph-view--commas x))
   ((not (floatp x)) (format "%s" x))
   ((zerop x) "0")
   ((or (= x (ftruncate x)) (>= (abs x) 1000))
    (litgraph-view--commas (round x)))
   ((>= (abs x) 100) (format "%.0f" x))
   ((>= (abs x) 10) (format "%.1f" x))
   ((>= (abs x) 1) (format "%.2f" x))
   (t (format "%.4g" x))))

(defun litgraph-view--text (value)
  "Plain text for the JSON VALUE (non-numbers)."
  (cond ((null value) "")
        ((stringp value) value)
        ((eq value t) "yes")
        ((numberp value) (format "%s" value))
        ((and (listp value) (seq-every-p #'stringp value))
         (string-join value ", "))
        (t (format "%S" value))))

(defun litgraph-view-cell (key value)
  "A table cell for VALUE found under KEY: (STRING . RAW)."
  (if (numberp value)
      (cons (litgraph-view-format-number
             value (memq key litgraph-view--percent-keys))
            value)
    (litgraph-view--text value)))

(defun litgraph-view--cell-text (cell)
  "The display string of table CELL."
  (if (consp cell) (car cell) (or cell "")))

(defun litgraph-view--cell-raw (cell)
  "The sort value of table CELL."
  (if (consp cell) (cdr cell) (or cell "")))

(defun litgraph-view--scalar-p (value)
  "Non-nil if VALUE is a JSON scalar."
  (or (stringp value) (numberp value) (eq value t)))

(defun litgraph-view--alist-p (value)
  "Non-nil if VALUE is a parsed JSON object."
  (and (consp value) (consp (car value)) (symbolp (caar value))))

(defun litgraph-view--records-p (value)
  "Non-nil if VALUE is a parsed JSON array of objects."
  (and (consp value) (seq-every-p #'litgraph-view--alist-p value)))

(defun litgraph-view--label (key)
  "Human label for the response KEY (a symbol or string)."
  (let ((name (replace-regexp-in-string "[_-]" " " (format "%s" key))))
    (concat (upcase (substring name 0 1)) (substring name 1))))

(defun litgraph-view--truncate (string width)
  "Truncate STRING to WIDTH columns."
  (truncate-string-to-width (replace-regexp-in-string "[\n\t]+" " " string)
                            width nil nil "…"))


;;;; Tables

(defun litgraph-view-table (title columns rows &optional ids)
  "Make a table named TITLE with COLUMNS, ROWS and optional row IDS."
  (list :title title :columns columns :rows rows :ids ids))

(defun litgraph-view-auto-table (title records &optional ids-key max-columns)
  "Table of the scalar fields of RECORDS (a list of alists), named TITLE.
IDS-KEY names the field to use as each row's id."
  (let* ((keys (let (acc)
                 (dolist (record records)
                   (dolist (cell record)
                     (when (and (litgraph-view--scalar-p (cdr cell))
                                (not (memq (car cell) acc)))
                       (push (car cell) acc))))
                 (nreverse acc)))
         (keys (seq-take keys (or max-columns 9))))
    (litgraph-view-table
     title
     (mapcar (lambda (key)
               (cons (litgraph-view--label key)
                     (if (seq-some (lambda (r) (numberp (alist-get key r)))
                                   records)
                         'right 'left)))
             keys)
     (mapcar (lambda (record)
               (mapcar (lambda (key)
                         (litgraph-view-cell key (alist-get key record)))
                       keys))
             records)
     (and ids-key (mapcar (lambda (r) (alist-get ids-key r)) records)))))

(defun litgraph-view--column-widths (table)
  "Display widths of the columns of TABLE."
  (let ((widths (mapcar (lambda (col) (string-width (car col)))
                        (plist-get table :columns))))
    (dolist (row (plist-get table :rows))
      (setq widths
            (cl-mapcar (lambda (w cell)
                         (max w (min litgraph-view-max-column-width
                                     (string-width
                                      (litgraph-view--cell-text cell)))))
                       widths row)))
    widths))

(defun litgraph-view--pad (string width align)
  "Pad or truncate STRING to WIDTH, aligned to ALIGN."
  (let* ((string (litgraph-view--truncate string width))
         (gap (make-string (max 0 (- width (string-width string))) ?\s)))
    (if (eq align 'right) (concat gap string) (concat string gap))))

(defun litgraph-view--insert-table (table index)
  "Insert TABLE (number INDEX in the buffer) with two spaces of indent."
  (let* ((widths (litgraph-view--column-widths table))
         (columns (plist-get table :columns))
         (ids (plist-get table :ids)))
    (insert (propertize
             (concat "  "
                     (string-join
                      (cl-mapcar (lambda (col w)
                                   (litgraph-view--pad (car col) w 'left))
                                 columns widths)
                      "  ")
                     "\n")
             'face 'litgraph-view-header 'litgraph-table index))
    (let ((n 0))
      (dolist (row (plist-get table :rows))
        (let ((start (point)))
          (insert "  "
                  (string-join
                   (cl-mapcar (lambda (col w cell)
                                (litgraph-view--pad
                                 (litgraph-view--cell-text cell) w (cdr col)))
                              columns widths row)
                   "  ")
                  "\n")
          (add-text-properties start (point)
                               (list 'litgraph-table index
                                     'litgraph-id (nth n ids))))
        (cl-incf n)))))

(defun litgraph-view-table-org-rows (table)
  "TABLE as a list for Org: header row, `hline', then rows with raw values."
  (append
   (list (mapcar #'car (plist-get table :columns)) 'hline)
   (mapcar (lambda (row)
             (mapcar (lambda (cell)
                       (let ((raw (litgraph-view--cell-raw cell)))
                         (cond ((integerp raw) raw)
                               ((and (floatp raw) (< (abs raw) 1e15)
                                     (= raw (ftruncate raw)))
                                (truncate raw))
                               ((numberp raw)
                                (string-to-number (format "%.6g" raw)))
                               (t (litgraph-view--cell-text cell)))))
                     row))
           (plist-get table :rows))))

(defun litgraph-view-table-to-org (table)
  "Render TABLE as Org table text."
  (let ((widths (litgraph-view--column-widths table))
        (columns (plist-get table :columns)))
    (concat
     (when-let* ((title (plist-get table :title)))
       (format "#+CAPTION: %s\n" title))
     "| " (string-join (mapcar #'car columns) " | ") " |\n"
     "|" (string-join (mapcar (lambda (w) (make-string (+ w 2) ?-)) widths)
                      "+")
     "|\n"
     (mapconcat
      (lambda (row)
        (concat "| "
                (string-join
                 (mapcar (lambda (cell)
                           (replace-regexp-in-string
                            "|" "\\\\vert{}"
                            (litgraph-view--cell-text cell)))
                         row)
                 " | ")
                " |\n"))
      (plist-get table :rows) ""))))


;;;; Renderers

(defvar litgraph-view-renderers nil
  "Alist of (OP . FUNCTION) turning a result into sections.
FUNCTION takes the parsed `result' and returns a list of sections; see
`litgraph-view-section'.  Add entries to customize an op's display.")

(defun litgraph-view-section (title blocks &optional folded)
  "A section named TITLE holding BLOCKS; hidden at first when FOLDED."
  (list :title title :blocks blocks :folded folded))

(defun litgraph-view--kv (&rest pairs)
  "A key/value block from PAIRS of LABEL VALUE-STRING; nil values are dropped."
  (let (acc)
    (while pairs
      (let ((label (pop pairs)) (value (pop pairs)))
        (when (and value (not (equal value "")))
          (push (cons label value) acc))))
    (list :kv (nreverse acc))))

(defun litgraph-view--num (result key &rest more)
  "Formatted number at KEY (and MORE path) in RESULT, or nil."
  (let* ((path (cons key more))
         (value (apply #'litgraph-get result path)))
    (and (numberp value)
         (litgraph-view-format-number
          value (memq (car (last path)) litgraph-view--percent-keys)))))

(defun litgraph-view--flatten (object &optional prefix)
  "Flatten the alist OBJECT into (LABEL . STRING) pairs, dotted under PREFIX."
  (cl-loop for (key . value) in object
           for label = (if prefix (format "%s.%s" prefix key) (format "%s" key))
           append
           (cond ((litgraph-view--alist-p value)
                  (litgraph-view--flatten value label))
                 ((litgraph-view--scalar-p value)
                  (list (cons label (if (numberp value)
                                        (litgraph-view-format-number value)
                                      (litgraph-view--text value)))))
                 ((null value) nil)
                 ((litgraph-view--records-p value)
                  (list (cons label (format "%d entries" (length value)))))
                 (t (list (cons label (litgraph-view--text value)))))))

(defun litgraph-view--generic-sections (result)
  "Sections for any RESULT, whatever its shape."
  (cond
   ((litgraph-view--records-p result)
    (list (litgraph-view-section
           "Results"
           (list (list :table (litgraph-view-auto-table nil result))))))
   ((litgraph-view--alist-p result)
    (let ((scalars (seq-filter (lambda (cell)
                                 (litgraph-view--scalar-p (cdr cell)))
                               result))
          (sections nil))
      (dolist (cell result)
        (let ((key (car cell)) (value (cdr cell)))
          (cond
           ((litgraph-view--scalar-p value) nil)
           ((litgraph-view--records-p value)
            (push (litgraph-view-section
                   (litgraph-view--label key)
                   (list (list :table (litgraph-view-auto-table nil value))))
                  sections))
           ((litgraph-view--alist-p value)
            (push (litgraph-view-section
                   (litgraph-view--label key)
                   (list (list :kv (litgraph-view--flatten value))))
                  sections))
           ((and (consp value) (seq-every-p #'litgraph-view--scalar-p value))
            (push (litgraph-view-section
                   (litgraph-view--label key)
                   (list (list :text (mapconcat #'litgraph-view--text value
                                                "\n"))))
                  sections))
           ((consp value)
            (push (litgraph-view-section
                   (litgraph-view--label key)
                   (list (list :text (format "%S" value))))
                  sections)))))
      (cons (litgraph-view-section
             "Summary"
             (list (list :kv (mapcar (lambda (c)
                                       (cons (litgraph-view--label (car c))
                                             (litgraph-view-cell-text (car c)
                                                                      (cdr c))))
                                     scalars))))
            (nreverse sections))))
   ((null result) nil)
   (t (list (litgraph-view-section
             "Result" (list (list :text (litgraph-view--text result))))))))

(defun litgraph-view-cell-text (key value)
  "Display text of VALUE under KEY."
  (litgraph-view--cell-text (litgraph-view-cell key value)))

(defun litgraph-view--node-label (node)
  "Label of the {id, label} object NODE."
  (or (litgraph-get node 'label) (litgraph-get node 'id) ""))

(defun litgraph-view--short-id (id)
  "Drop the pack prefix from the node or edge ID."
  (if (and (stringp id) (string-match "::\\(.*\\)\\'" id))
      (match-string 1 id)
    id))

(defun litgraph-view--render-solve (result)
  "Sections for a solve RESULT."
  (let* ((line (litgraph-get result 'best_line))
         (last-step (car (last line))))
    (list
     (litgraph-view-section
      "Summary"
      (list
       (litgraph-view--kv
        "Value" (litgraph-view--num result 'value)
        "Start" (litgraph-view--node-label (litgraph-get result 'start))
        "Best line" (and line
                         (format "%d steps, ending at %s" (length line)
                                 (litgraph-view--short-id
                                  (litgraph-get last-step 'to))))
        "Opponent value" (litgraph-view--num result 'opponent_value)
        "CVaR" (litgraph-view--num result 'cvar)
        "Converged" (and (assq 'converged result)
                         (if (litgraph-get result 'converged) "yes" "NO"))
        "Policy size" (litgraph-view--num result 'policy_size))))
     (litgraph-view-section
      (format "Best line (%d steps)" (length line))
      (list
       (list :table
             (litgraph-view-table
              nil
              '(("#" . right) ("Step" . left) ("Role" . left) ("p" . right)
                ("Value" . right) ("Authority" . left))
              (cl-loop for step in line for i from 1
                       collect (list (number-to-string i)
                                     (litgraph-view--text
                                      (litgraph-get step 'label))
                                     (litgraph-view--text
                                      (litgraph-get step 'role))
                                     (litgraph-view-cell 'p
                                                         (litgraph-get step 'p))
                                     (litgraph-view-cell 'q
                                                         (litgraph-get step 'q))
                                     (litgraph-view--text
                                      (litgraph-get step 'authority))))
              (mapcar (lambda (step) (litgraph-get step 'to)) line))))))))

(defun litgraph-view--absorption-table (result)
  "Table of outcomes from a chain RESULT, most likely first."
  (let ((rows (sort (copy-sequence (litgraph-get result 'absorption))
                    (lambda (a b) (> (or (litgraph-get a 'p) 0)
                                     (or (litgraph-get b 'p) 0))))))
    (litgraph-view-table
     "Outcomes"
     '(("Outcome" . left) ("p" . right) ("Utility" . right) ("Tags" . left))
     (mapcar (lambda (row)
               (list (litgraph-view--text (litgraph-get row 'label))
                     (litgraph-view-cell 'p (litgraph-get row 'p))
                     (litgraph-view-cell 'utility (litgraph-get row 'utility))
                     (litgraph-view--text (litgraph-get row 'outcome))))
             rows)
     (mapcar (lambda (row) (litgraph-get row 'terminal)) rows))))

(defun litgraph-view--render-chain (result)
  "Sections for a chain RESULT."
  (let* ((outcomes (litgraph-view--absorption-table result))
         (top (seq-take (plist-get outcomes :rows) 3))
         (expected (litgraph-get result 'expected)))
    (list
     (litgraph-view-section
      "Summary"
      (list
       (apply
        #'litgraph-view--kv
        "Start" (litgraph-view--node-label (litgraph-get result 'start))
        "Expected net" (litgraph-view--num result 'expected_net)
        "Expected utility" (litgraph-view--num result 'expected_utility)
        "Odds" (and top
                    (mapconcat (lambda (row)
                                 (format "%s %s"
                                         (litgraph-view--cell-text
                                          (nth 1 row))
                                         (litgraph-view--cell-text
                                          (nth 0 row))))
                               top "  ·  "))
        "Expected steps" (litgraph-view--num result 'expected_steps)
        "Sink mass" (and (not (zerop (or (litgraph-get result 'sink_mass) 0)))
                         (litgraph-view--num result 'sink_mass))
        (cl-loop for (key . value) in expected
                 append (list (format "Expected %s" key)
                              (litgraph-view-format-number value))))))
     (litgraph-view-section "Outcomes" (list (list :table outcomes)))
     (when-let* ((visits (litgraph-get result 'top_visits)))
       (litgraph-view-section
        "Most visited nodes"
        (list (list :table
                    (litgraph-view-table
                     nil '(("Node" . left) ("Visits" . right))
                     (mapcar (lambda (v)
                               (list (litgraph-get v 'node)
                                     (litgraph-view-cell
                                      'visits (litgraph-get v 'visits))))
                             visits)
                     (mapcar (lambda (v) (litgraph-get v 'node))
                             visits)))))))))

(defun litgraph-view--render-simulate (result)
  "Sections for a simulate RESULT."
  (let* ((net (litgraph-get result 'net))
         (stat-keys '(mean std p05 p25 p50 p75 p95))
         (rows (cons (cons 'net net)
                     (litgraph-get result 'metrics))))
    (list
     (litgraph-view-section
      "Summary"
      (list (litgraph-view--kv
             "Runs" (litgraph-view--num result 'runs)
             "Mean net" (litgraph-view--num result 'net 'mean)
             "P(loss)" (and (numberp (litgraph-get result 'p_loss))
                            (litgraph-view-format-number
                             (litgraph-get result 'p_loss) t))
             "CVaR" (litgraph-view--num result 'cvar)
             "Median net" (litgraph-view--num result 'net 'p50)
             "Range (p05..p95)" (and net
                                     (format "%s .. %s"
                                             (litgraph-view--num net 'p05)
                                             (litgraph-view--num net 'p95)))
             "Seed" (litgraph-view--num result 'seed)
             "Truncated runs" (and (> (or (litgraph-get result 'truncated) 0) 0)
                                   (litgraph-view--num result 'truncated)))))
     (litgraph-view-section
      "Distributions"
      (list (list :table
                  (litgraph-view-table
                   nil
                   (cons '("Metric" . left)
                         (mapcar (lambda (k) (cons (symbol-name k) 'right))
                                 stat-keys))
                   (mapcar (lambda (row)
                             (cons (format "%s" (car row))
                                   (mapcar (lambda (k)
                                             (litgraph-view-cell
                                              k (litgraph-get (cdr row) k)))
                                           stat-keys)))
                           rows)))))
     (when-let* ((terminals (litgraph-get result 'terminals)))
       (litgraph-view-section
        "Terminals"
        (list (list :table (litgraph-view-auto-table nil terminals
                                                     'terminal))))))))

(defun litgraph-view--sensitivity-table (title rows)
  "Table of sensitivity ROWS (tornado or calibration shape) named TITLE."
  (litgraph-view-table
   title
   '(("Input" . left) ("Low" . right) ("High" . right) ("Value@low" . right)
     ("Value@high" . right) ("Swing" . right) ("Changes policy" . left))
   (mapcar (lambda (row)
             (list (litgraph-view--text (litgraph-get row 'input))
                   (litgraph-view-cell 'low_value (litgraph-get row 'low_value))
                   (litgraph-view-cell 'high_value
                                       (litgraph-get row 'high_value))
                   (litgraph-view-cell 'v_low (litgraph-get row 'v_low))
                   (litgraph-view-cell 'v_high (litgraph-get row 'v_high))
                   (litgraph-view-cell 'swing (litgraph-get row 'swing))
                   (if (litgraph-get row 'policy_changes) "yes" "")))
           rows)
   (mapcar (lambda (row) (litgraph-get row 'input)) rows)))

(defun litgraph-view--render-tornado (result)
  "Sections for a tornado (or calibration) RESULT."
  (let ((rows (litgraph-get result 'rows))
        (top nil))
    (setq top (car (sort (copy-sequence rows)
                         (lambda (a b) (> (or (litgraph-get a 'swing) 0)
                                          (or (litgraph-get b 'swing) 0))))))
    (append
     (list
      (litgraph-view-section
       "Summary"
       (list (litgraph-view--kv
              "Base value" (litgraph-view--num result 'base_value)
              "Most sensitive" (and top (litgraph-get top 'input))
              "Its swing" (and top (litgraph-view--num top 'swing))
              "Inputs that change the policy"
              (let ((n (length (seq-filter
                                (lambda (r) (litgraph-get r 'policy_changes))
                                rows))))
                (and rows (number-to-string n)))))))
     (when rows
       (list (litgraph-view-section
              "Sensitivity"
              (list (list :table
                          (litgraph-view--sensitivity-table nil rows))))))
     ;; The calibration op returns the same row shape under other keys.
     (cl-loop for key in '(probabilities durations)
              for records = (litgraph-get result key)
              when records
              collect (litgraph-view-section
                       (litgraph-view--label key)
                       (list (list :table
                                   (litgraph-view--sensitivity-table
                                    nil records))))))))

(defun litgraph-view--path-table (paths)
  "Table of PATHS (path or pareto frontier entries)."
  (let ((keys (let (acc)
                (dolist (path paths)
                  (dolist (cell (litgraph-get path 'totals))
                    (unless (memq (car cell) acc) (push (car cell) acc))))
                (nreverse acc))))
    (litgraph-view-table
     nil
     (append '(("#" . right) ("Ends at" . left) ("Probability" . right)
               ("Steps" . right))
             (mapcar (lambda (k) (cons (litgraph-view--label k) 'right)) keys))
     (cl-loop for path in paths for i from 1
              collect (append
                       (list (number-to-string i)
                             (litgraph-view--node-label
                              (litgraph-get path 'end))
                             (litgraph-view-cell
                              'probability (litgraph-get path 'probability))
                             (litgraph-view-cell 'steps
                                                 (litgraph-get path 'steps)))
                       (mapcar (lambda (k)
                                 (litgraph-view-cell
                                  k (litgraph-get path 'totals k)))
                               keys)))
     (mapcar (lambda (path) (litgraph-get path 'end 'id)) paths))))

(defun litgraph-view--render-paths (result)
  "Sections for a path or pareto RESULT."
  (let ((paths (or (litgraph-get result 'paths)
                   (litgraph-get result 'frontier))))
    (list
     (litgraph-view-section
      "Summary"
      (list (litgraph-view--kv
             "From" (litgraph-view--node-label (litgraph-get result 'from))
             "To" (litgraph-get result 'to)
             "Metric" (litgraph-get result 'metric)
             "Objectives" (litgraph-view--text (litgraph-get result 'objectives))
             "Lines" (number-to-string (length paths))
             "Frontier size" (litgraph-view--num result 'frontier_size)
             "Truncated" (and (litgraph-get result 'truncated) "yes"))))
     (litgraph-view-section
      "Lines" (list (list :table (litgraph-view--path-table paths))))
     (litgraph-view-section
      "Steps of each line"
      (list (list :text
                  (mapconcat
                   (lambda (entry)
                     (format "Line %d\n%s" (car entry)
                             (mapconcat (lambda (s) (concat "  " s))
                                        (litgraph-get (cdr entry) 'line)
                                        "\n")))
                   (cl-loop for path in paths for i from 1
                            collect (cons i path))
                   "\n\n")))
      t))))

(defun litgraph-view--render-sweep (result)
  "Sections for a sweep RESULT."
  (let ((curve (litgraph-get result 'curve))
        (breakpoints (litgraph-get result 'breakpoints)))
    (list
     (litgraph-view-section
      "Summary"
      (list (litgraph-view--kv
             "Parameter" (litgraph-get result 'param)
             "Points" (number-to-string (length curve))
             "Value range"
             (and curve
                  (format "%s .. %s"
                          (litgraph-view-format-number
                           (apply #'min (mapcar #'cadr curve)))
                          (litgraph-view-format-number
                           (apply #'max (mapcar #'cadr curve)))))
             "Policy breakpoints" (number-to-string (length breakpoints)))))
     (litgraph-view-section
      "Curve"
      (list (list :table
                  (litgraph-view-table
                   nil (list (cons (format "%s" (litgraph-get result 'param))
                                   'right)
                             '("Value" . right))
                   (mapcar (lambda (pt)
                             (list (litgraph-view-cell 'x (car pt))
                                   (litgraph-view-cell 'y (cadr pt))))
                           curve)))))
     (when breakpoints
       (litgraph-view-section
        "Breakpoints"
        (list (list :table (litgraph-view-auto-table nil breakpoints))))))))

(defun litgraph-view--render-explain (result)
  "Sections for an explain RESULT."
  (let* ((node (litgraph-get result 'node))
         (options (litgraph-get result 'options)))
    (list
     (litgraph-view-section
      "Summary"
      (list (litgraph-view--kv
             "Node" (litgraph-view--node-label node)
             "Kind" (litgraph-get node 'kind)
             "Who decides" (litgraph-get result 'who_decides)
             "Value" (litgraph-view--num result 'value)
             "Cite" (litgraph-get node 'cite)
             "Options" (number-to-string (length options))
             "Best option"
             (when-let* ((chosen (seq-find (lambda (o) (litgraph-get o 'chosen))
                                           options)))
               (litgraph-view--node-label chosen)))
            (when-let* ((note (litgraph-get node 'note)))
              (list :text note))))
     (litgraph-view-section
      "Options"
      (list (list :table
                  (litgraph-view-table
                   nil
                   '(("" . left) ("Option" . left) ("Role" . left) ("p" . right)
                     ("Value" . right) ("Regret" . right) ("Cost" . right)
                     ("Authority" . left))
                   (mapcar
                    (lambda (o)
                      (list (if (litgraph-get o 'chosen) "*" "")
                            (litgraph-view--text (litgraph-get o 'label))
                            (litgraph-view--text (litgraph-get o 'role))
                            (litgraph-view-cell 'p (litgraph-get o 'p))
                            (litgraph-view-cell 'q (litgraph-get o 'q))
                            (litgraph-view-cell 'regret
                                                (litgraph-get o 'regret))
                            (litgraph-view-cell 'cost (litgraph-get o 'cost))
                            (litgraph-view--text (litgraph-get o 'authority))))
                    options)
                   (mapcar (lambda (o) (litgraph-get o 'to)) options)))))
     (litgraph-view-section
      "What happens next"
      (list (list :text
                  (mapconcat
                   (lambda (o)
                     (format "%s\n%s" (litgraph-view--node-label o)
                             (mapconcat
                              (lambda (then)
                                (format "    %s  %s"
                                        (litgraph-view-cell-text
                                         'p (litgraph-get then 'p))
                                        (litgraph-get then 'terminal)))
                              (litgraph-get o 'then) "\n")))
                   options "\n")))
      t))))

(defun litgraph-view--deadline-rows (result)
  "The deadline entries of RESULT sorted by due date."
  (sort (copy-sequence (litgraph-get result 'deadlines))
        (lambda (a b) (string< (or (litgraph-get a 'due_date) "")
                               (or (litgraph-get b 'due_date) "")))))

(defun litgraph-view--render-deadlines (result)
  "Sections for a deadlines RESULT."
  (let ((rows (litgraph-view--deadline-rows result)))
    (list
     (litgraph-view-section
      "Summary"
      (list (litgraph-view--kv
             "Trigger" (litgraph-get result 'trigger)
             "From" (litgraph-get result 'node)
             "Deadlines" (number-to-string (length rows))
             "First due" (litgraph-get (car rows) 'due_date)
             "Last due" (litgraph-get (car (last rows)) 'due_date))
            (list :text "Press d to turn these into Org headings with DEADLINE timestamps.")))
     (litgraph-view-section
      "Deadlines"
      (list (list :table
                  (litgraph-view-table
                   nil
                   '(("Due" . left) ("Deadline" . left) ("Authority" . left)
                     ("Rule set" . left) ("Days" . right) ("Notes" . left))
                   (mapcar
                    (lambda (d)
                      (list (litgraph-view--text (litgraph-get d 'due_date))
                            (litgraph-view--text
                             (litgraph-get d 'edge 'label))
                            (litgraph-view--text
                             (litgraph-get d 'edge 'authority))
                            (litgraph-view--text
                             (litgraph-get d 'rule_set_label))
                            (format "%s %s"
                                    (litgraph-view-format-number
                                     (litgraph-get d 'deadline_length))
                                    (litgraph-get d 'unit))
                            (string-join
                             (delq nil
                                   (list
                                    (and (litgraph-get d 'last_day_rolled)
                                         "rolled")
                                    (and (litgraph-get d 'service_days_added)
                                         "service days")
                                    (and (litgraph-get d
                                                       'clerk_inaccessibility_applied)
                                         "clerk closed")))
                             ", ")))
                    rows)
                   (mapcar (lambda (d) (litgraph-get d 'edge 'id)) rows)))))
     (litgraph-view-section
      "How each date was computed"
      (list (list :text
                  (mapconcat
                   (lambda (d)
                     (format "%s  %s\n%s"
                             (litgraph-get d 'due_date)
                             (litgraph-view--node-label (litgraph-get d 'edge))
                             (mapconcat (lambda (s) (concat "    " s))
                                        (litgraph-get d 'steps) "\n")))
                   rows "\n\n")))
      t))))

(defun litgraph-view--render-response (response prefix)
  "Sections for the nested RESPONSE envelope, titles prefixed by PREFIX."
  (let ((sections
         (if (litgraph-get response 'ok)
             (litgraph-view--sections (litgraph-get response 'op)
                                      (litgraph-get response 'result))
           (list (litgraph-view-section
                  "Error"
                  (list (list :text
                              (format "%s: %s"
                                      (litgraph-get response 'error 'code)
                                      (litgraph-get response 'error
                                                    'message)))))))))
    (mapcar (lambda (section)
              (plist-put (copy-sequence section) :title
                         (format "%s: %s" prefix (plist-get section :title))))
            sections)))

(defun litgraph-view--render-compare (result)
  "Sections for a compare RESULT."
  (append
   (list (litgraph-view-section
          "Delta (variant minus base)"
          (list (list :kv
                      (cl-loop for (key . value) in (litgraph-get result 'delta)
                               when (numberp value)
                               collect (cons (litgraph-view--label key)
                                             (litgraph-view-format-number
                                              value)))))))
   (litgraph-view--render-response (litgraph-get result 'base) "Base")
   (litgraph-view--render-response (litgraph-get result 'variant) "Variant")))

(defun litgraph-view--render-batch (result)
  "Sections for a batch RESULT (a list of response envelopes)."
  (cl-loop for response in result for i from 1
           append (litgraph-view--render-response
                   response (format "%d %s" i (litgraph-get response 'op)))))

(defun litgraph-view--render-packs (result)
  "Sections for a packs RESULT."
  (list (litgraph-view-section
         "Packs"
         (list (list :table
                     (litgraph-view-table
                      nil
                      '(("Id" . left) ("Forum" . left) ("Nodes" . right)
                        ("Edges" . right) ("Terminals" . right)
                        ("Probabilities" . right) ("Sources" . right)
                        ("Title" . left))
                      (mapcar
                       (lambda (pack)
                         (list (litgraph-get pack 'id)
                               (litgraph-get pack 'forum)
                               (litgraph-view-cell 'n (litgraph-get pack 'nodes))
                               (litgraph-view-cell 'n (litgraph-get pack 'edges))
                               (litgraph-view-cell
                                'n (litgraph-get pack 'terminals))
                               (litgraph-view--text
                                (litgraph-get pack 'quality
                                              'world_edges_with_probability))
                               (litgraph-view-cell
                                'n (litgraph-get pack 'quality 'sources))
                               (litgraph-view--text (litgraph-get pack 'title))))
                       result)
                      (mapcar (lambda (pack) (litgraph-get pack 'id))
                              result)))))))

(defun litgraph-view--render-describe (result)
  "Sections for a describe RESULT."
  (list
   (litgraph-view-section
    "Summary"
    (list (litgraph-view--kv
           "Engine" (litgraph-get result 'engine)
           "API version" (litgraph-view--num result 'api_version)
           "Packs from" (litgraph-get result 'packs_source)
           "Scenarios from" (litgraph-get result 'scenarios_source)
           "Links" (litgraph-view--num result 'links))))
   (litgraph-view-section
    "Ops"
    (list (list :table
                (litgraph-view-table
                 nil '(("Op" . left) ("Arguments and meaning" . left))
                 (mapcar (lambda (cell)
                           (list (symbol-name (car cell)) (cdr cell)))
                         (litgraph-get result 'ops))
                 (mapcar (lambda (cell) (symbol-name (car cell)))
                         (litgraph-get result 'ops))))))
   (litgraph-view-section
    "Scenario library"
    (list (list :table
                (litgraph-view-auto-table
                 nil (litgraph-get result 'scenario_library) 'id))))
   (litgraph-view-section
    "Packs"
    (list (list :table
                (litgraph-view-auto-table nil (litgraph-get result 'packs)
                                          'id))))))

(defun litgraph-view--render-lint (result)
  "Sections for a lint RESULT."
  (let ((diagnostics (litgraph-get result 'diagnostics)))
    (list
     (litgraph-view-section
      "Summary"
      (list (litgraph-view--kv
             "Diagnostics" (litgraph-view--num result 'count)
             "By severity"
             (string-join
              (mapcar (lambda (entry) (format "%s %d" (car entry) (cdr entry)))
                      (let (counts)
                        (dolist (d diagnostics)
                          (let* ((sev (litgraph-get d 'severity))
                                 (cell (assoc sev counts)))
                            (if cell (cl-incf (cdr cell))
                              (push (cons sev 1) counts))))
                        counts))
              ", "))))
     (litgraph-view-section
      "Diagnostics"
      (list (list :table (litgraph-view-auto-table nil diagnostics 'at)))))))

(setq litgraph-view-renderers
      '(("solve" . litgraph-view--render-solve)
        ("chain" . litgraph-view--render-chain)
        ("simulate" . litgraph-view--render-simulate)
        ("tornado" . litgraph-view--render-tornado)
        ("calibration" . litgraph-view--render-tornado)
        ("path" . litgraph-view--render-paths)
        ("pareto" . litgraph-view--render-paths)
        ("sweep" . litgraph-view--render-sweep)
        ("explain" . litgraph-view--render-explain)
        ("deadlines" . litgraph-view--render-deadlines)
        ("compare" . litgraph-view--render-compare)
        ("batch" . litgraph-view--render-batch)
        ("packs" . litgraph-view--render-packs)
        ("describe" . litgraph-view--render-describe)
        ("lint" . litgraph-view--render-lint)))

(defun litgraph-view--sections (op result)
  "Sections for OP's RESULT, via `litgraph-view-renderers' or the generic one."
  (let ((renderer (cdr (assoc op litgraph-view-renderers))))
    (or (and renderer
             (condition-case nil
                 (funcall renderer result)
               (error nil)))
        (litgraph-view--generic-sections result))))


;;;; Programmatic access

(defun litgraph-view-response-sections (response)
  "The sections litgraph would render for the parsed RESPONSE."
  (litgraph-view--sections (litgraph-get response 'op)
                           (litgraph-get response 'result)))

(defun litgraph-view-response-tables (response)
  "All tables litgraph would render for RESPONSE, in display order."
  (seq-mapcat
   (lambda (section)
     (delq nil
           (mapcar (lambda (block)
                     (when (eq (car block) :table)
                       (let ((table (copy-sequence (cadr block))))
                         (if (plist-get table :title)
                             table
                           (plist-put table :title (plist-get section :title))))))
                   (plist-get section :blocks))))
   (litgraph-view-response-sections response)))

(defun litgraph-view-response-summary (response)
  "The key/value pairs of RESPONSE's summary, as (LABEL . TEXT) pairs."
  (seq-mapcat
   (lambda (section)
     (seq-mapcat (lambda (block) (and (eq (car block) :kv) (cadr block)))
                 (plist-get section :blocks)))
   (seq-take (litgraph-view-response-sections response) 1)))


;;;; Warnings and provenance

(defconst litgraph-view--warning-meanings
  '(("payoff-not-authored" . "terminal payoff is an engine default, not authored law or data")
    ("probability-fill" . "probabilities were missing on some edges and got filled by a rule")
    ("mixed-node" . "a party may act or wait here and the world edges lack probabilities")
    ("sink" . "probability mass leaked into a dead end")
    ("fact-unset" . "a matter fact is not set; the authored prior stands in for it")
    ("prior-concentration-estimated" . "prior strength is a default estimate, not a sample size")
    ("compile" . "the graph compiled with adjustments"))
  "What the well-known warning codes mean for how far to trust a number.")

(defun litgraph-view--warnings-section (response)
  "A section listing the warnings of RESPONSE grouped by code."
  (when-let* ((warnings (litgraph-get response 'warnings)))
    (let ((groups nil))
      (dolist (w warnings)
        (let* ((code (or (litgraph-get w 'code) "warning"))
               (at (litgraph-get w 'at))
               (at (cond ((listp at) at) (at (list at))))
               (entry (assoc code groups)))
          (unless entry
            (setq entry (list code 0 nil nil))
            (push entry groups))
          (cl-incf (nth 1 entry) (or (litgraph-get w 'count) 1))
          (unless (nth 2 entry)
            (setf (nth 2 entry) (or (litgraph-get w 'example)
                                    (litgraph-get w 'message))))
          (setf (nth 3 entry) (append (nth 3 entry) at))))
      (litgraph-view-section
       (format "Warnings (%d kinds)" (length groups))
       (list
        (list :text
              (mapconcat
               (lambda (entry)
                 (pcase-let ((`(,code ,count ,example ,at) entry))
                   (format "%s x%d%s%s%s" code count
                           (if-let* ((m (cdr (assoc code
                                                    litgraph-view--warning-meanings))))
                               (format " -- %s" m) "")
                           (if example (format "\n    e.g. %s" example) "")
                           (if at (format "\n    at %s%s"
                                          (string-join (seq-take at 4) ", ")
                                          (if (> (length at) 4) ", ..." ""))
                             ""))))
               (nreverse groups) "\n")))))))

(defun litgraph-view--provenance-section (response)
  "A folded section of RESPONSE's provenance."
  (when-let* ((provenance (litgraph-get response 'provenance)))
    (litgraph-view-section
     "Provenance"
     (list (list :kv (litgraph-view--flatten
                      (seq-remove (lambda (c) (eq (car c) 'packs))
                                  provenance)))
           (when-let* ((packs (litgraph-get provenance 'packs)))
             (list :table (litgraph-view-auto-table "Packs" packs))))
     t)))


;;;; The buffer

(defvar-local litgraph-view--response nil "The parsed response shown.")
(defvar-local litgraph-view--request nil "The request that produced it.")
(defvar-local litgraph-view--raw nil "The raw JSON text of the response.")
(defvar-local litgraph-view--tables nil "Tables shown, in buffer order.")
(defvar litgraph-view--section-title nil
  "Title of the section being inserted, to avoid repeating it on a table.")

(defun litgraph-view--request-label (request)
  "One-line description of the target and parameters of REQUEST."
  (when (hash-table-p request)
    (let ((params (gethash "params" request))
          (sets nil))
      (when (hash-table-p params)
        (maphash (lambda (name value) (push (format "%s=%s" name value) sets))
                 params))
      (string-join
       (delq nil
             (list (litgraph-target-label (gethash "target" request))
                   (and sets (string-join (sort sets #'string<) " "))))
       "  "))))

(defun litgraph-view--insert-blocks (blocks)
  "Insert BLOCKS; collect tables into `litgraph-view--tables'."
  (dolist (block blocks)
    (pcase (car block)
      (:kv
       (let* ((pairs (cadr block))
              (width (apply #'max 0 (mapcar (lambda (p) (string-width (car p)))
                                            pairs))))
         (dolist (pair pairs)
           (insert "  "
                   (propertize (litgraph-view--pad (car pair) width 'left)
                               'face 'litgraph-view-label)
                   "  " (cdr pair) "\n"))))
      (:table
       (let ((table (cadr block)))
         (when (and (plist-get table :title)
                    (not (equal (plist-get table :title)
                                litgraph-view--section-title)))
           (insert "  " (propertize (plist-get table :title) 'face 'bold)
                   "\n"))
         (setq litgraph-view--tables
               (append litgraph-view--tables (list table)))
         (litgraph-view--insert-table table
                                      (1- (length litgraph-view--tables)))))
      (:text
       (dolist (line (split-string (cadr block) "\n"))
         (insert "  " line "\n")))))
  (insert "\n"))

(defun litgraph-view--insert-section (section)
  "Insert SECTION; fold it afterwards when asked."
  (let ((start (point))
        (litgraph-view--section-title (plist-get section :title)))
    (insert (propertize (format "* %s\n" (plist-get section :title))
                        'face 'outline-1))
    (litgraph-view--insert-blocks (plist-get section :blocks))
    (when (plist-get section :folded)
      (save-excursion
        (goto-char start)
        (outline-hide-subtree)))))

(defun litgraph-view--render (response request)
  "Erase the current buffer and render RESPONSE (for REQUEST) into it."
  (let ((inhibit-read-only t)
        (op (litgraph-get response 'op)))
    (erase-buffer)
    (setq litgraph-view--tables nil)
    (insert (propertize (format "litgraph %s" op) 'face 'litgraph-view-title)
            (propertize (format "  %s  %s ms\n\n"
                                (or (litgraph-view--request-label request) "")
                                (litgraph-view-format-number
                                 (or (litgraph-get response 'elapsed_ms) 0)))
                        'face 'litgraph-view-note))
    (dolist (section (delq nil
                           (append
                            (litgraph-view--sections
                             op (litgraph-get response 'result))
                            (list (litgraph-view--warnings-section response)
                                  (litgraph-view--provenance-section
                                   response)))))
      (litgraph-view--insert-section section))
    (goto-char (point-min))
    (set-buffer-modified-p nil)))


;;;; Mode

(defvar-keymap litgraph-view-mode-map
  :doc "Keymap of `litgraph-view-mode'."
  :parent special-mode-map
  "TAB" #'litgraph-view-cycle
  "<backtab>" #'outline-cycle-buffer
  "n" #'outline-next-visible-heading
  "p" #'outline-previous-visible-heading
  "j" #'litgraph-view-raw-json
  "T" #'litgraph-view-open-table
  "o" #'litgraph-view-copy-org
  "g" #'litgraph-view-rerun
  "G" #'litgraph-view-graph
  "d" #'litgraph-view-deadlines-to-org
  "RET" #'litgraph-view-act
  "?" #'litgraph-menu)

(define-derived-mode litgraph-view-mode special-mode "litgraph"
  "Major mode of litgraph result buffers.

\\{litgraph-view-mode-map}"
  (setq-local outline-regexp "\\* ")
  (setq-local outline-level (lambda () 1))
  (setq-local truncate-lines t)
  (outline-minor-mode 1))

(defun litgraph-view-cycle ()
  "Cycle visibility of the section at point."
  (interactive)
  (if (save-excursion (beginning-of-line) (looking-at-p outline-regexp))
      (outline-cycle)
    (outline-back-to-heading)
    (outline-cycle)))

(defun litgraph-view--buffer-name (response)
  "Buffer name for RESPONSE."
  (format "*litgraph: %s*" (or (litgraph-get response 'op) "result")))

;;;###autoload
(defun litgraph-view-show (response request &optional raw)
  "Display the parsed RESPONSE (for REQUEST, with RAW JSON text) and return it."
  (let ((buffer (get-buffer-create (litgraph-view--buffer-name response))))
    (with-current-buffer buffer
      (unless (derived-mode-p 'litgraph-view-mode) (litgraph-view-mode))
      (setq litgraph-view--response response
            litgraph-view--request request
            litgraph-view--raw (or raw litgraph--last-raw))
      (litgraph-view--render response request))
    (pop-to-buffer buffer litgraph-view-display-action)
    buffer))

(defun litgraph-view--require ()
  "Signal unless in a result buffer."
  (unless litgraph-view--response
    (user-error "Not in a litgraph result buffer")))

(defun litgraph-view-rerun ()
  "Run this buffer's request again."
  (interactive)
  (litgraph-view--require)
  (litgraph-run-request litgraph-view--request))

(defun litgraph-view-raw-json ()
  "Show the raw JSON of this result, pretty-printed."
  (interactive)
  (litgraph-view--require)
  (let ((raw litgraph-view--raw)
        (buffer (get-buffer-create
                 (format "*litgraph-json: %s*"
                         (litgraph-get litgraph-view--response 'op)))))
    (with-current-buffer buffer
      (let ((inhibit-read-only t))
        (erase-buffer)
        (if raw
            (progn (insert raw)
                   (require 'json)
                   (ignore-errors (json-pretty-print-buffer)))
          (pp litgraph-view--response (current-buffer)))
        (goto-char (point-min)))
      (cond ((fboundp 'js-json-mode) (js-json-mode))
            ((fboundp 'json-mode) (json-mode))
            (t (fundamental-mode)))
      (setq buffer-read-only t)
      (local-set-key "q" #'quit-window))
    (pop-to-buffer buffer)))

(defun litgraph-view--table-at-point ()
  "The table at point, or the first one."
  (litgraph-view--require)
  (let ((index (or (get-text-property (point) 'litgraph-table)
                   (and (not (bobp))
                        (get-text-property (1- (point)) 'litgraph-table))
                   0)))
    (or (nth index litgraph-view--tables)
        (user-error "No tables in this result"))))


;;;; Tables as tabulated lists

(defvar-local litgraph-table--table nil "The table shown in this buffer.")
(defvar-local litgraph-table--origin nil "Name of the result buffer it came from.")

(define-derived-mode litgraph-table-mode tabulated-list-mode "litgraph-table"
  "Sortable view of one litgraph table.

\\{litgraph-table-mode-map}"
  (setq tabulated-list-padding 1))

(define-key litgraph-table-mode-map (kbd "RET") #'litgraph-table-act)
(define-key litgraph-table-mode-map "o" #'litgraph-table-copy-org)

(defun litgraph-table--sorter (index numeric)
  "A sort predicate on column INDEX; NUMERIC compares the raw values.
Cells carry their raw value in the `litgraph-raw' text property."
  (lambda (a b)
    (let ((x (litgraph-table--raw (aref (cadr a) index)))
          (y (litgraph-table--raw (aref (cadr b) index))))
      (if (and numeric (numberp x) (numberp y))
          (< x y)
        (string< (format "%s" x) (format "%s" y))))))

(defun litgraph-table--raw (string)
  "The raw value stored on the cell STRING, or STRING itself."
  (if (and (> (length string) 0)
           (get-text-property 0 'litgraph-raw string))
      (get-text-property 0 'litgraph-raw string)
    string))

(defun litgraph-table--cell-string (cell)
  "Display string of CELL carrying its raw value."
  (let ((text (copy-sequence (litgraph-view--cell-text cell)))
        (raw (litgraph-view--cell-raw cell)))
    (if (and (numberp raw) (> (length text) 0))
        (propertize text 'litgraph-raw raw)
      text)))

(defun litgraph-table--show (table name)
  "Show TABLE in a sortable buffer named after NAME."
  (let ((buffer (get-buffer-create (format "*litgraph-table: %s*" name)))
        (widths (litgraph-view--column-widths table))
        (columns (plist-get table :columns))
        (ids (plist-get table :ids)))
    (with-current-buffer buffer
      (litgraph-table-mode)
      (setq litgraph-table--table table
            litgraph-table--origin name
            tabulated-list-format
            (vconcat
             (cl-loop for col in columns for w in widths for i from 0
                      collect (list (car col) (min w litgraph-view-max-column-width)
                                    (litgraph-table--sorter
                                     i (eq (cdr col) 'right))
                                    :right-align (eq (cdr col) 'right))))
            tabulated-list-entries
            (cl-loop for row in (plist-get table :rows)
                     for i from 0
                     collect (list (or (nth i ids) i)
                                   (vconcat (mapcar #'litgraph-table--cell-string
                                                    row)))))
      (tabulated-list-init-header)
      (tabulated-list-print)
      (goto-char (point-min)))
    (pop-to-buffer buffer)))

(defun litgraph-view-open-table ()
  "Open the table at point as a sortable `tabulated-list' buffer."
  (interactive)
  (let ((table (litgraph-view--table-at-point))
        (op (litgraph-get litgraph-view--response 'op)))
    (litgraph-table--show table (or (plist-get table :title) op))))

(defun litgraph-view-copy-org ()
  "Copy the table at point as an Org table (and show it in a buffer)."
  (interactive)
  (let ((text (litgraph-view-table-to-org (litgraph-view--table-at-point))))
    (kill-new text)
    (message "Org table copied (%d lines)"
             (length (split-string text "\n" t)))))

(defun litgraph-table-copy-org ()
  "Copy this table as an Org table."
  (interactive)
  (kill-new (litgraph-view-table-to-org litgraph-table--table))
  (message "Org table copied"))

(defun litgraph-table-act ()
  "Act on the id of the row at point."
  (interactive)
  (when-let* ((id (tabulated-list-get-id)))
    (litgraph-view--act-on-id id)))


;;;; Acting on rows

(defun litgraph-view--edge-source (id)
  "If ID is an edge of the cached graph, its source node; else nil."
  (when-let* ((graph (litgraph-get (gethash (litgraph--graph-key)
                                            litgraph--cache)
                                   'result))
              (edge (seq-find (lambda (e) (equal (litgraph-get e 'id) id))
                              (litgraph-get graph 'edges))))
    (litgraph-get edge 'from)))

(defun litgraph-view--target ()
  "The target of this buffer's request, or nil for the session's."
  (and (hash-table-p litgraph-view--request)
       (gethash "target" litgraph-view--request)))

(defun litgraph-view--act-on-id (id)
  "Drill into ID: sweep a parameter, or explain a node."
  (cond
   ((not (stringp id)) (user-error "This row has no id"))
   ((string-prefix-p "param:" id)
    (litgraph-run-request
     (litgraph-build-request
      (litgraph-obj "op" "sweep" "param" (substring id 6))
      (litgraph-view--target))))
   ((string-match-p "::" id)
    (litgraph-run-request
     (litgraph-build-request
      (litgraph-obj "op" "explain"
                    "node" (or (litgraph-view--edge-source id) id))
      (litgraph-view--target))))
   (t (user-error "Nothing to drill into for %s" id))))

(defun litgraph-view-act ()
  "Drill into the row at point (explain a node, sweep a parameter)."
  (interactive)
  (litgraph-view--act-on-id (get-text-property (point) 'litgraph-id)))

(defun litgraph-view-graph ()
  "Draw the graph of the current target."
  (interactive)
  (litgraph-view--require)
  (require 'litgraph-graph)
  (litgraph-graph))

(defun litgraph-view-deadlines-to-org ()
  "Send this deadlines result to Org."
  (interactive)
  (litgraph-view--require)
  (unless (equal (litgraph-get litgraph-view--response 'op) "deadlines")
    (user-error "This is not a deadlines result"))
  (require 'litgraph-org)
  (litgraph-org-import litgraph-view--response litgraph-view--request))

(provide 'litgraph-view)

;;; litgraph-view.el ends here
