;;; litgraph-graph.el --- Graphviz view of litgraph procedure graphs -*- lexical-binding: t; -*-

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

;; `litgraph-graph' asks litgraph for the procedure graph of the current
;; scenario (`graph' op), solves it (`solve' op) and draws the result:
;;
;;  - nodes coloured by who decides (you, the opponent, chance) and
;;    terminals by outcome (win, loss, settlement);
;;  - the solved best line highlighted in red;
;;  - chance edges labelled with their probabilities.
;;
;; When Graphviz `dot' is installed and Emacs can display images, the
;; drawing appears inline as SVG (or PNG).  Otherwise, or on demand with
;; \\`t', an indented text tree shows the same information.  With a prefix
;; argument, draw only the neighborhood of a node.

;;; Code:

(require 'cl-lib)
(require 'seq)
(require 'subr-x)
(require 'litgraph)
(require 'text-property-search)

(declare-function litgraph-view--act-on-id "litgraph-view")
(declare-function image-increase-size "image-mode")
(declare-function image-decrease-size "image-mode")

(defgroup litgraph-graph nil
  "Graph drawing for litgraph."
  :group 'litgraph
  :prefix "litgraph-graph-")

(defcustom litgraph-graph-dot-executable "dot"
  "Graphviz `dot' program."
  :type 'string)

(defcustom litgraph-graph-rankdir "TB"
  "Graphviz rank direction: TB (top to bottom) or LR (left to right)."
  :type '(choice (const "TB") (const "LR")))

(defcustom litgraph-graph-image-type 'svg
  "Image type requested from Graphviz: `svg' or `png'."
  :type '(choice (const svg) (const png)))

(defcustom litgraph-graph-edge-labels 'p
  "What to write on edges: nil, `p' (probability), `label' or `both'."
  :type '(choice (const :tag "Nothing" nil) (const p) (const label) (const both)))

(defcustom litgraph-graph-show-inactive nil
  "Non-nil draws edges the scenario has removed, dashed."
  :type 'boolean)

(defcustom litgraph-graph-highlight-color "#d62728"
  "Color of the solved best line."
  :type 'string)

(defcustom litgraph-graph-control-colors
  '(("me" . "#cfe4ff") ("opponent" . "#ffd9c7") ("chance" . "#ececec"))
  "Fill colors by who controls a node."
  :type '(alist :key-type string :value-type string))

(defcustom litgraph-graph-outcome-colors
  '(("win" . "#c9f2c7") ("loss" . "#f6c9c9") ("settlement" . "#fff1b8"))
  "Fill colors of terminal nodes by outcome tag."
  :type '(alist :key-type string :value-type string))

(defcustom litgraph-graph-text-max-lines 1500
  "Longest the text tree may grow."
  :type 'natnum)


;;;; Dot

(defun litgraph-graph--quote (string)
  "Quote STRING as a dot identifier or label."
  (concat "\""
          (replace-regexp-in-string
           "\n" "\\\\n"
           (replace-regexp-in-string
            "\"" "\\\\\""
            (replace-regexp-in-string "\\\\" "\\\\\\\\" (format "%s" string))))
          "\""))

(defun litgraph-graph--wrap (text width)
  "Wrap TEXT to WIDTH columns, joined by newlines."
  (let ((lines nil) (line ""))
    (dolist (word (split-string (or text "") "[ \n]+" t))
      (if (and (> (length line) 0)
               (> (+ (length line) 1 (length word)) width))
          (progn (push line lines) (setq line word))
        (setq line (if (string-empty-p line) word (concat line " " word)))))
    (unless (string-empty-p line) (push line lines))
    (mapconcat #'identity (nreverse lines) "\n")))

(defun litgraph-graph--terminal-color (node)
  "Fill color for the terminal NODE from its outcome tags."
  (let ((tags (mapcar (lambda (tag) (format "%s" tag))
                      (litgraph-get node 'outcome))))
    (or (cl-loop for (tag . color) in litgraph-graph-outcome-colors
                 when (member tag tags) return color)
        "#ffffff")))

(defun litgraph-graph--node-color (node)
  "Fill color of NODE."
  (if (equal (litgraph-get node 'kind) "terminal")
      (litgraph-graph--terminal-color node)
    (or (cdr (assoc (litgraph-get node 'control) litgraph-graph-control-colors))
        "#ffffff")))

(defun litgraph-graph--node-shape (node)
  "Shape attributes of NODE."
  (pcase (litgraph-get node 'control)
    ("me" "shape=box, style=\"rounded,filled\"")
    ("opponent" "shape=box, style=\"filled\"")
    ("terminal" "shape=box, style=\"filled\", peripheries=2")
    (_ "shape=ellipse, style=\"filled\"")))

(defun litgraph-graph--best-nodes (graph best-edges)
  "Ids of the nodes touched by the BEST-EDGES ids in GRAPH."
  (let ((nodes (make-hash-table :test #'equal)))
    (dolist (edge (litgraph-get graph 'edges))
      (when (member (litgraph-get edge 'id) best-edges)
        (puthash (litgraph-get edge 'from) t nodes)
        (puthash (litgraph-get edge 'to) t nodes)))
    nodes))

(defun litgraph-graph--node-line (node on-best)
  "The dot statement for NODE; ON-BEST when it lies on the best line."
  (let* ((label (litgraph-graph--wrap (litgraph-get node 'label) 26))
         (payoff (litgraph-get node 'payoff))
         (label (if (and (numberp payoff) (not (zerop payoff)))
                    (format "%s\n%s" label (litgraph-graph--money payoff))
                  label)))
    (format "  %s [label=%s, %s, fillcolor=%s%s];\n"
            (litgraph-graph--quote (litgraph-get node 'id))
            (litgraph-graph--quote label)
            (litgraph-graph--node-shape node)
            (litgraph-graph--quote (litgraph-graph--node-color node))
            (if on-best
                (format ", color=%s, penwidth=2.5"
                        (litgraph-graph--quote litgraph-graph-highlight-color))
              ""))))

(defun litgraph-graph--money (amount)
  "AMOUNT as dollars with thousands separators."
  (let ((digits (number-to-string (round (abs amount)))) (out nil) (n 0))
    (dolist (char (nreverse (string-to-list digits)))
      (when (and (> n 0) (zerop (% n 3))) (push ?, out))
      (push char out)
      (cl-incf n))
    (concat (if (< amount 0) "-$" "$") (concat out))))

(defun litgraph-graph--edge-label (edge)
  "The label text of EDGE according to `litgraph-graph-edge-labels'."
  (let* ((p (litgraph-get edge 'p))
         (chance (and (numberp p) (< p 0.9995)))
         (ptext (and chance (format "%.0f%%" (* 100 p))))
         (label (litgraph-graph--wrap (litgraph-get edge 'label) 22)))
    (pcase litgraph-graph-edge-labels
      ('p ptext)
      ('label label)
      ('both (string-join (delq nil (list ptext label)) "\n"))
      (_ nil))))

(defun litgraph-graph--edge-line (edge best-edges)
  "The dot statement for EDGE, highlighted when in BEST-EDGES."
  (let* ((best (member (litgraph-get edge 'id) best-edges))
         (label (litgraph-graph--edge-label edge))
         (attrs (delq nil
                      (list
                       (and label (format "label=%s" (litgraph-graph--quote label)))
                       (and best (format "color=%s, penwidth=2.5"
                                         (litgraph-graph--quote
                                          litgraph-graph-highlight-color)))
                       (and (not (litgraph-get edge 'active))
                            (assq 'active edge)
                            "style=dashed, color=\"#aaaaaa\"")
                       (and (equal (litgraph-get edge 'role) "opponent")
                            (not best) "color=\"#c0552b\"")
                       (and (equal (litgraph-get edge 'role) "self")
                            (not best) "color=\"#2b6cc0\"")))))
    (format "  %s -> %s [%s];\n"
            (litgraph-graph--quote (litgraph-get edge 'from))
            (litgraph-graph--quote (litgraph-get edge 'to))
            (string-join attrs ", "))))

(defun litgraph-graph-dot (graph &optional best-edges title)
  "Return Graphviz source for the `graph' result GRAPH.
BEST-EDGES is a list of edge ids on the solved best line, highlighted.
TITLE labels the drawing."
  (let ((best-nodes (litgraph-graph--best-nodes graph best-edges)))
    (concat
     "digraph litgraph {\n"
     (format "  rankdir=%s;\n  nodesep=0.3; ranksep=0.5;\n" litgraph-graph-rankdir)
     "  node [fontname=\"Helvetica\", fontsize=10, margin=\"0.08,0.04\"];\n"
     "  edge [fontname=\"Helvetica\", fontsize=9, arrowsize=0.7];\n"
     (when title
       (format "  labelloc=t; fontsize=12; label=%s;\n"
               (litgraph-graph--quote title)))
     (mapconcat (lambda (node)
                  (litgraph-graph--node-line
                   node (gethash (litgraph-get node 'id) best-nodes)))
                (litgraph-get graph 'nodes) "")
     (mapconcat (lambda (edge)
                  (if (or litgraph-graph-show-inactive
                          (not (assq 'active edge))
                          (litgraph-get edge 'active))
                      (litgraph-graph--edge-line edge best-edges)
                    ""))
                (litgraph-get graph 'edges) "")
     "}\n")))


;;;; Text fallback

(defconst litgraph-graph--control-glyphs
  '(("me" . "◆") ("opponent" . "◇") ("chance" . "○") ("terminal" . "■"))
  "Glyph for each kind of node.")

(defun litgraph-graph-text (graph &optional best-edges start)
  "Indented text tree of GRAPH from START, marking BEST-EDGES with a star.
The result is a list of (LINE . NODE-ID) pairs."
  (let* ((nodes (make-hash-table :test #'equal))
         (children (make-hash-table :test #'equal))
         (has-parent (make-hash-table :test #'equal))
         (seen (make-hash-table :test #'equal))
         (lines nil)
         (count 0))
    (dolist (node (litgraph-get graph 'nodes))
      (puthash (litgraph-get node 'id) node nodes))
    (dolist (edge (litgraph-get graph 'edges))
      (when (or litgraph-graph-show-inactive
                (not (assq 'active edge))
                (litgraph-get edge 'active))
        (puthash (litgraph-get edge 'from)
                 (append (gethash (litgraph-get edge 'from) children)
                         (list edge))
                 children)
        (puthash (litgraph-get edge 'to) t has-parent)))
    (let ((start (or start
                     (litgraph-get (seq-find
                                    (lambda (n)
                                      (not (gethash (litgraph-get n 'id)
                                                    has-parent)))
                                    (litgraph-get graph 'nodes))
                                   'id))))
      (cl-labels
          ((glyph (node)
             (or (cdr (assoc (if (equal (litgraph-get node 'kind) "terminal")
                                 "terminal"
                               (litgraph-get node 'control))
                             litgraph-graph--control-glyphs))
                 "·"))
           (emit (line id)
             (when (< count litgraph-graph-text-max-lines)
               (push (cons line id) lines))
             (cl-incf count))
           (describe-node (id)
             (let ((node (gethash id nodes)))
               (format "%s %s%s" (glyph node)
                       (or (litgraph-get node 'label) id)
                       (if-let* (((equal (litgraph-get node 'kind) "terminal"))
                                 (payoff (litgraph-get node 'payoff))
                                 ((numberp payoff)))
                           (format "  [%s]" (litgraph-graph--money payoff))
                         ""))))
           (walk (id prefix)
             (let* ((edges (gethash id children))
                    (n (length edges)))
               (cl-loop
                for edge in edges for i from 1
                for last = (= i n)
                for to = (litgraph-get edge 'to)
                for best = (member (litgraph-get edge 'id) best-edges)
                for p = (litgraph-get edge 'p)
                do
                (emit (format "%s%s%s%s%s"
                              prefix (if last "└─" "├─")
                              (if best "★" " ")
                              (if (and (numberp p) (< p 0.9995))
                                  (format "(%.0f%%) " (* 100 p)) "")
                              (if (gethash to seen)
                                  (format "%s  ↺ shown above" (describe-node to))
                                (describe-node to)))
                      to)
                (unless (gethash to seen)
                  (puthash to t seen)
                  (walk to (concat prefix (if last "   " "│  "))))))))
        (when start
          (puthash start t seen)
          (emit (describe-node start) start)
          (walk start ""))
        (when (> count litgraph-graph-text-max-lines)
          (push (cons (format "… %d more lines not shown" (- count litgraph-graph-text-max-lines))
                      nil)
                lines))
        (nreverse lines)))))


;;;; Buffer

(defvar-local litgraph-graph--dot nil "Dot source shown in this buffer.")
(defvar-local litgraph-graph--text nil "Text tree lines shown in this buffer.")
(defvar-local litgraph-graph--image nil "Image data (a string) of the drawing.")
(defvar-local litgraph-graph--title nil "Title of the drawing.")
(defvar-local litgraph-graph--mode-view 'image "Either `image' or `text'.")

(defvar-keymap litgraph-graph-mode-map
  :doc "Keymap of `litgraph-graph-mode'."
  :parent special-mode-map
  "t" #'litgraph-graph-toggle-text
  "d" #'litgraph-graph-show-dot
  "w" #'litgraph-graph-write
  "g" #'litgraph-graph
  "+" #'litgraph-graph-zoom-in
  "-" #'litgraph-graph-zoom-out
  "RET" #'litgraph-graph-act)

(define-derived-mode litgraph-graph-mode special-mode "litgraph-graph"
  "Major mode showing a litgraph procedure graph.

\\{litgraph-graph-mode-map}"
  (setq-local truncate-lines t))

(defun litgraph-graph--image-possible-p ()
  "Non-nil when a Graphviz image can be produced and displayed."
  (and (display-graphic-p)
       (executable-find litgraph-graph-dot-executable)
       (image-type-available-p litgraph-graph-image-type)))

(defun litgraph-graph--redraw ()
  "Insert the current view (image or text tree) into the buffer."
  (let ((inhibit-read-only t))
    (erase-buffer)
    (insert (propertize (concat (or litgraph-graph--title "litgraph graph") "\n")
                        'face 'bold))
    (insert (propertize
             "◆ you  ◇ opponent  ○ chance  ■ terminal  ★ solved best line   [t text/image  d dot  w write  +/- zoom]\n\n"
             'face 'shadow))
    (if (and (eq litgraph-graph--mode-view 'image) litgraph-graph--image)
        (insert-image (create-image litgraph-graph--image
                                    litgraph-graph-image-type t))
      (dolist (entry litgraph-graph--text)
        (insert (propertize (car entry)
                            'litgraph-id (cdr entry)
                            'face (and (string-match-p "★" (car entry))
                                       `(:foreground ,litgraph-graph-highlight-color)))
                "\n")))
    (goto-char (point-min))
    (set-buffer-modified-p nil)))

(defun litgraph-graph-toggle-text ()
  "Switch between the drawing and the text tree."
  (interactive)
  (setq litgraph-graph--mode-view
        (if (eq litgraph-graph--mode-view 'image) 'text 'image))
  (when (and (eq litgraph-graph--mode-view 'image)
             (not litgraph-graph--image))
    (setq litgraph-graph--mode-view 'text)
    (message "No image available (needs Graphviz and a graphical Emacs)"))
  (litgraph-graph--redraw))

(defun litgraph-graph-show-dot ()
  "Show the Graphviz source in its own buffer."
  (interactive)
  (let ((dot litgraph-graph--dot))
    (with-current-buffer (get-buffer-create "*litgraph-dot*")
      (let ((inhibit-read-only t))
        (erase-buffer)
        (insert dot)
        (goto-char (point-min)))
      (if (fboundp 'graphviz-dot-mode) (graphviz-dot-mode) (fundamental-mode))
      (pop-to-buffer (current-buffer)))))

(defun litgraph-graph-write (file)
  "Write the drawing to FILE (.dot, .svg or .png) — dot source or image."
  (interactive "FWrite graph to (.dot/.svg/.png): ")
  (cond
   ((string-suffix-p ".dot" file)
    (with-temp-file file (insert litgraph-graph--dot)))
   ((and litgraph-graph--image
         (string-suffix-p (format ".%s" litgraph-graph-image-type) file))
    (let ((coding-system-for-write 'no-conversion))
      (with-temp-file file (set-buffer-multibyte nil)
                      (insert litgraph-graph--image))))
   (t (litgraph-graph--dot-to-file litgraph-graph--dot file)))
  (message "Wrote %s" file))

(defun litgraph-graph--dot-to-file (dot file)
  "Render DOT to FILE with Graphviz, choosing the format from FILE's extension."
  (unless (executable-find litgraph-graph-dot-executable)
    (user-error "Graphviz `%s' not found" litgraph-graph-dot-executable))
  (let ((type (or (file-name-extension file) "svg")))
    (with-temp-buffer
      (insert dot)
      (unless (zerop (call-process-region (point-min) (point-max)
                                          litgraph-graph-dot-executable t nil
                                          nil (concat "-T" type) "-o"
                                          (expand-file-name file)))
        (error "Graphviz failed")))))

(defun litgraph-graph--image-position ()
  "Buffer position of the drawing, or nil when the text tree is shown."
  (save-excursion
    (goto-char (point-min))
    (let ((match (text-property-search-forward 'display nil
                                               (lambda (_ value)
                                                 ;; Not `imagep': Emacs
                                                 ;; built without image
                                                 ;; support lacks it.
                                                 (eq (car-safe value) 'image)))))
      (and match (prop-match-beginning match)))))

(defun litgraph-graph--zoom (command)
  "Run the image size COMMAND (increase or decrease) on the drawing."
  (require 'image-mode)
  (if-let* ((position (litgraph-graph--image-position)))
      (funcall command 2 position)
    (user-error "No image shown; press t for the drawing")))

(defun litgraph-graph-zoom-in ()
  "Enlarge the drawing."
  (interactive)
  (litgraph-graph--zoom #'image-increase-size))

(defun litgraph-graph-zoom-out ()
  "Shrink the drawing."
  (interactive)
  (litgraph-graph--zoom #'image-decrease-size))

(defun litgraph-graph-act ()
  "Explain the node on this line of the text tree."
  (interactive)
  (if-let* ((id (get-text-property (point) 'litgraph-id)))
      (progn (require 'litgraph-view)
             (litgraph-view--act-on-id id))
    (user-error "No node on this line")))

(defun litgraph-graph--render-image (dot callback)
  "Run Graphviz over DOT asynchronously; call CALLBACK with image data or nil."
  (let* ((out (generate-new-buffer " *litgraph-dot-out*" t))
         (err (generate-new-buffer " *litgraph-dot-err*" t))
         (proc (make-process
                :name "litgraph-dot"
                :buffer out
                :stderr err
                :command (list (executable-find litgraph-graph-dot-executable)
                               (format "-T%s" litgraph-graph-image-type))
                :coding 'no-conversion
                :connection-type 'pipe
                :noquery t
                :sentinel
                (lambda (process _event)
                  (unless (process-live-p process)
                    (let ((data (with-current-buffer out (buffer-string)))
                          (ok (zerop (process-exit-status process))))
                      (kill-buffer out)
                      (kill-buffer err)
                      (funcall callback (and ok (> (length data) 0) data))))))))
    (set-process-query-on-exit-flag (get-buffer-process err) nil)
    (process-send-string proc dot)
    (process-send-eof proc)
    proc))

(defun litgraph-graph-show (graph best-edges title)
  "Display GRAPH with BEST-EDGES highlighted under TITLE."
  (let ((buffer (get-buffer-create "*litgraph-graph*"))
        (dot (litgraph-graph-dot graph best-edges title))
        (text (litgraph-graph-text graph best-edges)))
    (with-current-buffer buffer
      (unless (derived-mode-p 'litgraph-graph-mode) (litgraph-graph-mode))
      (setq litgraph-graph--dot dot
            litgraph-graph--text text
            litgraph-graph--title title
            litgraph-graph--image nil
            litgraph-graph--mode-view 'text)
      (litgraph-graph--redraw))
    (pop-to-buffer buffer)
    (when (litgraph-graph--image-possible-p)
      (litgraph-graph--render-image
       dot
       (lambda (data)
         (when (buffer-live-p buffer)
           (with-current-buffer buffer
             (when data
               (setq litgraph-graph--image data
                     litgraph-graph--mode-view 'image)
               (litgraph-graph--redraw)))))))
    buffer))

(defun litgraph-graph--best-edge-ids (solve-response)
  "Ids of the best-line edges in SOLVE-RESPONSE (or nil)."
  (mapcar (lambda (step) (litgraph-get step 'id))
          (litgraph-get solve-response 'result 'best_line)))

;;;###autoload
(defun litgraph-graph (&optional neighborhood)
  "Draw the procedure graph of the current scenario, best line highlighted.
With prefix argument NEIGHBORHOOD, ask for a node and draw only its
neighborhood.  Asynchronous; the buffer appears when the data arrives."
  (interactive "P")
  (litgraph-with-graph
   (lambda (full)
     (let ((node (and neighborhood
                      (litgraph-read "Neighborhood of node"
                                     (litgraph-node-candidates full)
                                     'litgraph-node nil t))))
       (if node
           (litgraph-request
            (litgraph-build-request (litgraph-obj "op" "graph" "node" node))
            (lambda (response)
              (litgraph-graph--solve-and-show
               (litgraph-get response 'result) node)))
         (litgraph-graph--solve-and-show full nil))))))

(defun litgraph-graph--solve-and-show (graph node)
  "Solve the current scenario, then show GRAPH (around NODE) with the best line."
  (litgraph-request
   (litgraph-build-request "solve")
   (lambda (solve-response)
     (litgraph-graph-show graph (litgraph-graph--best-edge-ids solve-response)
                          (litgraph-graph--title solve-response node)))
   (lambda (_message &optional _response)
     (litgraph-graph-show graph nil (litgraph-graph--title nil node)))))

(defun litgraph-graph--title (solve-response node)
  "Title for a drawing of the current context around NODE.
SOLVE-RESPONSE may be nil."
  (string-join
   (delq nil
         (list (litgraph-context-label)
               (and node (format "neighborhood of %s" node))
               (when-let* ((value (litgraph-get solve-response 'result 'value)))
                 (format "solved value %s" (litgraph-graph--money value)))))
   "  ·  "))

(provide 'litgraph-graph)

;;; litgraph-graph.el ends here
