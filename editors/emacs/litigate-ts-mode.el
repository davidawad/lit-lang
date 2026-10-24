;;; litigate-ts-mode.el --- Major mode for litigate (.lit/.case) using tree-sitter  -*- lexical-binding: t; -*-

;; Copyright (C) 2026 David Awad

;; Author: David Awad
;; URL: https://github.com/davidawad/litgraph
;; Version: 0.3.0
;; Package-Requires: ((emacs "29.1"))
;; Keywords: languages, tree-sitter, litigate
;; SPDX-License-Identifier: GPL-3.0-or-later

;; This file is not part of GNU Emacs.

;;; Commentary:

;; A major mode for litigate rule modules (`.lit') and matters (`.case'),
;; built on Emacs's built-in tree-sitter support (`treesit', Emacs 29+) and
;; the grammar in `editors/tree-sitter-litigate'.  It provides:
;;
;; - font-lock mirroring `queries/highlights.scm' (four levels),
;; - indentation mirroring `queries/indents.scm' (two spaces, SPEC §10),
;; - imenu and defun navigation from the declarations `queries/locals.scm'
;;   defines,
;; - outline (Emacs 30+) over top-level declarations,
;; - an `eglot' entry for `litigate lsp --stdio',
;; - a `treesit-language-source-alist' entry and `auto-mode-alist' entries
;;   for `.lit' and `.case'.
;;
;; Install the grammar with `M-x treesit-install-language-grammar RET
;; litigate', or with `cargo run -p litigate-editors --bin litigate-grammar' when your Emacs is
;; linked against libtree-sitter older than 0.25 (it needs an ABI 14
;; parser).  See `editors/README.md'.

;;; Code:

(require 'treesit)
(require 'subr-x)
(require 'seq)
(eval-when-compile (require 'rx))

(declare-function treesit-parser-create "treesit.c")
(declare-function treesit-node-type "treesit.c")
(declare-function treesit-node-text "treesit.c")
(declare-function treesit-node-child-by-field-name "treesit.c")
(declare-function treesit-node-parent "treesit.c")

(defgroup litigate nil
  "Major mode for the litigate legal-procedure language."
  :group 'languages
  :prefix "litigate-ts-mode-")

(defcustom litigate-ts-mode-indent-offset 2
  "Number of spaces per indentation level in `litigate-ts-mode'.
The canonical format (`litigate fmt', SPEC §10) uses two."
  :type 'integer
  :safe #'integerp)

(defcustom litigate-ts-mode-lsp-command '("litigate" "lsp" "--stdio")
  "Command that starts the litigate language server, for `eglot'."
  :type '(repeat string))

;;;###autoload
(defconst litigate-ts-mode-grammar-recipe
  '(litigate "https://github.com/davidawad/litgraph" nil
             "editors/tree-sitter-litigate/src")
  "`treesit-language-source-alist' entry for the litigate grammar.")

;;;###autoload
(with-eval-after-load 'treesit
  (add-to-list 'treesit-language-source-alist litigate-ts-mode-grammar-recipe))

;;; Keywords (kept equal to queries/highlights.scm by the ERT suite).

(defconst litigate-ts-mode--declaration-keywords
  '("edition" "module" "import" "package" "start" "group" "type" "enum" "event"
    "state" "decision" "chance" "terminal" "link" "instance" "rule"
    "deadline" "counting" "calendar" "transition" "authority" "default"
    "exception" "test"
    "case" "uses" "as_of" "known_at" "summary" "notes" "source" "perspective"
    "parties" "events" "facts" "stakes" "beliefs" "prior" "observe" "analysis")
  "Keywords that open a declaration.")

(defconst litigate-ts-mode--clause-keywords
  '("forum" "by" "docket" "outcome" "info" "p" "effort" "cost" "within" "duration" "min" "mode"
    "max" "sets" "clears" "requires" "forbids" "replaces" "tags" "attrs" "note"
    "valence" "court_listener_url" "action_id" "extendable" "extension_authority"
    "payoff" "payoff_by_flag" "pack" "remove_edges" "probabilities" "roles"
    "payoff_transform" "stake" "patch" "duty" "power" "liability" "from" "after" "before"
    "counted_by" "exclude_trigger_day" "include_last_day" "roll_forward_on"
    "service_extension" "holidays" "anchor" "short_period" "below" "count"
    "allow_hours" "end_of_month" "effective" "observed" "amends" "subject_to" "notwithstanding"
    "supersedes" "revives" "override" "delegable_to" "rebuts" "undercuts"
    "priority" "url" "bearer" "designated" "interpretation" "trigger" "extend" "waive" "after_expiry" "never"
    "removes" "elects" "extends" "waives" "to" "where" "and" "or" "forums"
    "forward_holidays" "declared_holidays"
    "given" "expect" "on")
  "Clause, field and test keywords.")

(defconst litigate-ts-mode--conditional-keywords
  '("when" "unless" "otherwise" "match")
  "Conditional keywords.")

(defconst litigate-ts-mode--operators
  '("->" "=>" "==" "!=" "<=" ">=" "&&" "||" "+" "-" "*" "/" "%" "!" "=")
  "Operator tokens.")

(defconst litigate-ts-mode--builtins
  '("deadline" "has_event" "prob" "given" "in" "intersect" "disjoint" "len"
    "contains" "credal" "lower" "upper" "mean" "at" "fixed" "nth" "last" "every" "once")
  "Builtin functions and holiday rules.")

;;; Font-lock.

(defvar litigate-ts-mode--font-lock-settings
  (treesit-font-lock-rules
   :language 'litigate
   :feature 'comment
   '((doc_comment) @font-lock-doc-face
     [(line_comment) (block_comment)] @font-lock-comment-face)

   :language 'litigate
   :feature 'definition
   '((module_decl path: (module_path (identifier) @font-lock-constant-face))
     (import_decl path: (module_path (identifier) @font-lock-constant-face))
     (import_list (identifier) @font-lock-variable-name-face)
     (package_decl name: (identifier) @font-lock-constant-face)
     (package_decl forum: (identifier) @font-lock-constant-face)
     (package_decl authority: (identifier) @font-lock-type-face)
     (use_item package: (identifier) @font-lock-constant-face)
     (case_file name: (identifier) @font-lock-function-name-face)
     (type_decl name: (identifier) @font-lock-type-face)
     (enum_decl name: (identifier) @font-lock-type-face)
     (variant_list (identifier) @font-lock-constant-face)
     (event_decl name: (identifier) @font-lock-type-face)
     (param name: (identifier) @font-lock-variable-name-face)
     (authority_decl level: (identifier) @font-lock-type-face)
     (node_decl name: (identifier) @font-lock-function-name-face)
     (group_decl name: (identifier) @font-lock-function-name-face)
     (edge_decl name: (identifier) @font-lock-property-name-face)
     (link_decl name: (identifier) @font-lock-property-name-face)
     (holiday_decl name: (identifier) @font-lock-property-name-face)
     (party_decl name: (identifier) @font-lock-variable-name-face)
     ([(rule_decl) (deadline_decl) (counting_decl) (calendar_decl)
       (transition_decl) (default_decl) (exception_decl)]
      name: (identifier) @font-lock-function-name-face)
     (instance_decl base: (identifier) @font-lock-constant-face)
     (instance_decl origin: (identifier) @font-lock-constant-face)
     (test_decl name: (string) @font-lock-doc-face))

   :language 'litigate
   :feature 'string
   '((escape_sequence) @font-lock-escape-face
     (string) @font-lock-string-face
     (version_req) @font-lock-constant-face)

   :language 'litigate
   :feature 'keyword
   `([,@litigate-ts-mode--conditional-keywords] @font-lock-keyword-face
     "cite" @font-lock-preprocessor-face
     (discretion) @font-lock-keyword-face
     [,@litigate-ts-mode--declaration-keywords] @font-lock-keyword-face
     [,@litigate-ts-mode--clause-keywords] @font-lock-keyword-face)

   :language 'litigate
   :feature 'type
   '((type name: (identifier) @font-lock-type-face)
     (by_clause actor: (ref (identifier) @font-lock-type-face))
     (party_decl actor: (identifier) @font-lock-type-face)
     (party_decl entity: (identifier) @font-lock-type-face))

   :language 'litigate
   :feature 'number
   '([(number) (money) (duration) (date)] @font-lock-number-face)

   :language 'litigate
   :feature 'constant
   '([(boolean) (unit) (unvalued) (hole) (open_end)]
     @font-lock-constant-face
     [(anonymous) (wildcard)] @font-lock-builtin-face)

   :language 'litigate
   :feature 'property
   '([(num_entry key: (identifier) @font-lock-property-name-face)
      (money_entry key: (identifier) @font-lock-property-name-face)
      (record_field key: (identifier) @font-lock-property-name-face)
      (probability_entry key: (identifier) @font-lock-property-name-face)
      (role_entry key: (identifier) @font-lock-property-name-face)
      (service_entry method: (identifier) @font-lock-constant-face)]
     (field_expression field: (identifier) @font-lock-property-use-face))

   :language 'litigate
   :feature 'builtin
   `(((call_expression
       function: (ref :anchor (identifier) @font-lock-builtin-face :anchor))
      (:match ,(rx-to-string `(seq bos (or ,@litigate-ts-mode--builtins) eos))
              @font-lock-builtin-face)))

   :language 'litigate
   :feature 'function
   '((call_expression
      function: (ref (identifier) @font-lock-function-call-face :anchor)))

   :language 'litigate
   :feature 'operator
   `([,@litigate-ts-mode--operators] @font-lock-operator-face)

   :language 'litigate
   :feature 'bracket
   '(["(" ")" "[" "]" "{" "}"] @font-lock-bracket-face)

   :language 'litigate
   :feature 'delimiter
   '(["," ":" "::" "." "@" "|"] @font-lock-delimiter-face)

   :language 'litigate
   :feature 'variable
   '((identifier) @font-lock-variable-use-face)

   :language 'litigate
   :feature 'error
   :override t
   '((ERROR) @font-lock-warning-face))
  "Tree-sitter font-lock settings for `litigate-ts-mode'.")

(defconst litigate-ts-mode--font-lock-feature-list
  '((comment definition)
    (keyword string type)
    (number constant property builtin function)
    (operator bracket delimiter variable error))
  "Font-lock features by level, for `treesit-font-lock-feature-list'.")

;;; Indentation (kept equal to queries/indents.scm by the ERT suite).

(defconst litigate-ts-mode--indent-body-types
  '("package_decl" "variant_list" "param_list" "node_body" "edge_fields" "patch_fields"
    "duration_range" "deadline_opts" "deadline_decl_opts" "capability"
    "instance_decl" "probability_map"
    "role_map" "rule_body" "counting_body" "service_map" "short_period"
    "calendar_body" "transition_body" "test_body" "source_body"
    "parties_block" "events_block" "facts_block" "stakes_block"
    "beliefs_block" "count_map" "num_record" "money_record"
    "id_list" "import_list" "record_expression"
    "list_expression" "arguments" "match_expression")
  "Node types whose contents indent one level (`@indent.begin').")

(defconst litigate-ts-mode--delimited-types
  (seq-difference litigate-ts-mode--indent-body-types
                  '("package_decl" "instance_decl" "match_expression"))
  "Indenting node types that open with a delimiter, not a keyword.
On a line of their own they stay at their parent's indentation.")

(defvar litigate-ts-mode--indent-rules
  `((litigate
     ((parent-is ,(rx bos (or "source_file" "lit_file" "case_file") eos))
      column-0 0)
     ((node-is ,(rx bos (or "}" "]" ")" "{") eos)) parent-bol 0)
     ((parent-is ,(rx bos "block_comment" eos)) prev-adaptive-prefix 0)
     ((parent-is ,(rx bos (or "string" "string_content") eos)) no-indent 0)
     ((node-is ,(rx-to-string
                 `(seq bos (or ,@litigate-ts-mode--delimited-types) eos)))
      parent-bol 0)
     ((parent-is ,(rx-to-string
                   `(seq bos (or ,@litigate-ts-mode--indent-body-types) eos)))
      parent-bol litigate-ts-mode-indent-offset)
     (no-node parent-bol 0)
     ;; A comment between a header and its body stays with the header.
     ((node-is ,(rx bos (or "line_comment" "doc_comment" "block_comment") eos))
      parent-bol 0)
     ;; Anything else on a line of its own continues the construct above
     ;; it (a clause after a header line, an operand after an operator).
     (catch-all parent-bol litigate-ts-mode-indent-offset)))
  "Tree-sitter indent rules for `litigate-ts-mode'.")

;;; Navigation and imenu.

(defconst litigate-ts-mode--defun-types
  '("module_decl" "package_decl" "group_decl" "type_decl" "enum_decl"
    "event_decl" "authority_decl" "node_decl" "link_decl" "instance_decl"
    "rule_decl" "counting_decl" "calendar_decl" "transition_decl"
    "test_decl" "case_file" "source_decl" "parties_block" "events_block"
    "facts_block" "stakes_block" "beliefs_block" "analysis_block")
  "Top-level declaration node types (defuns).")

(defun litigate-ts-mode--defun-name (node)
  "Return the declared name of NODE, or nil."
  (pcase (treesit-node-type node)
    ((or "module_decl" "import_decl")
     (when-let* ((path (treesit-node-child-by-field-name node "path")))
       (treesit-node-text path t)))
    ("test_decl"
     (when-let* ((name (treesit-node-child-by-field-name node "name")))
       (string-trim (treesit-node-text name t) "\"" "\"")))
    (_
     (when-let* ((name (treesit-node-child-by-field-name node "name")))
       (treesit-node-text name t)))))

(defun litigate-ts-mode--top-level-p (node)
  "Return non-nil when NODE is a direct item of a file."
  (when-let* ((parent (treesit-node-parent node)))
    (member (treesit-node-type parent) '("lit_file" "case_file"))))

(defun litigate-ts-mode--imenu-entry (type &optional pred)
  "Return a `treesit-simple-imenu-settings' regexp for node TYPE.
PRED, when given, further filters matching nodes."
  (list (rx-to-string `(seq bos (or ,@(ensure-list type)) eos)) pred))

(defconst litigate-ts-mode--imenu-categories
  '(("Module" "module_decl") ("Case" "case_file")
    ("Node" "node_decl") ("Edge" "edge_decl" "link_decl")
    ("Group" "group_decl")
    ("Type" "type_decl" "enum_decl") ("Event" "event_decl")
    ("Rule" "rule_decl") ("Deadline" "deadline_decl")
    ("Counting" "counting_decl") ("Calendar" "calendar_decl")
    ("Holiday" "holiday_decl")
    ("Transition" "transition_decl")
    ("Default" "default_decl") ("Exception" "exception_decl")
    ("Test" "test_decl") ("Party" "party_decl"))
  "Imenu categories and the node types listed under each.")

(defun litigate-ts-mode--imenu-settings ()
  "Build `treesit-simple-imenu-settings' for litigate."
  (mapcar (lambda (category)
            (cons (car category)
                  (litigate-ts-mode--imenu-entry
                   (cdr category)
                   (lambda (node) (litigate-ts-mode--defun-name node)))))
          litigate-ts-mode--imenu-categories))

;;; Syntax table.

(defvar litigate-ts-mode--syntax-table
  (let ((table (make-syntax-table)))
    (modify-syntax-entry ?/ ". 124b" table)
    (modify-syntax-entry ?* ". 23" table)
    (modify-syntax-entry ?\n "> b" table)
    (modify-syntax-entry ?\" "\"" table)
    (modify-syntax-entry ?\\ "\\" table)
    (modify-syntax-entry ?- "_" table)
    (modify-syntax-entry ?_ "_" table)
    (modify-syntax-entry ?' "." table)
    (dolist (c '(?+ ?% ?< ?> ?= ?! ?& ?| ?^ ?@ ?? ?:))
      (modify-syntax-entry c "." table))
    table)
  "Syntax table for `litigate-ts-mode'.")

;;; Mode.

;;;###autoload
(define-derived-mode litigate-ts-mode prog-mode "litigate"
  "Major mode for litigate rule modules (.lit) and matters (.case).

Uses the tree-sitter grammar `litigate'; install it with
\\[treesit-install-language-grammar] (see `editors/README.md').

\\{litigate-ts-mode-map}"
  :group 'litigate
  :syntax-table litigate-ts-mode--syntax-table
  (setq-local comment-start "// ")
  (setq-local comment-end "")
  (setq-local comment-start-skip (rx (or (seq "/" (+ "/")) (seq "/" (+ "*"))) (* space)))
  (setq-local comment-end-skip (rx (* space) (group (or (syntax comment-end) (seq (+ "*") "/")))))
  (setq-local indent-tabs-mode nil)
  (setq-local tab-width litigate-ts-mode-indent-offset)
  (setq-local electric-indent-chars (append "{}[]()" electric-indent-chars))
  (when (treesit-ready-p 'litigate)
    (treesit-parser-create 'litigate)
    (setq-local treesit-font-lock-settings litigate-ts-mode--font-lock-settings)
    (setq-local treesit-font-lock-feature-list litigate-ts-mode--font-lock-feature-list)
    (setq-local treesit-simple-indent-rules litigate-ts-mode--indent-rules)
    (setq-local treesit-defun-type-regexp
                (cons (rx-to-string `(seq bos (or ,@litigate-ts-mode--defun-types) eos))
                      #'litigate-ts-mode--top-level-p))
    (setq-local treesit-defun-name-function #'litigate-ts-mode--defun-name)
    (setq-local treesit-simple-imenu-settings (litigate-ts-mode--imenu-settings))
    (when (boundp 'treesit-outline-predicate)
      (setq-local treesit-outline-predicate
                  (cons (rx-to-string `(seq bos (or ,@litigate-ts-mode--defun-types) eos))
                        #'litigate-ts-mode--top-level-p)))
    (treesit-major-mode-setup)))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.lit\\'" . litigate-ts-mode))
;;;###autoload
(add-to-list 'auto-mode-alist '("\\.case\\'" . litigate-ts-mode))

;;;###autoload
(with-eval-after-load 'eglot
  (defvar eglot-server-programs)
  (add-to-list 'eglot-server-programs
               `(litigate-ts-mode . ,(lambda (&rest _) litigate-ts-mode-lsp-command))))

(provide 'litigate-ts-mode)
;;; litigate-ts-mode.el ends here
