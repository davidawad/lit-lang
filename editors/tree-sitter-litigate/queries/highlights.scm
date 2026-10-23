; Syntax highlighting for litigate (.lit and .case).
; Capture names follow the common tree-sitter convention shared by Neovim,
; Helix and Zed. Later patterns win in editors that use "last match wins",
; so general captures come first and specific ones after.

(identifier) @variable

; Declaration keywords.
[
  "module" "import" "package" "start" "group" "type" "enum" "event"
  "state" "decision" "chance" "terminal" "link" "instance" "rule"
  "deadline" "counting" "calendar" "transition" "authority" "default"
  "exception" "test"
  "case" "uses" "as_of" "known_at" "summary" "notes" "source" "perspective"
  "parties" "events" "facts" "stakes" "beliefs" "observe" "analysis"
] @keyword

; Clause and field keywords.
[
  "forum" "by" "outcome" "p" "effort" "cost" "within" "duration" "min" "mode"
  "max" "sets" "clears" "requires" "forbids" "replaces" "tags" "attrs" "note"
  "valence" "court_listener_url" "action_id" "extendable" "extension_authority"
  "payoff" "payoff_by_flag" "pack" "remove_edges" "probabilities" "roles"
  "payoff_transform" "patch" "duty" "power" "liability" "from" "after" "before"
  "counted_by" "exclude_trigger_day" "include_last_day" "roll_forward_on"
  "service_extension" "holidays" "anchor" "short_period" "below" "count"
  "allow_hours" "end_of_month" "effective" "observed" "amends" "subject_to" "notwithstanding"
  "supersedes" "revives" "override" "delegable_to" "rebuts" "undercuts"
  "priority" "url"
] @keyword

(discretion) @keyword

[ "when" "unless" "otherwise" "match" ] @keyword.conditional

[ "given" "expect" "on" ] @keyword

"cite" @keyword.directive

; Literals.
(string) @string
(escape_sequence) @string.escape
(cite_text) @string.special
(number) @number
(money) @number
(duration) @number
(date) @number
(version_req) @string.special
(boolean) @boolean
[(unit) (unset) (unvalued)] @constant.builtin
(hole) @constant.builtin
(open_end) @constant.builtin
[(anonymous) (wildcard)] @variable.builtin

; Comments.
(line_comment) @comment
(block_comment) @comment
(doc_comment) @comment.documentation

; Operators and punctuation.
[
  "->" "=>" "==" "!=" "<=" ">=" "&&" "||" "+" "-" "*" "/" "%" "!" "="
] @operator

[ "(" ")" "[" "]" "{" "}" ] @punctuation.bracket

[ "," ":" "::" "." "@" ] @punctuation.delimiter

; Declared names.
(module_decl path: (module_path (identifier) @module))
(import_decl path: (module_path (identifier) @module))
(import_list (identifier) @variable)
(package_decl name: (identifier) @module)
(package_decl forum: (identifier) @constant)
(package_decl authority: (identifier) @type)
(use_item package: (identifier) @module)

(type_decl name: (identifier) @type.definition)
(enum_decl name: (identifier) @type.definition)
(variant_list (identifier) @constant)
(event_decl name: (identifier) @type.definition)
(type name: (identifier) @type)
(param name: (identifier) @variable.parameter)
(authority_decl level: (identifier) @type)

(node_decl name: (identifier) @label)
(start_decl node: (identifier) @label)
(edge_decl name: (identifier) @property)
(link_decl name: (identifier) @property)
(group_decl name: (identifier) @label)
(by_clause actor: (ref (identifier) @type))
(party_decl actor: (identifier) @type)
(party_decl entity: (identifier) @type)
(party_decl name: (identifier) @variable)

(rule_decl name: (identifier) @function)
(deadline_decl name: (identifier) @function)
(counting_decl name: (identifier) @function)
(calendar_decl name: (identifier) @function)
(holiday_decl name: (identifier) @property)
(transition_decl name: (identifier) @function)
(default_decl name: (identifier) @function)
(exception_decl name: (identifier) @function)
(instance_decl base: (identifier) @module)
(instance_decl origin: (identifier) @module)

(test_decl name: (string) @string.special)

; Record and map keys.
[
  (num_entry key: (identifier) @property)
  (money_entry key: (identifier) @property)
  (json_entry key: (_) @property)
  (record_field key: (identifier) @property)
  (probability_entry key: (identifier) @property)
  (role_entry key: (identifier) @property)
  (service_entry method: (identifier) @constant)
]

(field_expression field: (identifier) @property)

; Calls: event constructors, holiday rules and builtins.
(call_expression function: (ref (identifier) @function.call .))

((call_expression function: (ref . (identifier) @function.builtin .))
  (#any-of? @function.builtin
    "deadline" "has_event" "prob" "given" "in" "intersect" "disjoint" "len" "contains"
    "fixed" "nth" "last" "every" "once"))
