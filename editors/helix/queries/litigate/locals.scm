; Generated from editors/tree-sitter-litigate/queries/locals.scm by
; `just editor-queries`. Do not edit; edit the source and rerun.

; Scopes, definitions and references for litigate.
; A file is one scope; rule bodies nest (their declarations are hoisted into
; the module by the compiler, but the editor view keeps them local so
; highlighting a rule's own names stays precise). Node bodies scope edge ids,
; calendars scope holiday ids, event declarations scope their parameters.

(source_file) @local.scope
(rule_body) @local.scope
(node_body) @local.scope
(calendar_body) @local.scope
(event_decl) @local.scope

(type_decl name: (identifier) @local.definition.type)
(enum_decl name: (identifier) @local.definition.type)
(variant_list (identifier) @local.definition.constant)
(event_decl name: (identifier) @local.definition.type)
(param name: (identifier) @local.definition.variable.parameter)

(node_decl name: (identifier) @local.definition.label)
(edge_decl name: (identifier) @local.definition.variable.other.member)
(group_decl name: (identifier) @local.definition.label)
(rule_decl name: (identifier) @local.definition.function)
(deadline_decl name: (identifier) @local.definition.function)
(counting_decl name: (identifier) @local.definition.function)
(calendar_decl name: (identifier) @local.definition.function)
(holiday_decl name: (identifier) @local.definition.variable.other.member)
(transition_decl name: (identifier) @local.definition.function)
(grant_decl name: (identifier) @local.definition.function)
(recharacterize_decl name: (identifier) @local.definition.function)
(invalidate_decl name: (identifier) @local.definition.function)
(default_decl name: (identifier) @local.definition.function)
(exception_decl name: (identifier) @local.definition.function)
(import_list (identifier) @local.definition.variable)
(party_decl name: (identifier) @local.definition.label)
(case_event id: (identifier) @local.definition.label)

(ref . (identifier) @local.reference)
(start_decl node: (identifier) @local.reference)
(node_field value: (identifier) @local.reference)
(type name: (identifier) @local.reference)
