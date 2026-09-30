;;; lint.el --- checkdoc and package-lint driver for litgraph-emacs -*- lexical-binding: t; -*-

;; Run through the Makefile:  make lint

;;; Code:

(require 'checkdoc)

(defun litgraph-lint-checkdoc ()
  "Run checkdoc over the files on the command line; exit 1 on any finding."
  (let ((failed nil))
    (dolist (file command-line-args-left)
      (when (get-buffer "*Warnings*") (kill-buffer "*Warnings*"))
      (checkdoc-file file)
      (when-let* ((buffer (get-buffer "*Warnings*")))
        (setq failed t)
        (princ (format "checkdoc: %s\n%s\n" file
                       (with-current-buffer buffer (buffer-string))))))
    (setq command-line-args-left nil)
    (if failed
        (kill-emacs 1)
      (princ "checkdoc: clean\n"))))

(defun litgraph-lint--installed (dir)
  "The package-lint directory inside the ELPA directory DIR, or nil."
  (when (and dir (not (string-empty-p dir)) (file-directory-p dir))
    (car (directory-files dir t "\\`package-lint-[0-9]" t))))

(defun litgraph-lint--find-package-lint (dir)
  "Return a directory holding package-lint: DIR, .deps/, ~/.emacs.d, or MELPA."
  (or (litgraph-lint--installed dir)
      (litgraph-lint--installed (expand-file-name ".deps/elpa"))
      (litgraph-lint--installed (locate-user-emacs-file "elpa"))
      (progn
        (require 'package)
        (setq package-user-dir (expand-file-name ".deps/elpa"))
        (add-to-list 'package-archives
                     '("melpa" . "https://melpa.org/packages/") t)
        (package-initialize)
        (package-refresh-contents)
        (package-install 'package-lint)
        (litgraph-lint--installed package-user-dir))))

(defun litgraph-lint-package-lint (&optional dir)
  "Run package-lint over the files on the command line; exit 1 on findings.
DIR may name an ELPA directory that already contains package-lint."
  (let ((lint-dir (litgraph-lint--find-package-lint dir)))
    (add-to-list 'load-path lint-dir)
    (require 'package)
    (package-initialize)
    (require 'package-lint)
    (setq package-lint-main-file "litgraph.el")
    (let ((failed nil))
      (dolist (file command-line-args-left)
        (with-current-buffer (find-file-noselect file)
          (emacs-lisp-mode)
          (dolist (issue (package-lint-buffer))
            (setq failed t)
            (princ (format "%s:%d:%d: %s: %s\n" file (nth 0 issue) (nth 1 issue)
                           (nth 2 issue) (nth 3 issue))))))
      (setq command-line-args-left nil)
      (if failed
          (kill-emacs 1)
        (princ "package-lint: clean\n")))))

;;; lint.el ends here
