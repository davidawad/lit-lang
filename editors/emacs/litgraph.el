;;; litgraph.el --- Litigation procedure graphs and deadlines via litgraph -*- lexical-binding: t; -*-

;; Copyright (C) 2026 David Awad

;; Author: David Awad <me@davidaw.ad>
;; Maintainer: David Awad <me@davidaw.ad>
;; Version: 0.1.0
;; Package-Requires: ((emacs "29.1") (transient "0.4.0"))
;; Keywords: tools, convenience, law
;; URL: https://github.com/davidawad/litgraph
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

;; litgraph models litigation procedure as a stochastic game on a graph.
;; The `litigate' command line compiles a matter (a `.case' file) or a
;; package (a `.lit' file or package directory) and runs one engine op
;; on it: `litigate run TARGET --op OP --set NAME=VALUE', printing one
;; JSON envelope.  This package brings that into Emacs:
;;
;;  - `litgraph-menu' (a transient) is the front door.  Pick a matter or
;;    package, then run any op.  Nothing ever blocks Emacs: every call is
;;    an asynchronous process.
;;  - `litgraph-run-op' runs any op the installed binary knows about
;;    (ops, node ids and edge ids complete from the binary's own
;;    `describe' and `graph' output for the target, so new ops appear by
;;    themselves).
;;  - Results open in readable buffers: summary first, tables, warnings
;;    grouped, provenance folded, raw JSON one key away (litgraph-view.el).
;;  - `litgraph-deadlines-to-org' turns computed deadlines into Org headings
;;    with DEADLINE timestamps so they show up in the agenda
;;    (litgraph-org.el).
;;  - `litgraph-graph' draws the procedure graph with Graphviz
;;    (litgraph-graph.el).
;;  - `ob-litgraph' runs ops on a matter from Org source blocks.
;;  - `litgraph-timeline' shows a .case or .lit file's timeline
;;    (`litigate timeline'; litgraph-timeline.el).
;;  - `litigate-ts-mode' edits .lit and .case files (litigate-ts-mode.el).
;;
;; This file holds the shared core: customization, the process layer, request
;; construction, and the cache of engine metadata used for completion.

;;; Code:

(require 'cl-lib)
(require 'seq)
(require 'subr-x)
(require 'json)

(declare-function litgraph-view-show "litgraph-view")
(declare-function litgraph-menu "litgraph-menu" ())
(declare-function litgraph-graph "litgraph-graph")
(declare-function litgraph-deadlines-to-org "litgraph-org")
(declare-function litgraph-timeline "litgraph-timeline" (file))
(declare-function org-read-date "org")


;;;; Customization

(defgroup litgraph nil
  "Litigation procedure graphs."
  :group 'tools
  :prefix "litgraph-"
  :link '(url-link "https://github.com/davidawad/litgraph"))

(defcustom litgraph-executable "litigate"
  "Name or absolute path of the `litigate' binary."
  :type 'string)

(defcustom litgraph-project-directory nil
  "Directory the engine runs in: the one holding `lit/' and `cases/'.
`litigate run' finds the packages a matter `uses' in `lit/' under the
directory it runs in, and bare matter names resolve to `cases/NAME.case'
there.  Nil looks for the nearest directory holding `lit/', starting
from `default-directory'."
  :type '(choice (const :tag "Nearest with lit/" nil) directory))

(defcustom litgraph-fallback-directories
  '("/opt/homebrew/bin" "/usr/local/bin" "~/.cargo/bin" "~/.nix-profile/bin")
  "Directories searched when `litgraph-executable' is not found.
They are tried after the variable `exec-path'.  GUI Emacs on macOS
often lacks the Homebrew prefix on its path."
  :type '(repeat directory))

(defcustom litgraph-default-target nil
  "Target used when none has been selected with `litgraph-set-target'.
A `.case' matter, a `.lit' file or a package directory, or the bare
name of a matter in the project's `cases/' (or a package in `lit/')."
  :type '(choice (const :tag "None" nil) string))

(defcustom litgraph-timeout 120
  "Seconds before a running litgraph process is killed."
  :type 'natnum)

(defcustom litgraph-display-function #'litgraph-view-show
  "Function called with (RESPONSE REQUEST RAW) to display a finished response.
RAW is the response's JSON text."
  :type 'function)


;;;; Session state

(defvar litgraph-target nil
  "Target for this session: a matter or package path or name, or nil.")

(defvar litgraph-params nil
  "Alist of (NAME . NUMBER) scenario parameter overrides for this session.")

(defvar litgraph-last-response nil
  "The most recent response (parsed) received from litgraph.")

(defvar litgraph-last-error nil
  "Message of the most recent failed litgraph call.")

(defvar litgraph--last-raw nil
  "Raw JSON text of the most recent successful response.")

(defun litgraph--target ()
  "The target in effect, or nil."
  (or litgraph-target litgraph-default-target))

(defun litgraph-target-label (target)
  "A short name for TARGET: a matter's name, or a package's directory."
  (cond ((not (stringp target)) nil)
        ((string-suffix-p ".case" target)
         (file-name-base target))
        (t (file-name-nondirectory (directory-file-name target)))))

(defun litgraph-context-label ()
  "A one-line description of the current target and params."
  (string-join
   (delq nil
         (list (or (litgraph-target-label (litgraph--target)) "no target")
               (and litgraph-params
                    (mapconcat (lambda (p) (format "%s=%s" (car p) (cdr p)))
                               litgraph-params " "))))
   "  "))


;;;; JSON

(defun litgraph-parse (string)
  "Parse JSON STRING into alists (symbol keys) and lists.
JSON null and false become nil; empty arrays and objects become nil."
  (json-parse-string string :object-type 'alist :array-type 'list
                     :null-object nil :false-object nil))

(defun litgraph-parse-strict (string)
  "Parse JSON STRING into hash tables and vectors, keeping every type."
  (json-parse-string string :object-type 'hash-table :array-type 'array
                     :null-object :null :false-object :false))

(defun litgraph-serialize (object)
  "Serialize OBJECT (hash tables, vectors, strings, numbers) to JSON."
  (json-serialize object :null-object :null :false-object :false))

(defun litgraph-obj (&rest keys-and-values)
  "Build a JSON object (hash table) from KEYS-AND-VALUES, dropping nil values.
Keys are strings."
  (let ((table (make-hash-table :test #'equal)))
    (while keys-and-values
      (let ((key (pop keys-and-values))
            (value (pop keys-and-values)))
        (when value (puthash key value table))))
    table))

(defun litgraph-vec (list)
  "Return LIST as a JSON array."
  (apply #'vector list))

(defun litgraph-get (object &rest path)
  "Look up PATH (symbols, strings or list indices) in the parsed OBJECT."
  (dolist (key path object)
    (setq object
          (cond ((integerp key) (nth key object))
                ((stringp key) (alist-get (intern key) object))
                (t (alist-get key object))))))

(defun litgraph-hash-copy (table)
  "Deep-copy the parsed-strict JSON TABLE."
  (litgraph-parse-strict (litgraph-serialize table)))


;;;; Requests

;; A request is a hash table {"target": T, "op": OP, "params": {NAME: N}}:
;; T a matter or package (see `litgraph-resolve-target'), OP an op object
;; and the params scenario overrides.  `litgraph-request-args' turns it
;; into a `litigate run' command line.

(defun litgraph-project-root ()
  "Return the directory to run the engine in, or nil if there is none.
That is `litgraph-project-directory', else the nearest directory
holding `lit/' at or above `default-directory'."
  (if litgraph-project-directory
      (file-name-as-directory (expand-file-name litgraph-project-directory))
    (when (and (file-directory-p default-directory)
               (not (file-remote-p default-directory)))
      (locate-dominating-file
       default-directory
       (lambda (dir) (file-directory-p (expand-file-name "lit" dir)))))))

(defun litgraph-resolve-target (target)
  "The absolute path of TARGET, or TARGET itself when nothing matches.
TARGET is a path (relative to `default-directory' or the project root),
or a bare name: `cases/NAME.case', then `lit/NAME', in the project."
  (let ((root (litgraph-project-root)))
    (or (seq-find #'file-exists-p
                  (delq nil
                        (list (expand-file-name target)
                              (and root (expand-file-name target root))
                              (and root (expand-file-name
                                         (format "cases/%s.case" target) root))
                              (and root (expand-file-name
                                         (format "lit/%s" target) root)))))
        target)))

(defun litgraph-target-candidates ()
  "The project's matters and packages, as (PATH . KIND) pairs.
PATH is relative to the project root: every `.case' under `cases/' and
every package directory in `lit/'."
  (when-let* ((root (litgraph-project-root)))
    (append
     (let ((cases (expand-file-name "cases" root)))
       (when (file-directory-p cases)
         (mapcar (lambda (file) (cons (file-relative-name file root) "matter"))
                 (sort (directory-files-recursively cases "\\.case\\'") #'string<))))
     (let ((lit (expand-file-name "lit" root)))
       (when (file-directory-p lit)
         (mapcar (lambda (dir) (cons (file-relative-name dir root) "package"))
                 (seq-filter #'file-directory-p
                             (directory-files lit t "\\`[^.]"))))))))

(defun litgraph--op-object (op)
  "OP as an op object: OP itself, or {\"op\": OP} for a name."
  (if (stringp op) (litgraph-obj "op" op) op))

(defun litgraph--params-object (params)
  "PARAMS (alist of (NAME . NUMBER)) as a JSON object, or nil when empty."
  (when params
    (let ((table (make-hash-table :test #'equal)))
      (dolist (p (reverse params)) (puthash (format "%s" (car p)) (cdr p) table))
      table)))

(defun litgraph-build-request (op &optional target params)
  "Build a request hash table for OP.
OP is an op name string, or a hash table with an \"op\" key.  TARGET
defaults to the session target and PARAMS (an alist of (NAME . NUMBER))
to the session parameters."
  (litgraph-obj "target" (or target (litgraph--target))
                "op" (litgraph--op-object op)
                "params" (litgraph--params-object (or params litgraph-params))))

(defun litgraph--format-param (value)
  "VALUE (a number) as `--set' writes it."
  (if (and (floatp value) (= value (ffloor value)) (< (abs value) 1e15))
      (format "%d" (truncate value))
    (format "%s" value)))

(defun litgraph-request-args (request)
  "The `litigate' arguments that run REQUEST (a hash table).
Signal a `user-error' when REQUEST names no target."
  (let ((target (gethash "target" request))
        (params (gethash "params" request))
        (sets nil))
    (unless target
      (user-error "No matter selected: choose a .case file or package with `litgraph-set-target'"))
    (when (hash-table-p params)
      (maphash (lambda (name value)
                 (push (format "%s=%s" name (litgraph--format-param value)) sets))
               params))
    (append (list "run" (litgraph-resolve-target target)
                  "--op" (litgraph-serialize (gethash "op" request)))
            (mapcan (lambda (set) (list "--set" set)) (sort sets #'string<))
            (list "--format" "json"))))


;;;; Process layer

(defvar litgraph-process-function #'litgraph--spawn
  "Function (ARGS INPUT CALLBACK) that runs the binary asynchronously.
ARGS are the `litigate' arguments.  INPUT is a string or nil.
CALLBACK is called with (EXIT STDOUT STDERR) when the process ends,
where EXIT is an exit status, or the symbol `timeout' or `signal'.
Tests replace this to stub out the binary.")

(defvar litgraph--processes nil
  "Live litgraph processes.")

(defun litgraph-executable-path ()
  "Absolute path to the `litigate' binary, or signal a `user-error'."
  (or (and (file-name-absolute-p litgraph-executable)
           (file-executable-p litgraph-executable)
           litgraph-executable)
      (executable-find litgraph-executable)
      (let ((exec-path (append exec-path
                               (mapcar #'expand-file-name
                                       litgraph-fallback-directories))))
        (executable-find litgraph-executable))
      (user-error
       "Cannot find `%s'; install litigate or set `litgraph-executable'"
       litgraph-executable)))

(defun litgraph--spawn (args input callback)
  "Run the binary with ARGS asynchronously, feeding INPUT on stdin.
Call CALLBACK with (EXIT STDOUT STDERR) when done.  Return the process."
  (let* ((out (generate-new-buffer " *litgraph-out*" t))
         (err (generate-new-buffer " *litgraph-err*" t))
         (default-directory (or (litgraph-project-root)
                                (if (and (file-directory-p default-directory)
                                         (not (file-remote-p default-directory)))
                                    default-directory
                                  temporary-file-directory)))
         (done nil)
         (timer nil)
         (proc
          (make-process
           :name "litgraph"
           :buffer out
           :stderr err
           :command (cons (litgraph-executable-path) args)
           :coding 'utf-8-unix
           :connection-type 'pipe
           :noquery t
           :sentinel
           (lambda (process _event)
             (unless (or done (process-live-p process))
               (setq done t)
               (when timer (cancel-timer timer))
               (setq litgraph--processes (delq process litgraph--processes))
               (let ((stdout (with-current-buffer out (buffer-string)))
                     (stderr (with-current-buffer err (buffer-string)))
                     (status (process-status process))
                     (code (process-exit-status process)))
                 (kill-buffer out)
                 (kill-buffer err)
                 (funcall callback
                          (cond ((eq (process-get process 'litgraph-end)
                                     'timeout)
                                 'timeout)
                                ((eq status 'signal) 'signal)
                                (t code))
                          stdout stderr)))))))
    (set-process-query-on-exit-flag (get-buffer-process err) nil)
    (push proc litgraph--processes)
    (when (and litgraph-timeout (> litgraph-timeout 0))
      (setq timer
            (run-at-time litgraph-timeout nil
                         (lambda ()
                           (when (process-live-p proc)
                             (process-put proc 'litgraph-end 'timeout)
                             (delete-process proc))))))
    (when input (process-send-string proc input))
    (process-send-eof proc)
    proc))

(defun litgraph-cancel ()
  "Kill every running litgraph process."
  (interactive)
  (mapc #'delete-process (copy-sequence litgraph--processes))
  (setq litgraph--processes nil)
  (message "litgraph: cancelled"))

(defun litgraph--describe-failure (exit stdout stderr)
  "Explain a failed process run given EXIT, STDOUT and STDERR."
  (pcase exit
    ('timeout (format "timed out after %ss" litgraph-timeout))
    ('signal "process was killed")
    (_ (let ((text (string-trim (if (string-empty-p (string-trim stderr))
                                    stdout
                                  stderr))))
         (format "exit %s: %s" exit (truncate-string-to-width text 300))))))

(defun litgraph--diagnostic-message (envelope)
  "The first error diagnostic of the `litigate' ENVELOPE, as a message."
  (when-let* ((diagnostic
               (or (seq-find (lambda (d) (equal (litgraph-get d 'severity) "error"))
                             (litgraph-get envelope 'diagnostics))
                   (car (litgraph-get envelope 'diagnostics)))))
    (let ((span (litgraph-get diagnostic 'span)))
      (format "%s: %s%s" (or (litgraph-get diagnostic 'code) "error")
              (or (litgraph-get diagnostic 'message) "")
              (if span
                  (format " (%s:%s)" (litgraph-get span 'file)
                          (litgraph-get span 'line))
                "")))))

(defun litgraph--engine-response (stdout)
  "The engine response inside the `litigate run' envelope STDOUT, or nil.
Return (RESPONSE . RAW), RAW being the response's own JSON text."
  (when-let* ((strict (ignore-errors (litgraph-parse-strict stdout)))
              ((hash-table-p strict))
              (inner (gethash "result" strict))
              ((hash-table-p inner))
              ((gethash "ok" inner)))
    (let ((raw (litgraph-serialize inner)))
      (cons (litgraph-parse raw) raw))))

(defun litgraph--decode (exit stdout stderr)
  "Turn a finished run (EXIT, STDOUT, STDERR) into (OK . PAYLOAD).
PAYLOAD is (RESPONSE . RAW) when OK: the engine response `litigate run'
wraps, parsed and as JSON text.  Else it is (MESSAGE . RESPONSE): an
error message, with the engine response for an engine error (`ok':
false), or nil when the run failed before the engine (a compile
error, a usage error or a dead process)."
  (let* ((envelope (and (not (memq exit '(timeout signal)))
                        (not (string-empty-p (string-trim stdout)))
                        (ignore-errors (litgraph-parse stdout))))
         (engine (and (consp envelope) (litgraph--engine-response stdout)))
         (response (car engine)))
    (cond
     ((and response (litgraph-get response 'ok))
      (cons t engine))
     (response
      (let ((error-object (litgraph-get response 'error)))
        (cons nil (cons (format "%s: %s%s"
                                (or (litgraph-get error-object 'code) "error")
                                (or (litgraph-get error-object 'message) "")
                                (if-let* ((hint (litgraph-get error-object 'hint)))
                                    (format " (%s)" hint)
                                  ""))
                        response))))
     ((and (consp envelope) (litgraph--diagnostic-message envelope))
      (cons nil (cons (litgraph--diagnostic-message envelope) nil)))
     (t (cons nil (cons (litgraph--describe-failure exit stdout stderr) nil))))))

(defun litgraph--report-error (message &optional _response)
  "Default error handler: remember MESSAGE and tell the user."
  (setq litgraph-last-error message)
  (message "litgraph: %s" message))

(defun litgraph-call (args callback &optional errback)
  "Run `litigate run' with ARGS; hand the engine response to CALLBACK.
ERRBACK receives an error message string (default: report in the echo
area) and the engine response when there was one."
  (funcall litgraph-process-function args nil
           (lambda (exit stdout stderr)
             (let ((decoded (litgraph--decode exit stdout stderr)))
               (if (car decoded)
                   (progn (setq litgraph-last-response (cadr decoded)
                                litgraph--last-raw (cddr decoded))
                          (funcall callback (cadr decoded)))
                 (funcall (or errback #'litgraph--report-error)
                          (cadr decoded) (cddr decoded)))))))

(defun litgraph-request (request callback &optional errback)
  "Run REQUEST (a hash table); call CALLBACK with the engine response.
ERRBACK, if given, receives the error message and the engine response."
  (litgraph-call (litgraph-request-args request) callback errback))

(defun litgraph-request-sync (request &optional timeout)
  "Send REQUEST and wait for the response, at most TIMEOUT seconds.
Emacs stays responsive to timers and process output meanwhile, and
\\[keyboard-quit] works.  Signal an error on failure."
  (let* ((box (list nil))
         (deadline (+ (float-time) (or timeout litgraph-timeout 120))))
    (litgraph-request request
                      (lambda (response) (setcar box (cons 'ok response)))
                      (lambda (message _response)
                        (setcar box (cons 'error message))))
    (while (and (not (car box)) (< (float-time) deadline))
      (accept-process-output nil 0.05))
    (pcase (car box)
      (`(ok . ,response) response)
      (`(error . ,message) (error "Litgraph: %s" message))
      (_ (litgraph-cancel) (error "Litgraph timed out")))))

(defun litgraph-response-error-p (response)
  "Non-nil if RESPONSE is not an ok envelope."
  (not (litgraph-get response 'ok)))


;;;; Engine metadata (describe / graph), cached

(defvar litgraph--cache (make-hash-table :test #'equal)
  "Cache of engine metadata keyed by binary, project and request.")

(defun litgraph-clear-cache ()
  "Forget cached describe and graph data."
  (interactive)
  (clrhash litgraph--cache)
  (message "litgraph: metadata cache cleared"))

(defun litgraph--cache-key (&rest parts)
  "Return a cache key: PARTS plus what identifies the engine and target."
  (cl-list* litgraph-executable (litgraph-project-root)
            (let ((target (litgraph--target)))
              (and target (litgraph-resolve-target target)))
            parts))

(defun litgraph--fetch-cached (key request callback &optional errback)
  "Call CALLBACK with the cached response for KEY, fetching it by REQUEST."
  (let ((hit (gethash key litgraph--cache)))
    (if hit
        (funcall callback hit)
      (litgraph-request request
                        (lambda (response)
                          (puthash key response litgraph--cache)
                          (funcall callback response))
                        errback))))

(defun litgraph--no-target (errback)
  "Tell ERRBACK (or the echo area) that no target is selected."
  (funcall (or errback #'litgraph--report-error)
           "no matter selected (`litgraph-set-target')" nil))

(defun litgraph-with-describe (callback &optional errback)
  "Call CALLBACK with the engine's `describe' of the current target (cached).
ERRBACK receives the error message if the call fails or there is no
target."
  (if (litgraph--target)
      (litgraph--fetch-cached (litgraph--cache-key 'describe)
                              (litgraph-build-request "describe")
                              callback errback)
    (litgraph--no-target errback)))

(defun litgraph--cached-describe ()
  "The cached `describe' result, or nil when not fetched yet."
  (litgraph-get (gethash (litgraph--cache-key 'describe) litgraph--cache)
                'result))

(defun litgraph--graph-request ()
  "The request that yields the graph for the current context."
  (litgraph-build-request "graph"))

(defun litgraph--graph-key ()
  "Cache key of the current context's graph."
  (litgraph--cache-key 'graph (litgraph-serialize (litgraph--graph-request))))

(defun litgraph-with-graph (callback &optional errback)
  "Call CALLBACK with the current context's `graph' result (cached).
ERRBACK receives the error message if the call fails."
  (if (litgraph--target)
      (litgraph--fetch-cached
       (litgraph--graph-key) (litgraph--graph-request)
       (lambda (response) (funcall callback (litgraph-get response 'result)))
       errback)
    (litgraph--no-target errback)))

(defun litgraph-prefetch ()
  "Warm the metadata cache in the background; never signals."
  (ignore-errors
    (when (litgraph--target)
      (litgraph-with-describe #'ignore #'ignore))))

(defconst litgraph--known-ops
  '("describe" "packs" "lint" "validate" "graph" "metric" "explain" "solve"
    "chain" "simulate" "path" "pareto" "sweep" "tornado" "structure"
    "deadlines" "compare" "batch" "calibration")
  "Ops offered before the binary has been asked.")

(defun litgraph-op-names ()
  "Names of every op the binary supports (falls back to a known list)."
  (or (mapcar (lambda (cell) (symbol-name (car cell)))
              (litgraph-get (litgraph--cached-describe) 'ops))
      litgraph--known-ops))

(defun litgraph-op-available-p (op)
  "Non-nil unless the binary is known not to support OP."
  (or (null (litgraph--cached-describe))
      (member op (litgraph-op-names))))

(defun litgraph-op-doc (op)
  "One-line description of OP from `describe', or nil."
  (litgraph-get (litgraph--cached-describe) 'ops (intern op)))

(defun litgraph--named (list id-key note-key)
  "Turn LIST of alists into (ID . NOTE) pairs using ID-KEY and NOTE-KEY."
  (mapcar (lambda (entry)
            (cons (litgraph-get entry id-key) (litgraph-get entry note-key)))
          list))

(defun litgraph-pack-candidates ()
  "Ids and titles of the target's packs, from `describe'."
  (litgraph--named (litgraph-get (litgraph--cached-describe) 'packs) 'id 'title))

(defun litgraph-calibration-candidates ()
  "Return the ids and titles of calibration data, from `describe'."
  (litgraph--named (litgraph-get (litgraph--cached-describe) 'calibration_sets)
                   'id 'title))

(defun litgraph-metric-candidates ()
  "Built-in metric names with their docs, from `describe'."
  (litgraph--named (litgraph-get (litgraph--cached-describe) 'metrics)
                   'name 'doc))

(defun litgraph-param-candidates ()
  "Scenario parameter names with their docs, from `describe'."
  (litgraph--named (litgraph-get (litgraph--cached-describe) 'params)
                   'name 'doc))

(defun litgraph-node-candidates (graph)
  "Node ids with labels from the `graph' result GRAPH."
  (litgraph--named (litgraph-get graph 'nodes) 'id 'label))

(defun litgraph-edge-candidates (graph)
  "Edge ids with labels from the `graph' result GRAPH."
  (litgraph--named (litgraph-get graph 'edges) 'id 'label))

(defun litgraph-terminal-candidates (graph)
  "Terminal node ids of GRAPH, plus the `terminals' and `tag:' path targets."
  (append
   '(("terminals" . "every terminal"))
   (mapcar (lambda (node) (cons (litgraph-get node 'id)
                                (litgraph-get node 'label)))
           (seq-filter (lambda (node)
                         (equal (litgraph-get node 'kind) "terminal"))
                       (litgraph-get graph 'nodes)))
   (mapcar (lambda (tag) (cons (concat "tag:" tag) "outcome tag"))
           (delete-dups
            (seq-mapcat (lambda (node)
                          (mapcar (lambda (tag) (format "%s" tag))
                                  (litgraph-get node 'outcome)))
                        (litgraph-get graph 'nodes))))))


;;;; Completion

(defun litgraph--completion-table (candidates category)
  "Completion table for CANDIDATES ((STRING . NOTE) or STRING) in CATEGORY."
  (let ((pairs (mapcar (lambda (c) (if (consp c) c (cons c nil))) candidates)))
    (lambda (string pred action)
      (if (eq action 'metadata)
          `(metadata
            (category . ,category)
            (display-sort-function . identity)
            (annotation-function
             . ,(lambda (candidate)
                  (when-let* ((note (cdr (assoc candidate pairs)))
                              ((stringp note)))
                    (concat "  " (truncate-string-to-width
                                  (replace-regexp-in-string "\n" " " note)
                                  70 nil nil "…"))))))
        (complete-with-action action (mapcar #'car pairs) string pred)))))

(defun litgraph-read (prompt candidates &optional category default require-match)
  "Read one of CANDIDATES with PROMPT; CANDIDATES may carry annotations."
  (completing-read (format-prompt prompt default)
                   (litgraph--completion-table candidates
                                               (or category 'litgraph))
                   nil require-match nil nil default))

(defun litgraph-read-multiple (prompt candidates &optional category)
  "Read several of CANDIDATES with PROMPT, returning a list of strings."
  (completing-read-multiple
   (format-prompt prompt nil)
   (litgraph--completion-table candidates (or category 'litgraph))))


;;;; Op arguments

(defconst litgraph--node-args '("node" "from" "to" "edge")
  "Op argument names that hold graph ids.")

(defun litgraph-op-arg-specs (describe op)
  "Argument specs for OP, parsed from its summary in DESCRIBE's `ops'.
The summary opens with the op's fields, as in \"{param, lo?, what:
summary|scc}\": a trailing `?' marks an optional one and a `|' list
its choices.  Return a list of plists with :name, :type, :required,
:doc and :enum."
  (when-let* ((doc (litgraph-get describe 'ops (intern op)))
              ((string-match "{\\([^}]*\\)}" doc)))
    (delq nil
          (mapcar
           (lambda (part)
             (let* ((part (string-trim part))
                    (name (and (string-match "\\`\\([a-z_]+\\)\\(\\?\\)?" part)
                               (match-string 1 part)))
                    (optional (and name (match-string 2 part)))
                    (doc (and (string-match ":\\s-*\\(.*\\)\\'" part)
                              (string-trim (match-string 1 part))))
                    (enum (and doc
                               (string-match-p "\\`[a-z_-]+\\(?:|[a-z_-]+\\)+\\'" doc)
                               (split-string doc "|"))))
               (when name
                 (list :name name
                       :type (if enum 'enum 'guess)
                       :required (not optional)
                       :doc doc
                       :enum enum))))
           (split-string (match-string 1 doc) "," t)))))

(defun litgraph-coerce-value (type text)
  "Convert the string TEXT to a JSON value of TYPE."
  (let ((text (string-trim text)))
    (pcase type
      ((or 'integer 'number)
       (if (string-match-p "\\`-?[0-9]+\\(\\.[0-9]+\\)?\\([eE][-+]?[0-9]+\\)?\\'"
                           text)
           (string-to-number text)
         (user-error "Not a number: %s" text)))
      ('boolean (if (member (downcase text) '("t" "true" "yes" "y" "1"))
                    t :false))
      ('strings (litgraph-vec (split-string text "[ ,]+" t)))
      ('json (litgraph-parse-strict text))
      ('guess
       (cond ((string-match-p "\\`-?[0-9]+\\(\\.[0-9]+\\)?\\'" text)
              (string-to-number text))
             ((member text '("true" "false")) (if (equal text "true") t :false))
             ((string-match-p "\\`[[{]" text) (litgraph-parse-strict text))
             (t text)))
      (_ text))))

(defun litgraph--arg-candidates (spec graph describe)
  "Completion candidates for the argument SPEC, or nil.
GRAPH and DESCRIBE are the parsed graph and describe results."
  (let ((name (plist-get spec :name)))
    (cond
     ((plist-get spec :enum) (plist-get spec :enum))
     ((equal name "edge") (litgraph-edge-candidates graph))
     ((equal name "to") (litgraph-terminal-candidates graph))
     ((member name litgraph--node-args) (litgraph-node-candidates graph))
     ((member name '("metric" "metrics" "objectives" "spec"))
      (litgraph-metric-candidates))
     ((member name '("param" "params" "watch"))
      (litgraph-param-candidates))
     (t (ignore describe) nil))))

(defun litgraph--read-arg (spec graph describe)
  "Prompt for the argument SPEC; return its JSON value, or nil to skip.
GRAPH and DESCRIBE supply completion candidates."
  (let* ((name (plist-get spec :name))
         (type (plist-get spec :type))
         (candidates (litgraph--arg-candidates spec graph describe))
         (doc (plist-get spec :doc))
         (prompt (format "%s%s" name (if doc (format " (%s)"
                                                     (car (split-string doc "\n")))
                                       "")))
         (text
          (cond
           ((equal name "trigger") (org-read-date nil nil nil "Trigger date: "))
           ((eq type 'boolean)
            (if (y-or-n-p (format "%s? " name)) "true" "false"))
           ((and candidates (memq type '(strings)))
            (string-join (litgraph-read-multiple prompt candidates
                                                 (intern name))
                         ","))
           (candidates
            (litgraph-read prompt candidates (intern name) nil
                           (memq type '(enum))))
           (t (read-string (format-prompt prompt nil))))))
    (unless (string-empty-p (string-trim text))
      (litgraph-coerce-value type text))))

(defun litgraph-read-op-arguments (op describe graph &optional all)
  "Prompt for the arguments of OP; return the op object (a hash table).
Required arguments are always asked; optional ones are chosen from a
list (or all asked when ALL).  DESCRIBE and GRAPH are the parsed
metadata used for argument names and completion."
  (let* ((specs (litgraph-op-arg-specs describe op))
         (object (litgraph-obj "op" op))
         (required (seq-filter (lambda (s) (plist-get s :required)) specs))
         (optional (seq-remove (lambda (s) (plist-get s :required)) specs))
         (chosen
          (if (or all (null optional))
              optional
            (let ((names (litgraph-read-multiple
                          (format "Optional %s arguments (empty to skip)" op)
                          (mapcar (lambda (s)
                                    (cons (plist-get s :name)
                                          (plist-get s :doc)))
                                  optional)
                          'litgraph-arg)))
              (seq-filter (lambda (s) (member (plist-get s :name) names))
                          optional)))))
    (dolist (spec (append required chosen))
      (when-let* ((value (litgraph--read-arg spec graph describe)))
        (puthash (plist-get spec :name) value object)))
    object))


;;;; Running ops

(defun litgraph--display (response request raw)
  "Show RESPONSE for REQUEST (RAW is its JSON text) with the display function."
  (funcall litgraph-display-function response request raw))

(defun litgraph-run-request (request &optional callback)
  "Send REQUEST, then display the response (or call CALLBACK with it)."
  (message "litgraph: running %s…" (litgraph--request-op-name request))
  (litgraph-request request
                    (lambda (response)
                      (if callback
                          (funcall callback response)
                        (litgraph--display response request
                                           litgraph--last-raw)))))

(defun litgraph--request-op-name (request)
  "The op name of REQUEST (hash table)."
  (let ((op (gethash "op" request)))
    (if (hash-table-p op) (gethash "op" op) op)))

(defun litgraph--needs-graph-p (op)
  "Non-nil if prompting for OP's arguments benefits from the graph."
  (member op '("explain" "solve" "chain" "simulate" "path" "pareto" "structure"
               "graph" "deadlines" "calibration")))

;;;###autoload
(defun litgraph-run-op (op &optional all)
  "Run OP asynchronously, asking for its arguments; show the result.
Interactively, complete OP from the binary's own list of ops.  With a
prefix argument ALL, ask for every optional argument as well."
  (interactive
   (list (progn
           (litgraph-prefetch)
           (litgraph-read "litgraph op" (litgraph--op-candidates) 'litgraph-op
                          nil t))
         current-prefix-arg))
  (litgraph--with-metadata
   op
   (lambda (describe graph)
     (litgraph-run-request
      (litgraph-build-request
       (litgraph-read-op-arguments op describe graph all))))))

(defun litgraph--op-candidates ()
  "Ops with their one-line docs."
  (mapcar (lambda (op) (cons op (litgraph-op-doc op))) (litgraph-op-names)))

(defun litgraph--with-metadata (op callback)
  "Fetch describe and (when useful for OP) the graph, then call CALLBACK.
CALLBACK receives (DESCRIBE GRAPH) as parsed results.  Signal a
`user-error' when no target is selected."
  (unless (litgraph--target)
    (user-error "No matter selected: choose a .case file or package with `litgraph-set-target'"))
  (litgraph-with-describe
   (lambda (describe)
     (let ((describe (litgraph-get describe 'result)))
       (if (litgraph--needs-graph-p op)
           (litgraph-with-graph
            (lambda (graph) (funcall callback describe graph))
            (lambda (_message &optional _response)
              (funcall callback describe nil)))
         (funcall callback describe nil))))))

(defmacro litgraph--define-op-command (name op doc)
  "Define the interactive command NAME to run OP.  DOC is its docstring."
  (declare (indent 2))
  `(defun ,name (&optional all)
     ,(concat doc "\n\nWith prefix argument ALL, ask for every optional argument.")
     (interactive "P")
     (litgraph-run-op ,op all)))

;;;###autoload (autoload 'litgraph-solve "litgraph" nil t)
(litgraph--define-op-command litgraph-solve "solve"
  "Solve the current target: value and best line.")
;;;###autoload (autoload 'litgraph-chain "litgraph" nil t)
(litgraph--define-op-command litgraph-chain "chain"
  "Absorption probabilities and expected totals under the optimal policy.")
;;;###autoload (autoload 'litgraph-explain "litgraph" nil t)
(litgraph--define-op-command litgraph-explain "explain"
  "Explain the decision at a node: every option with value and regret.")
;;;###autoload (autoload 'litgraph-simulate "litgraph" nil t)
(litgraph--define-op-command litgraph-simulate "simulate"
  "Monte Carlo outcome distribution, CVaR and P(loss).")
;;;###autoload (autoload 'litgraph-path "litgraph" nil t)
(litgraph--define-op-command litgraph-path "path"
  "Shortest lines to a node, the terminals, or an outcome tag.")
;;;###autoload (autoload 'litgraph-pareto "litgraph" nil t)
(litgraph--define-op-command litgraph-pareto "pareto"
  "Cost by time by likelihood Pareto frontier.")
;;;###autoload (autoload 'litgraph-sweep "litgraph" nil t)
(litgraph--define-op-command litgraph-sweep "sweep"
  "Value curve and policy breakpoints as one parameter varies.")
;;;###autoload (autoload 'litgraph-tornado "litgraph" nil t)
(litgraph--define-op-command litgraph-tornado "tornado"
  "Which inputs the answer is most sensitive to.")
;;;###autoload (autoload 'litgraph-structure "litgraph" nil t)
(litgraph--define-op-command litgraph-structure "structure"
  "Structural analyses: cycles, dominators, min-cut, reachability.")
;;;###autoload (autoload 'litgraph-lint "litgraph" nil t)
(litgraph--define-op-command litgraph-lint "lint"
  "Content diagnostics for the target's packs.")
;;;###autoload (autoload 'litgraph-list-packs "litgraph" nil t)
(litgraph--define-op-command litgraph-list-packs "packs"
  "List the target's packs with data quality.")

;;;###autoload
(defun litgraph-compare ()
  "Compare the current target with a variant, over an inner op."
  (interactive)
  (litgraph--with-metadata
   "compare"
   (lambda (describe graph)
     (let* ((inner (litgraph-read "Inner op" (litgraph--op-candidates)
                                  'litgraph-op "chain" t))
            (inner-object (litgraph-read-op-arguments inner describe graph))
            (variant (litgraph-parse-strict
                      (read-string "Variant (JSON merge-patch): "
                                   "{\"params\": {}}"))))
       (litgraph-run-request
        (litgraph-build-request
         (litgraph-obj "op" "compare" "variant" variant
                       "inner" inner-object)))))))


;;;; Session commands

;;;###autoload
(defun litgraph-set-target (target)
  "Select TARGET for this session: a `.case' matter or a package.
Interactively, complete from the project's `cases/' and `lit/', or
name any file or directory.  An empty answer clears the selection."
  (interactive (list (litgraph-read-target)))
  (setq litgraph-target (and target (not (string-empty-p target))
                             (litgraph-resolve-target target)))
  (litgraph-prefetch)
  (message "litgraph: %s" (litgraph-context-label)))

(defun litgraph-read-target ()
  "Read a target: a project matter or package, or (on RET) any file."
  (let* ((root (litgraph-project-root))
         (choice (litgraph-read "Matter or package (RET for a file)"
                                (litgraph-target-candidates) 'litgraph-target)))
    (cond ((not (string-empty-p choice))
           (if root (expand-file-name choice root) choice))
          (t (read-file-name "Matter (.case), .lit file or package directory: "
                             root nil t nil
                             (lambda (f) (or (file-directory-p f)
                                             (string-suffix-p ".case" f)
                                             (string-suffix-p ".lit" f))))))))

;;;###autoload
(defun litgraph-set-param (name value)
  "Override scenario parameter NAME with numeric VALUE for this session.
A nil VALUE removes the override."
  (interactive
   (progn
     (litgraph-prefetch)
     (let* ((name (litgraph-read "Parameter" (litgraph-param-candidates)
                                 'litgraph-param))
            (text (read-string (format "%s (empty to remove): " name))))
       (list name (and (not (string-empty-p text)) (string-to-number text))))))
  (setq litgraph-params (assoc-delete-all name (copy-alist litgraph-params)))
  (when value (push (cons name value) litgraph-params))
  (message "litgraph: %s" (litgraph-context-label)))

;;;###autoload
(defun litgraph-clear-context ()
  "Forget the session target and parameters."
  (interactive)
  (setq litgraph-target nil litgraph-params nil)
  (message "litgraph: context cleared"))

;;;###autoload
(defun litgraph-describe ()
  "Show the engine's `describe' manual for the current target."
  (interactive)
  (litgraph-run-request (litgraph-build-request "describe")))

;;;###autoload
(defun litgraph-refresh ()
  "Drop cached metadata and refetch it in the background."
  (interactive)
  (clrhash litgraph--cache)
  (litgraph-prefetch)
  (message "litgraph: refreshing metadata"))

;;;###autoload
(defun litgraph-version ()
  "Report the `litigate' version."
  (interactive)
  (funcall litgraph-process-function '("--version") nil
           (lambda (exit stdout stderr)
             (if (eq exit 0)
                 (message "litgraph: %s (%s)" (string-trim stdout)
                          (litgraph-executable-path))
               (litgraph--report-error
                (litgraph--describe-failure exit stdout stderr))))))

;;;###autoload (autoload 'litgraph-command-map "litgraph" nil nil 'keymap)
(defvar litgraph-command-map
  (let ((map (make-sparse-keymap)))
    (define-key map "m" #'litgraph-menu)
    (define-key map "r" #'litgraph-run-op)
    (define-key map "s" #'litgraph-solve)
    (define-key map "c" #'litgraph-chain)
    (define-key map "x" #'litgraph-explain)
    (define-key map "S" #'litgraph-simulate)
    (define-key map "p" #'litgraph-path)
    (define-key map "t" #'litgraph-tornado)
    (define-key map "w" #'litgraph-sweep)
    (define-key map "g" #'litgraph-graph)
    (define-key map "d" #'litgraph-deadlines-to-org)
    (define-key map "T" #'litgraph-timeline)
    (define-key map "C" #'litgraph-set-target)
    (define-key map "." #'litgraph-set-param)
    (define-key map "k" #'litgraph-cancel)
    map)
  "Keymap of litgraph commands; bind it to a prefix of your choice.")

(provide 'litgraph)

;;; litgraph.el ends here
