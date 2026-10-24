; Helix textobjects for litigate (hand-written; not generated).
; function: declarations with a body (`mf`, `]f`); test: test blocks;
; entry: edges and links; parameter: event params and call arguments.

(node_decl body: (_) @function.inside) @function.around
(rule_decl body: (_) @function.inside) @function.around
(counting_decl body: (_) @function.inside) @function.around
(calendar_decl body: (_) @function.inside) @function.around
(transition_decl body: (_) @function.inside) @function.around
(grant_decl body: (_) @function.inside) @function.around

[
  (node_decl)
  (rule_decl)
  (deadline_decl)
  (counting_decl)
  (calendar_decl)
  (transition_decl)
  (grant_decl)
  (default_decl)
  (exception_decl)
  (event_decl)
  (type_decl)
  (enum_decl)
  (instance_decl)
] @function.around

(test_decl body: (_) @test.inside) @test.around

[
  (edge_decl)
  (link_decl)
] @entry.around

(edge_decl fields: (_) @entry.inside)
(link_decl fields: (_) @entry.inside)

(param) @parameter.inside
(arguments (_) @parameter.inside)

[
  (line_comment)
  (block_comment)
  (doc_comment)
] @comment.inside

(line_comment)+ @comment.around
(doc_comment)+ @comment.around
(block_comment) @comment.around
