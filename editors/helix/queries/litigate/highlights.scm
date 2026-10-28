; Generated from editors/tree-sitter-litigate/queries/highlights.scm by
; `just editor-queries`. Do not edit; edit the source and rerun.

; Generated from highlights.scm.in and crates/litigate-syntax/keywords.txt by
; `just editor-queries`. Do not edit; edit those and rerun.

; Syntax highlighting for litigate (.lit and .case).
; Capture names follow the common tree-sitter convention shared by Neovim,
; Helix and Zed. Later patterns win in editors that use "last match wins",
; so general captures come first and specific ones after.

(identifier) @variable

; Declaration keywords.
[
  "module" "import" "package" "start" "group" "type" "enum" "state" "decision"
  "chance" "terminal" "rule" "deadline" "counting" "calendar" "transition"
  "authority" "default" "exception" "test" "link" "instance" "event" "case"
  "uses" "as_of" "known_at" "summary" "notes" "source" "perspective" "parties"
  "events" "facts" "stakes" "beliefs" "observe" "prior" "opponents"
  "adversarial" "own_objective" "analysis" "edition" "limit" "grant"
  "recharacterize" "invalidate" "governing_law" "calibration"
] @keyword

; Clause and field keywords.
[
  "forum" "duty" "power" "liability" "amends" "allow_hours" "end_of_month"
  "below" "count" "by" "docket" "from" "after" "before" "counted_by" "outcome"
  "payoff" "payoff_by_flag" "fee_shift" "info" "p" "effort" "cost" "within"
  "duration" "min" "mode" "max" "sets" "clears" "requires" "forbids" "tags"
  "attrs" "note" "valence" "basis" "court_listener_url" "action_id" "extendable"
  "extension_authority" "replaces" "pack" "remove_edges" "probabilities" "roles"
  "payoff_transform" "stake" "patch" "effective" "exclude_trigger_day"
  "include_last_day" "roll_forward_on" "service_extension" "holidays" "anchor"
  "short_period" "observed" "subject_to" "notwithstanding" "supersedes"
  "revives" "interprets" "delegable_to" "override" "rebuts" "undercuts"
  "priority" "url" "bearer" "trigger" "extend" "waive" "after_expiry" "never"
  "removes" "elects" "extends" "waives" "to" "where" "designated"
  "interpretation" "formerly" "forums" "forward_holidays" "declared_holidays"
  "quote" "words" "pages" "lines" "certificate" "excludes" "ecf" "for" "paper"
  "delegable" "op" "unlock" "characterization" "retracted" "unknown_before"
  "commenced" "spawn" "joins" "anytime" "lock" "against" "Rule" "vintage"
  "retrieved" "n" "method" "attr"
] @keyword

(discretion) @keyword

; These words are keywords only in typed guard operations, never in expressions.
(guard_op operator: ["and" "or"] @keyword)

[
  "match" "when" "unless" "otherwise"
] @keyword.control.conditional

[
  "on" "given" "expect"
] @keyword

[
  "cite"
] @keyword.directive

; Literals.
(string) @string
(escape_sequence) @constant.character.escape
(number) @constant.numeric
(money) @constant.numeric
(duration) @constant.numeric
(date) @constant.numeric
(version_req) @string.special
(boolean) @constant.builtin.boolean
[(unit) (unvalued)] @constant.builtin
(hole) @constant.builtin
(open_end) @constant.builtin
[(anonymous) (wildcard)] @variable.builtin

; Comments.
(line_comment) @comment
(block_comment) @comment
(doc_comment) @comment.line.documentation

; Operators and punctuation.
[
  "->" "=>" "==" "!=" "<=" ">=" "&&" "||" "+" "-" "*" "/" "%" "!" "="
] @operator

[ "(" ")" "[" "]" "{" "}" ] @punctuation.bracket

[ "," ":" "::" "." "@" "|" ] @punctuation.delimiter

; Declared names.
(module_decl path: (module_path (identifier) @namespace))
(import_decl path: (module_path (identifier) @namespace))
(import_list (identifier) @variable)
(package_decl name: (identifier) @namespace)
(package_decl forum: (identifier) @constant)
(package_decl authority: (identifier) @type)
(use_item package: (identifier) @namespace)

(type_decl name: (identifier) @type)
(enum_decl name: (identifier) @type)
(variant_list (identifier) @constant)
(event_decl name: (identifier) @type)
(type name: (identifier) @type)
(param name: (identifier) @variable.parameter)
(authority_decl level: (identifier) @type)

(node_decl name: (identifier) @label)
(start_decl node: (identifier) @label)
(edge_decl name: (identifier) @variable.other.member)
(link_decl name: (identifier) @variable.other.member)
(group_decl name: (identifier) @label)
(by_clause actor: (ref (identifier) @type))
(party_decl actor: (identifier) @type)
(party_decl entity: (identifier) @type)
(party_decl name: (identifier) @variable)
(case_event id: (identifier) @label)

(rule_decl name: (identifier) @function)
(deadline_decl name: (identifier) @function)
(counting_decl name: (identifier) @function)
(calendar_decl name: (identifier) @function)
(holiday_decl name: (identifier) @variable.other.member)
(transition_decl name: (identifier) @function)
(grant_decl name: (identifier) @function)
(recharacterize_decl name: (identifier) @function)
(invalidate_decl name: (identifier) @function)
(default_decl name: (identifier) @function)
(exception_decl name: (identifier) @function)
(instance_decl base: (identifier) @namespace)
(instance_decl origin: (identifier) @namespace)

(test_decl name: (string) @string.special)

; Record and map keys.
[
  (num_entry key: (identifier) @variable.other.member)
  (money_entry key: (identifier) @variable.other.member)
  (record_field key: (identifier) @variable.other.member)
  (probability_entry key: (identifier) @variable.other.member)
  (role_entry key: (identifier) @variable.other.member)
  (service_entry method: (identifier) @constant)
]

(field_expression field: (identifier) @variable.other.member)

; Calls: event constructors, holiday rules and builtins.
(call_expression function: (ref (identifier) @function .))

((call_expression function: (ref . (identifier) @function.builtin .))
  (#any-of? @function.builtin
    "deadline" "has_event" "prob" "given" "in" "intersect" "disjoint" "len" "contains"
    "credal" "lower" "upper" "mean" "at"
    "fixed" "nth" "last" "every" "once"))
