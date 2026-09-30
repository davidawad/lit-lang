;;; litgraph-org.el --- Litigation deadlines as Org agenda items -*- lexical-binding: t; -*-

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

;; `litgraph-deadlines-to-org' asks litgraph for concrete due dates (FRCP 6,
;; RCFC 6, FRAP 26, 19 CFR 210.6(a) counting, with weekend/holiday rolls)
;; from a triggering event and files each as an Org heading with a DEADLINE
;; timestamp, so the dates appear in the agenda:
;;
;;   * Matter: Acme v. United States                       :litgraph:
;;   :PROPERTIES:
;;   :LITGRAPH_MATTER: Acme v. United States
;;   :CATEGORY: Acme v. United States
;;   :END:
;;   ** TODO United States answers (RCFC 12(a)(1)(A))     :deadline:
;;   DEADLINE: <2026-11-30 Mon>
;;   :PROPERTIES:
;;   :LITGRAPH_ID: Acme v. United States|cofc::e-responsefork-answer
;;   :LITGRAPH_TRIGGER: 2026-10-01
;;   ...
;;   :END:
;;   :LITGRAPH_STEPS:
;;   RCFC 6(a)(1)(A): exclude the trigger date 2026-10-01
;;   ...
;;   :END:
;;
;; Filing is idempotent.  Each deadline carries a LITGRAPH_ID (matter plus
;; graph edge); running the command again with a corrected trigger date
;; updates the DEADLINE, the properties and the computation steps in place
;; and leaves your TODO state, notes and other edits alone.

;;; Code:

(require 'cl-lib)
(require 'subr-x)
(require 'org)
(require 'litgraph)

(declare-function org-agenda-files "org")
(defvar org-agenda-files)
(defvar org-directory)

(defgroup litgraph-org nil
  "Filing litgraph deadlines into Org."
  :group 'litgraph
  :prefix "litgraph-org-")

(defcustom litgraph-org-file nil
  "Org file that receives deadlines.
Nil means litgraph-deadlines.org inside `org-directory'."
  :type '(choice (const :tag "org-directory/litgraph-deadlines.org" nil) file))

(defcustom litgraph-org-todo-keyword "TODO"
  "Keyword given to new deadline headings; nil for none."
  :type '(choice (const :tag "None" nil) string))

(defcustom litgraph-org-tags '("deadline")
  "Tags put on each deadline heading."
  :type '(repeat string))

(defcustom litgraph-org-matter-tags '("litgraph")
  "Tags put on each matter heading."
  :type '(repeat string))

(defcustom litgraph-org-warning-period nil
  "Org warning period appended to DEADLINE stamps, for example \"-14d\".
Nil leaves Org's default `org-deadline-warning-days' in charge."
  :type '(choice (const nil) string))

(defcustom litgraph-org-save t
  "Non-nil saves the Org file after filing."
  :type 'boolean)

(defcustom litgraph-org-jump t
  "Non-nil shows the matter in the Org file after filing."
  :type 'boolean)

(defcustom litgraph-org-offer-agenda t
  "Non-nil offers to add the file to `org-agenda-files' when it is missing."
  :type 'boolean)

(defcustom litgraph-org-title-function #'litgraph-org-default-title
  "Function turning a deadline (alist) into a heading title."
  :type 'function)

(defcustom litgraph-org-after-import-hook nil
  "Hook run after deadlines were filed, in the Org buffer."
  :type 'hook)

(defvar litgraph-org--offered nil
  "Files for which adding to the agenda was already offered this session.")


;;;; Pure formatting

(defun litgraph-org--one-line (string)
  "STRING squeezed onto one line, without a leading star."
  (string-trim
   (replace-regexp-in-string
    "\\`\\*+ *" ""
    (replace-regexp-in-string "[ \t\n]+" " " (or string "")))))

(defun litgraph-org-default-title (deadline)
  "Default heading title for DEADLINE: the edge label and its authority."
  (let ((label (litgraph-org--one-line (litgraph-get deadline 'edge 'label)))
        (authority (litgraph-org--one-line
                    (litgraph-get deadline 'edge 'authority))))
    (truncate-string-to-width
     (if (string-empty-p authority) label (format "%s (%s)" label authority))
     140 nil nil "…")))

(defun litgraph-org-timestamp (date)
  "Org active timestamp for DATE (\"YYYY-MM-DD\"), with weekday and warning."
  (unless (string-match "\\`\\([0-9]\\{4\\}\\)-\\([0-9]\\{2\\}\\)-\\([0-9]\\{2\\}\\)\\'"
                        date)
    (error "Bad date `%s'" date))
  (let* ((year (string-to-number (match-string 1 date)))
         (month (string-to-number (match-string 2 date)))
         (day (string-to-number (match-string 3 date)))
         (system-time-locale "C")
         (weekday (format-time-string "%a" (encode-time 0 0 12 day month year))))
    (format "<%s %s%s>" date weekday
            (if litgraph-org-warning-period
                (concat " " litgraph-org-warning-period)
              ""))))

(defun litgraph-org--key (matter deadline)
  "The idempotency key of DEADLINE in MATTER."
  (format "%s|%s" matter (litgraph-get deadline 'edge 'id)))

(defun litgraph-org--properties (matter deadline)
  "Alist of property drawer entries for DEADLINE in MATTER."
  (delq nil
        (list
         (cons "LITGRAPH_ID" (litgraph-org--key matter deadline))
         (cons "LITGRAPH_EDGE" (litgraph-get deadline 'edge 'id))
         (cons "LITGRAPH_TRIGGER" (litgraph-get deadline 'trigger))
         (cons "LITGRAPH_PERIOD"
               (format "%s %s days"
                       (let ((n (litgraph-get deadline 'deadline_length)))
                         (if (and (numberp n) (= n (round n))) (round n) n))
                       (or (litgraph-get deadline 'unit) "calendar")))
         (when-let* ((rule (litgraph-get deadline 'rule_set_label)))
           (cons "LITGRAPH_RULES" rule))
         (when-let* ((authority (litgraph-get deadline 'edge 'authority)))
           (cons "LITGRAPH_AUTHORITY" (litgraph-org--one-line authority)))
         (when (litgraph-get deadline 'last_day_rolled)
           (cons "LITGRAPH_ROLLED" "yes"))
         (when-let* ((url (litgraph-get deadline 'source_url)))
           (cons "LITGRAPH_SOURCE" url)))))

(defun litgraph-org--steps (deadline)
  "Lines explaining how DEADLINE's date was computed."
  (append
   (list (format "Due %s (%s)." (litgraph-get deadline 'due_date)
                 (or (litgraph-get deadline 'rule_set_label) "computed")))
   (litgraph-get deadline 'steps)))

(defun litgraph-org--heading-line (deadline level)
  "The heading line of DEADLINE at LEVEL."
  (let ((tags (and litgraph-org-tags
                   (format " :%s:" (string-join litgraph-org-tags ":")))))
    (format "%s %s%s%s"
            (make-string level ?*)
            (if litgraph-org-todo-keyword
                (concat litgraph-org-todo-keyword " ") "")
            (funcall litgraph-org-title-function deadline)
            (or tags ""))))

(defun litgraph-org-entry-string (deadline matter &optional level)
  "The Org subtree for DEADLINE in MATTER, at LEVEL (default 2)."
  (let ((level (or level 2)))
    (concat
     (litgraph-org--heading-line deadline level) "\n"
     "DEADLINE: " (litgraph-org-timestamp (litgraph-get deadline 'due_date))
     "\n"
     ":PROPERTIES:\n"
     (mapconcat (lambda (p) (format ":%s: %s\n" (car p) (cdr p)))
                (litgraph-org--properties matter deadline) "")
     ":END:\n"
     ":LITGRAPH_STEPS:\n"
     (mapconcat (lambda (line) (concat line "\n"))
                (litgraph-org--steps deadline) "")
     ":END:\n")))

(defun litgraph-org-matter-string (matter)
  "The Org heading that opens MATTER."
  (concat "* Matter: " matter
          (if litgraph-org-matter-tags
              (format " :%s:" (string-join litgraph-org-matter-tags ":"))
            "")
          "\n"
          ":PROPERTIES:\n"
          (format ":LITGRAPH_MATTER: %s\n" matter)
          (format ":CATEGORY: %s\n" matter)
          ":END:\n"))


;;;; Filing into a buffer

(defun litgraph-org--find-key (key)
  "Position of the heading whose LITGRAPH_ID is KEY, or nil."
  (org-find-property "LITGRAPH_ID" key))

(defun litgraph-org--ensure-matter (matter)
  "Return the position of MATTER's heading, creating it at the end."
  (or (org-find-property "LITGRAPH_MATTER" matter)
      (progn
        (goto-char (point-max))
        (unless (bolp) (insert "\n"))
        (let ((pos (point)))
          (insert (litgraph-org-matter-string matter))
          pos))))

(defun litgraph-org--update-planning (deadline)
  "Set the DEADLINE stamp of the entry at point from DEADLINE."
  (let ((stamp (litgraph-org-timestamp (litgraph-get deadline 'due_date)))
        (limit (save-excursion (forward-line 1) (line-end-position))))
    (save-excursion
      (forward-line 1)
      (cond
       ((looking-at-p "[ \t]*\\(DEADLINE\\|SCHEDULED\\|CLOSED\\):")
        (if (re-search-forward "DEADLINE: *<[^>]*>" limit t)
            (replace-match (concat "DEADLINE: " stamp) t t)
          (end-of-line)
          (insert " DEADLINE: " stamp)))
       (t (insert "DEADLINE: " stamp "\n"))))))

(defun litgraph-org--update-steps (deadline)
  "Replace the computation drawer of the entry at point from DEADLINE."
  (let* ((end (save-excursion (org-end-of-subtree t t) (point)))
         (text (concat ":LITGRAPH_STEPS:\n"
                       (mapconcat (lambda (l) (concat l "\n"))
                                  (litgraph-org--steps deadline) "")
                       ":END:\n")))
    (save-excursion
      (org-back-to-heading t)
      (if (re-search-forward "^[ \t]*:LITGRAPH_STEPS:[ \t]*\n\\(?:.*\n\\)*?[ \t]*:END:[ \t]*\n"
                             end t)
          (replace-match text t t)
        (goto-char (or (and (re-search-forward "^[ \t]*:END:[ \t]*\n" end t)
                            (point))
                       (progn (forward-line 1) (point))))
        (insert text)))))

(defun litgraph-org--update-entry (pos matter deadline)
  "Refresh the entry at POS from DEADLINE in MATTER; keep user edits."
  (save-excursion
    (goto-char pos)
    (litgraph-org--update-planning deadline)
    (dolist (property (litgraph-org--properties matter deadline))
      (org-entry-put nil (car property) (cdr property)))
    (litgraph-org--update-steps deadline)))

(defun litgraph-org--add-entry (matter-pos matter deadline)
  "Append DEADLINE for MATTER as a new child of the heading at MATTER-POS."
  (save-excursion
    (goto-char matter-pos)
    (let ((level (org-current-level)))
      (org-end-of-subtree t t)
      (unless (bolp) (insert "\n"))
      (insert (litgraph-org-entry-string deadline matter (1+ level))))))

(defun litgraph-org-file-deadlines (deadlines matter)
  "File DEADLINES (list of alists) for MATTER into the current Org buffer.
Return a plist (:added N :updated N :position POS)."
  (let ((added 0) (updated 0))
    (save-excursion
      (litgraph-org--ensure-matter matter)
      (dolist (deadline deadlines)
        (if-let* ((pos (litgraph-org--find-key
                        (litgraph-org--key matter deadline))))
            (progn (litgraph-org--update-entry pos matter deadline)
                   (cl-incf updated))
          (litgraph-org--add-entry
           (org-find-property "LITGRAPH_MATTER" matter) matter deadline)
          (cl-incf added))))
    (list :added added :updated updated
          :position (org-find-property "LITGRAPH_MATTER" matter))))


;;;; Destination and agenda

(defun litgraph-org--destination ()
  "The Org file to file into."
  (expand-file-name
   (or litgraph-org-file
       (expand-file-name "litgraph-deadlines.org"
                         (or (bound-and-true-p org-directory) "~/org")))))

(defun litgraph-org--in-agenda-p (file)
  "Non-nil when FILE is covered by `org-agenda-files'."
  (member (file-truename file)
          (mapcar #'file-truename (ignore-errors (org-agenda-files t)))))

(defun litgraph-org--offer-agenda (file)
  "Offer once to add FILE to `org-agenda-files' for this session."
  (when (and litgraph-org-offer-agenda
             (not (member file litgraph-org--offered))
             (listp org-agenda-files)
             (not (litgraph-org--in-agenda-p file)))
    (push file litgraph-org--offered)
    (if (y-or-n-p (format "%s is not in `org-agenda-files'; add it for this session? "
                          (abbreviate-file-name file)))
        (progn (setq org-agenda-files (append org-agenda-files (list file)))
               (message "Added to org-agenda-files for this session.  To keep it: (add-to-list 'org-agenda-files %S)"
                        file))
      (message "Deadlines will not show in the agenda until %s is in `org-agenda-files'"
               (abbreviate-file-name file)))))

(defun litgraph-org--matter-default (request)
  "A default matter name derived from REQUEST."
  (let ((scenario (and (hash-table-p request) (gethash "scenario" request))))
    (cond ((stringp scenario) scenario)
          ((and (hash-table-p scenario) (gethash "extends" scenario)))
          (t "litgraph matter"))))

(defun litgraph-org--read-matter (request)
  "Ask for the matter name, defaulting from REQUEST."
  (let ((default (litgraph-org--matter-default request)))
    (string-trim
     (read-string (format-prompt "Matter (heading in the Org file)" default)
                  nil nil default))))

;;;###autoload
(defun litgraph-org-import (response request &optional matter)
  "File the deadlines in RESPONSE (from REQUEST) into Org under MATTER.
MATTER defaults to a prompt.  Return the filing summary plist."
  (let* ((deadlines (litgraph-get response 'result 'deadlines))
         (matter (or matter (litgraph-org--read-matter request)))
         (file (litgraph-org--destination)))
    (unless deadlines
      (user-error "No deadlines in this result; try a different trigger node"))
    (make-directory (file-name-directory file) t)
    (let* ((buffer (find-file-noselect file))
           (summary
            (with-current-buffer buffer
              (unless (derived-mode-p 'org-mode) (org-mode))
              (when (zerop (buffer-size))
                (insert "#+title: Litigation deadlines (litgraph)\n\n"))
              (prog1 (litgraph-org-file-deadlines deadlines matter)
                (run-hooks 'litgraph-org-after-import-hook)
                (when litgraph-org-save (save-buffer))))))
      (litgraph-org--offer-agenda file)
      (message "litgraph: %d added, %d updated in %s under \"%s\""
               (plist-get summary :added) (plist-get summary :updated)
               (abbreviate-file-name file) matter)
      (when litgraph-org-jump
        (pop-to-buffer buffer)
        (goto-char (plist-get summary :position))
        (org-fold-show-subtree)
        (recenter 0))
      summary)))

;;;###autoload
(defun litgraph-deadlines-to-org (&optional all)
  "Compute deadlines from a triggering event and file them into Org.
Asks for the trigger date and the node whose outgoing deadlines to
compute, then for the matter name; each deadline becomes a heading with
a DEADLINE timestamp in `litgraph-org-file'.  With prefix argument ALL,
ask for every option of the `deadlines' op (service method, holidays,
downstream deadlines, clerk inaccessibility)."
  (interactive "P")
  (litgraph--with-metadata
   "deadlines"
   (lambda (schema describe graph)
     (let* ((op (if all
                    (litgraph-read-op-arguments "deadlines" schema describe
                                                graph t)
                  (litgraph-obj
                   "op" "deadlines"
                   "trigger" (org-read-date nil nil nil "Trigger date: ")
                   "node" (litgraph-read "Trigger event (node)"
                                         (litgraph-node-candidates graph)
                                         'litgraph-node nil t))))
            (request (litgraph-build-request op)))
       (litgraph-request
        request
        (lambda (response) (litgraph-org-import response request)))))))

(provide 'litgraph-org)

;;; litgraph-org.el ends here
