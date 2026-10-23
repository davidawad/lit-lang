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
  (deadline_decl_opts)
  (capability)
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
  (id_list)
  (import_list)
  (record_expression)
  (list_expression)
  (arguments)
  (match_expression)
] @indent.begin

[ "}" "]" ")" ] @indent.branch @indent.end

[ (block_comment) (string) ] @indent.auto
