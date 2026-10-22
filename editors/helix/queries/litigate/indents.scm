; Generated from editors/tree-sitter-litigate/queries/indents.scm by
; editors/helix/sync-queries.sh. Do not edit; edit the source and rerun.

; Indentation (two spaces per level, SPEC §10). Braced, bracketed and
; parenthesized bodies indent their contents; the closing delimiter dedents.
[
  (package_decl)
  (variant_list)
  (param_list)
  (node_body)
  (edge_fields)
  (duration_range)
  (deadline_opts)
  (instance_decl)
  (probability_map)
  (role_map)
  (rule_body)
  (counting_body)
  (service_map)
  (short_period)
  (calendar_body)
  (transition_body)
  (test_body)
  (source_body)
  (parties_block)
  (events_block)
  (facts_block)
  (stakes_block)
  (beliefs_block)
  (count_map)
  (num_record)
  (money_record)
  (json_record)
  (json_list)
  (id_list)
  (import_list)
  (record_expression)
  (list_expression)
  (arguments)
  (match_expression)
] @indent

[ "}" "]" ")" ] @outdent

