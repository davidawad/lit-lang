;;; litgraph-scenario.el --- Edit and validate litgraph scenario JSON -*- lexical-binding: t; -*-

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

;; Editing support for litgraph JSON documents (scenarios, requests, named
;; scenarios, packs):
;;
;;  - `litgraph-new-scenario' and `litgraph-open-scenario' create or visit
;;    a document;
;;  - `litgraph-scenario-mode' validates the buffer with `litgraph validate'
;;    through flymake (JSON syntax errors first, then the engine's own
;;    checks, located in the buffer as well as the messages allow), completes
;;    ids and field names, and runs the document;
;;  - files ending in .litgraph.json open in `litgraph-scenario-json-mode'.
;;
;; The buffer is validated as text over stdin, so unsaved edits are checked.

;;; Code:

(require 'cl-lib)
(require 'seq)
(require 'subr-x)
(require 'flymake)
(require 'js)
(require 'litgraph)

(declare-function litgraph-graph "litgraph-graph")
(declare-function litgraph-menu "litgraph-menu")
(defvar litgraph-scenario-mode)

(defgroup litgraph-scenario nil
  "Editing litgraph scenario documents."
  :group 'litgraph
  :prefix "litgraph-scenario-")

(defcustom litgraph-scenario-directory nil
  "Directory where scenario files live; nil falls back to `litgraph-scenarios-dir'."
  :type '(choice (const nil) directory))

(defcustom litgraph-flymake-warnings nil
  "Non-nil also reports the engine's warnings, not only errors.
Warnings such as `probability-fill' describe fallbacks the engine used;
they are numerous and rarely point at a place in the file."
  :type 'boolean)

(defvar-local litgraph-scenario-kind nil
  "Document kind passed to `litgraph validate --kind', or nil to auto-detect.")

(defun litgraph-scenario--directory ()
  "Directory offered when opening or creating scenarios."
  (file-name-as-directory
   (expand-file-name (or litgraph-scenario-directory litgraph-scenarios-dir
                         default-directory))))


;;;; Locating messages in the buffer

(defun litgraph-scenario--token-region (token)
  "Region (BEG . END) of the JSON string literal TOKEN in the buffer, or nil."
  (save-excursion
    (goto-char (point-min))
    (when (search-forward (concat "\"" token "\"") nil t)
      (cons (match-beginning 0) (match-end 0)))))

(defun litgraph-scenario--message-token (message)
  "The token of MESSAGE that names something in the document, or nil."
  (cond
   ((string-match "`\\([^`]+\\)`" message) (match-string 1 message))
   ((string-match
     "not found: \\(?:edge\\|node\\|pack\\|scenario\\|set\\) \\([^ ,;]+\\)"
     message)
    (match-string 1 message))
   ((string-match "\\([a-z0-9._-]+::[a-z0-9._-]+\\)" message)
    (match-string 1 message))
   ((string-match "invalid type: [a-z]+ \\\\?\"\\([^\"\\]*\\)\\\\?\"" message)
    (match-string 1 message))))

(defun litgraph-scenario--locate (message)
  "Region (BEG . END) in the buffer for MESSAGE; first line as a fallback."
  (or (when-let* ((token (litgraph-scenario--message-token message)))
        (or (litgraph-scenario--token-region token)
            ;; A bare word: match it as a JSON key or value.
            (save-excursion
              (goto-char (point-min))
              (when (search-forward token nil t)
                (cons (match-beginning 0) (match-end 0))))))
      (save-excursion
        (goto-char (point-min))
        (skip-chars-forward " \t\n")
        (cons (point) (line-end-position)))))

(defun litgraph-scenario--diagnostic (buffer message type)
  "A flymake diagnostic in BUFFER for MESSAGE of TYPE, located best-effort."
  (with-current-buffer buffer
    (save-restriction
      (widen)
      (let ((region (litgraph-scenario--locate message)))
        (flymake-make-diagnostic buffer (car region) (max (cdr region)
                                                          (1+ (car region)))
                                 type message)))))

(defun litgraph-scenario--syntax-error (buffer)
  "If BUFFER is not valid JSON, return a diagnostic for the first error."
  (with-current-buffer buffer
    (save-restriction
      (widen)
      (save-excursion
        (goto-char (point-min))
        (condition-case err
            (progn (json-parse-buffer)
                   (skip-chars-forward " \t\n")
                   (unless (eobp)
                     (flymake-make-diagnostic
                      buffer (point) (min (point-max) (1+ (point)))
                      :error "JSON: unexpected text after the document")))
          (json-parse-error
           (let* ((data (cdr err))
                  (line (and (integerp (nth 0 data)) (nth 0 data)))
                  (col (and (integerp (nth 1 data)) (nth 1 data)))
                  (pos (save-excursion
                         (goto-char (point-min))
                         (when line
                           (forward-line (1- line))
                           (forward-char (min (max 0 (1- (or col 1)))
                                              (- (line-end-position) (point)))))
                         (point))))
             (flymake-make-diagnostic buffer pos (min (point-max) (1+ pos))
                                      :error "JSON syntax error here"))))))))

(defun litgraph-scenario--diagnostics (buffer output)
  "Diagnostics for BUFFER from the parsed `litgraph validate' OUTPUT."
  (append
   (mapcar (lambda (message)
             (litgraph-scenario--diagnostic buffer message :error))
           (litgraph-get output 'errors))
   (mapcar (lambda (d)
             (litgraph-scenario--diagnostic
              buffer
              (format "%s: %s" (or (litgraph-get d 'code) "lint")
                      (litgraph-get d 'message))
              (if (equal (litgraph-get d 'severity) "error") :error :warning)))
           (litgraph-get output 'diagnostics))
   (when litgraph-flymake-warnings
     (mapcar (lambda (w)
               (litgraph-scenario--diagnostic
                buffer
                (format "%s at %s: %s" (litgraph-get w 'code) (litgraph-get w 'at)
                        (litgraph-get w 'message))
                :note))
             (litgraph-get output 'warnings)))))


;;;; Flymake backend

(defvar-local litgraph-flymake--generation 0
  "Counter that discards the results of superseded validation runs.")
(defvar-local litgraph-flymake--process nil
  "The running validation process of this buffer.")

(defun litgraph-flymake (report-fn &rest _args)
  "Flymake backend validating the buffer with `litgraph validate'.
Calls REPORT-FN with the diagnostics; the process is asynchronous."
  (let ((buffer (current-buffer))
        (generation (cl-incf litgraph-flymake--generation)))
    (when (process-live-p litgraph-flymake--process)
      (delete-process litgraph-flymake--process))
    (if-let* ((syntax (litgraph-scenario--syntax-error buffer)))
        (funcall report-fn (list syntax))
      (let* ((input (save-restriction (widen) (buffer-string)))
             (args (append '("validate")
                           (and litgraph-scenario-kind
                                (list "--kind" litgraph-scenario-kind))
                           '("-")))
             (proc
              (funcall
               litgraph-process-function args input
               (lambda (exit stdout stderr)
                 (when (and (buffer-live-p buffer)
                            (= generation
                               (buffer-local-value 'litgraph-flymake--generation
                                                   buffer)))
                   (let ((output (and (stringp stdout)
                                      (ignore-errors (litgraph-parse stdout)))))
                     (if (and output (assq 'valid output))
                         (funcall report-fn
                                  (litgraph-scenario--diagnostics buffer output))
                       (funcall report-fn
                                :panic
                                :explanation
                                (litgraph--describe-failure exit stdout
                                                            stderr)))))))))
        (setq litgraph-flymake--process proc)))))


;;;; Completion

(defun litgraph-scenario--previous-key ()
  "The JSON key whose value the point is in, or nil."
  (save-excursion
    (when (re-search-backward "\"\\([A-Za-z_]+\\)\"[ \t\n]*:[ \t\n]*[^:{}]*\\=" nil t)
      (match-string-no-properties 1))))

(defun litgraph-scenario--string-bounds ()
  "Bounds (BEG . END) of the contents of the JSON string at point, or nil."
  (when (nth 3 (syntax-ppss))
    (let ((beg (1+ (nth 8 (syntax-ppss)))))
      (cons beg (max (point)
                     (save-excursion
                       (goto-char beg)
                       (skip-chars-forward "^\"\n")
                       (point)))))))

(defun litgraph-scenario-capf ()
  "Completion at point for ids, scenarios, packs and fields in a JSON string."
  (when-let* ((bounds (litgraph-scenario--string-bounds))
              (describe (litgraph--cached-describe)))
    (let* ((key (litgraph-scenario--previous-key))
           (graph (litgraph-get (gethash (litgraph--graph-key) litgraph--cache)
                                'result))
           (candidates
            (pcase key
              ("extends" (litgraph-scenario-candidates))
              ("packs" (litgraph-pack-candidates))
              ("calibration" (litgraph-calibration-candidates))
              ("op" (litgraph--op-candidates))
              ((or "metric" "metrics" "cost" "utility")
               (litgraph-metric-candidates))
              ((or "params" "param") (litgraph-param-candidates))
              (_ (append (litgraph-node-candidates graph)
                         (litgraph-edge-candidates graph)
                         (mapcar (lambda (cell) (cons (symbol-name (car cell))
                                                      (cdr cell)))
                                 (litgraph-get describe 'scenario)))))))
      (when candidates
        (list (car bounds) (cdr bounds)
              (litgraph--completion-table candidates 'litgraph)
              :exclusive 'no)))))


;;;; Running

(defun litgraph-scenario--document ()
  "The buffer parsed as a strict JSON document."
  (litgraph-parse-strict (save-restriction (widen) (buffer-string))))

;;;###autoload
(defun litgraph-scenario-run (&optional all)
  "Run the document in this buffer, asking for an op if it has none.
A document with an \"op\" is sent as a whole request; a scenario is run
with the op you choose.  With prefix argument ALL ask every op argument."
  (interactive "P")
  (let ((doc (litgraph-scenario--document)))
    (if (and (hash-table-p doc) (gethash "op" doc))
        (litgraph-run-request doc)
      (let ((op (litgraph-read "litgraph op" (litgraph--op-candidates)
                               'litgraph-op nil t)))
        (litgraph--with-metadata
         op
         (lambda (schema describe graph)
           (litgraph-run-request
            (litgraph-build-request
             (litgraph-read-op-arguments op schema describe graph all)
             doc))))))))

(defun litgraph-scenario-use-buffer ()
  "Make this file the session scenario, saving it first if needed."
  (interactive)
  (unless buffer-file-name (user-error "Save the buffer to a file first"))
  (when (buffer-modified-p) (save-buffer))
  (setq litgraph-scenario buffer-file-name)
  (litgraph-clear-cache)
  (message "litgraph scenario: %s" (litgraph-context-label)))

(defun litgraph-scenario-graph ()
  "Draw the graph of this buffer's scenario."
  (interactive)
  (litgraph-scenario-use-buffer)
  (require 'litgraph-graph)
  (litgraph-graph))

;;;###autoload
(defun litgraph-validate-buffer ()
  "Validate this buffer now and list what is wrong."
  (interactive)
  (unless litgraph-scenario-mode (litgraph-scenario-mode 1))
  (flymake-start)
  (flymake-show-buffer-diagnostics))

(defvar-keymap litgraph-scenario-mode-map
  :doc "Keymap of `litgraph-scenario-mode'."
  "C-c C-c" #'litgraph-scenario-run
  "C-c C-v" #'litgraph-validate-buffer
  "C-c C-g" #'litgraph-scenario-graph
  "C-c C-u" #'litgraph-scenario-use-buffer
  "C-c C-m" #'litgraph-menu)

;;;###autoload
(define-minor-mode litgraph-scenario-mode
  "Validate, complete and run a litgraph JSON document.

\\{litgraph-scenario-mode-map}"
  :lighter " LG"
  :keymap litgraph-scenario-mode-map
  (if litgraph-scenario-mode
      (progn
        (litgraph-prefetch)
        (add-hook 'flymake-diagnostic-functions #'litgraph-flymake nil t)
        (add-hook 'completion-at-point-functions #'litgraph-scenario-capf nil t)
        (flymake-mode 1))
    (remove-hook 'flymake-diagnostic-functions #'litgraph-flymake t)
    (remove-hook 'completion-at-point-functions #'litgraph-scenario-capf t)))

;;;###autoload
(define-derived-mode litgraph-scenario-json-mode js-json-mode "litgraph-JSON"
  "JSON mode for litgraph documents, with validation and completion."
  (litgraph-scenario-mode 1))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.litgraph\\.json\\'" . litgraph-scenario-json-mode))

;;;###autoload
(defun litgraph-scenario-maybe-enable ()
  "Enable `litgraph-scenario-mode' if the buffer resembles a litgraph document.
Suitable for `js-json-mode-hook' and `json-ts-mode-hook'."
  (when (and (derived-mode-p 'js-json-mode 'json-ts-mode)
             (or (and buffer-file-name
                      (litgraph-scenario--in-directory-p buffer-file-name))
                 (save-excursion
                   (goto-char (point-min))
                   (re-search-forward
                    "\"\\(?:op\\|extends\\|packs\\|remove_edges\\|payoffs\\)\"[ \t]*:"
                    4000 t))))
    (litgraph-scenario-mode 1)))

(defun litgraph-scenario--in-directory-p (file)
  "Non-nil if FILE lies under the scenario directory."
  (when-let* ((dir (or litgraph-scenario-directory litgraph-scenarios-dir)))
    (file-in-directory-p file dir)))


;;;; Creating and opening

(defun litgraph-scenario--template (base)
  "Text of a runnable request skeleton extending the library scenario BASE."
  (let* ((scenario (litgraph-obj "extends" base
                                 "params" (litgraph-obj "rate" 500)))
         (request (litgraph-obj
                   "packs" (and (not base) (litgraph--packs)
                                (litgraph-vec (litgraph--packs)))
                   "scenario" scenario
                   "op" (litgraph-obj "op" "solve"))))
    (with-temp-buffer
      (insert (litgraph-serialize request))
      (json-pretty-print-buffer)
      (buffer-string))))

;;;###autoload
(defun litgraph-new-scenario (file &optional base)
  "Create scenario FILE from a runnable skeleton extending library scenario BASE."
  (interactive
   (progn
     (litgraph-prefetch)
     (let* ((base (litgraph-read "Extend library scenario (empty for none)"
                                 (litgraph-scenario-candidates)
                                 'litgraph-scenario))
            (base (and (not (string-empty-p base)) base))
            (default (expand-file-name
                      (concat (or base "scenario") ".litgraph.json")
                      (litgraph-scenario--directory))))
       (list (read-file-name "New scenario file: " nil default nil
                             (file-name-nondirectory default))
             base))))
  (when (and (file-exists-p file)
             (not (y-or-n-p (format "%s exists; open it instead? " file))))
    (user-error "Not overwriting %s" file))
  (unless (file-exists-p file)
    (make-directory (file-name-directory (expand-file-name file)) t)
    (with-temp-file file (insert (litgraph-scenario--template base) "\n")))
  (find-file file)
  (unless litgraph-scenario-mode (litgraph-scenario-mode 1)))

;;;###autoload
(defun litgraph-open-scenario (file)
  "Open the scenario FILE for editing, offering those in the scenario directory."
  (interactive
   (list (read-file-name "Scenario file: " (litgraph-scenario--directory)
                         nil nil nil
                         (lambda (name) (or (file-directory-p name)
                                            (string-suffix-p ".json" name))))))
  (find-file file)
  (unless litgraph-scenario-mode (litgraph-scenario-mode 1)))

(provide 'litgraph-scenario)

;;; litgraph-scenario.el ends here
