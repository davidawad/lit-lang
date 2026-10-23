/**
 * @file `.lit` rule-module rules (SPEC §3.1).
 */

/* eslint-disable no-undef */

const { braced, commaList, EDGE_ID_WORDS } = require('./helpers');

module.exports = {
  // ---------------------------------------------------------------- .lit

  lit_file: $ => seq(
    $.module_decl,
    repeat($.import_decl),
    optional($.package_decl),
    repeat($._item),
  ),

  module_decl: $ => seq('module', field('path', $.module_path)),

  module_path: $ => seq($.identifier, repeat(seq('.', $.identifier))),

  import_decl: $ => seq(
    'import',
    field('path', $.module_path),
    optional(seq('.', $.import_list)),
  ),

  import_list: $ => commaList('{', $.identifier, '}'),

  package_decl: $ => seq(
    'package',
    field('name', $.identifier),
    '{',
    'forum', field('forum', $.identifier), optional(','),
    'authority', field('authority', $.identifier),
    '}',
  ),

  _item: $ => choice(
    $.start_decl,
    $.group_decl,
    $.type_decl,
    $.enum_decl,
    $.event_decl,
    $.node_decl,
    $.link_decl,
    $.instance_decl,
    $.rule_decl,
    $.deadline_decl,
    $.counting_decl,
    $.calendar_decl,
    $.transition_decl,
    $.authority_decl,
    $.default_decl,
    $.exception_decl,
    $.test_decl,
  ),

  start_decl: $ => seq('start', field('node', $.identifier)),

  group_decl: $ => seq('group', field('name', $.identifier), ':', field('label', $.string)),

  type_decl: $ => seq(
    'type',
    field('name', $.identifier),
    '=',
    field('type', $._type),
    optional(field('refinement', $.where_clause)),
  ),

  enum_decl: $ => seq('enum', field('name', $.identifier), $.variant_list),

  variant_list: $ => seq('{', $.identifier, repeat(seq(',', $.identifier)), optional(','), '}'),

  event_decl: $ => seq(
    'event',
    field('name', $.identifier),
    $.param_list,
    optional(field('effect', $.event_effect)),
  ),

  // SPEC §5.2: the one effect an event type may declare.
  event_effect: $ => choice(
    seq(field('key', choice('removes', 'elects')), field('param', $.identifier)),
    seq(field('key', 'extends'), field('deadline', $.ref), 'to', field('param', $.identifier)),
    seq(field('key', 'waives'), field('deadline', $.ref)),
  ),

  param_list: $ => commaList('(', $.param, ')'),

  param: $ => seq(
    field('name', $.identifier),
    ':',
    field('type', $._type),
    optional(field('refinement', $.where_clause)),
  ),

  _type: $ => choice($.type, $.refined_type),

  type: $ => seq(field('name', $.identifier), optional($.type_args)),

  type_args: $ => seq('<', $._type, repeat(seq(',', $._type)), '>'),

  // SPEC §3.5: refinement syntax is reserved for the SMT tier. Both forms
  // parse here so editors highlight them; the compiler rejects them (L0009).
  refined_type: $ => seq(
    '{',
    field('binder', $.identifier),
    ':',
    field('base', $._type),
    '|',
    field('predicate', $._expression),
    '}',
  ),

  where_clause: $ => seq('where', field('predicate', $._expression)),

  // Nodes and edges.

  node_decl: $ => seq(
    field('kind', choice('state', 'decision', 'chance', 'terminal')),
    field('name', $.identifier),
    ':',
    field('label', $.string),
    repeat($._node_clause),
    optional(field('body', $.node_body)),
  ),

  _node_clause: $ => choice(
    $.by_clause,
    $.discretion,
    $.outcome_clause,
    $._version_clause,
    $.relation,
    $.cite,
  ),

  by_clause: $ => seq('by', field('actor', $.ref)),

  discretion: _ => 'discretion',

  outcome_clause: $ => seq('outcome', $.id_list),

  node_body: $ => braced(choice($.node_field, $.edge_decl)),

  node_field: $ => choice(
    seq(field('key', 'group'), field('value', $.identifier)),
    seq(field('key', 'note'), field('value', $.string)),
    seq(field('key', 'valence'), field('value', $.identifier)),
    seq(field('key', 'court_listener_url'), field('value', $.string)),
    seq(field('key', 'tags'), field('value', $.id_list)),
    seq(field('key', 'attrs'), field('value', $.num_record)),
    seq(field('key', 'payoff'), field('value', choice($.signed_money, $.hole))),
    seq(field('key', 'payoff_by_flag'), field('value', $.money_record)),
  ),

  // SPEC §3.4: `Ident|_ String ->` is an edge; the field keywords are also
  // valid edge ids, so they are aliased back to identifiers here.
  edge_decl: $ => seq(
    field('name', choice(
      $.identifier,
      alias('_', $.anonymous),
      ...EDGE_ID_WORDS.map(word => alias(word, $.identifier)),
    )),
    field('label', $.string),
    '->',
    field('target', $.ref),
    repeat($._edge_clause),
    optional(field('fields', $.edge_fields)),
  ),

  _edge_clause: $ => choice($.by_clause, $.probability),

  probability: $ => seq('p', field('value', $._prob_value)),

  _prob_value: $ => choice($.fraction, $.number, $.hole, $.parenthesized_expression),

  fraction: $ => seq(field('numerator', $.number), '/', field('denominator', $.number)),

  edge_fields: $ => braced($.edge_field),

  edge_field: $ => choice(
    seq(field('key', 'event'), field('value', $.call_expression)),
    seq(field('key', 'authority'), field('value', choice($.string, $.cite))),
    seq(field('key', 'effort'), field('value', $.duration)),
    seq(field('key', 'cost'), field('value', choice($.signed_money, $.hole))),
    seq(field('key', 'within'), field('value', $.ref), optional($.deadline_opts)),
    seq(field('key', 'deadline'), field('value', $.duration), optional($.deadline_opts)),
    seq(field('key', 'duration'), field('value', choice($.duration, $.duration_range))),
    seq(field('key', 'valence'), field('value', $.identifier)),
    seq(
      field('key', choice('court_listener_url', 'action_id', 'note')),
      field('value', $.string),
    ),
    seq(
      field('key', choice('tags', 'sets', 'clears', 'requires', 'forbids')),
      field('value', $.id_list),
    ),
    // A string entry names an anonymous edge by derived id (SPEC §3.1).
    seq(field('key', 'replaces'), field('value', $.replaces_list)),
    seq(field('key', 'attrs'), field('value', $.num_record)),
  ),

  // `min` and `max` are optional; `mode` is required (SPEC §3.1).
  duration_range: $ => seq(
    '{',
    optional(seq('min', field('min', $.duration), optional(','))),
    'mode', field('mode', $.duration),
    optional(seq(optional(','), 'max', field('max', $.duration))),
    '}',
  ),

  replaces_list: $ => commaList('[', choice($.ref, $.string), ']'),

  deadline_opts: $ => braced($.deadline_opt),

  deadline_opt: $ => choice(
    seq(field('key', 'extendable'), field('value', $.boolean)),
    seq(field('key', choice('extension_authority', 'note')), field('value', $.string)),
  ),

  // A deadline declaration's options add the obligation it creates
  // (SPEC §5.3): bearer, trigger policy, and capabilities.
  deadline_decl_opts: $ => braced(choice(
    $.deadline_opt,
    seq(field('key', 'bearer'), field('value', $.identifier)),
    seq(field('key', 'trigger'), field('value', choice($.identifier, $.call_expression))),
    seq(field('key', choice('extend', 'waive')), field('value', $.capability)),
  )),

  capability: $ => braced(choice(
    seq(field('key', 'by'), field('value', $.identifier)),
    seq(field('key', 'max'), field('value', $.duration)),
    seq(field('key', 'to'), field('value', $._expression)),
    seq(field('key', 'discretion'), field('value', $.identifier)),
    field('key', 'after_expiry'),
    seq(field('key', 'on'), field('value', $.call_expression)),
    seq(field('key', 'within'), field('value', $._expression)),
    seq(field('key', 'never'), field('value', $.cite)),
  )),

  // Links and instances.

  link_decl: $ => seq(
    'link',
    field('name', choice($.identifier, alias('_', $.anonymous))),
    field('label', $.string),
    ':',
    field('source', $.ref),
    '->',
    field('target', $.ref),
    repeat($._edge_clause),
    optional(field('fields', $.edge_fields)),
  ),

  instance_decl: $ => seq(
    'instance',
    field('base', $.identifier),
    '@',
    field('origin', $.identifier),
    braced($.instance_field),
  ),

  instance_field: $ => choice(
    seq(field('key', 'pack'), field('value', $.identifier)),
    seq(field('key', choice('note', 'payoff_transform')), field('value', $.string)),
    seq(field('key', 'remove_edges'), field('value', $.id_list)),
    seq(field('key', 'probabilities'), field('value', $.probability_map)),
    seq(field('key', 'roles'), field('value', $.role_map)),
    seq(field('key', 'stake'), field('value', choice($.signed_money, $.hole))),
    seq(field('key', 'patch'), field('edge', $.identifier), field('value', $.edge_fields)),
  ),

  probability_map: $ => braced($.probability_entry),

  probability_entry: $ => seq(field('key', $.identifier), ':', field('value', $._prob_value)),

  role_map: $ => braced($.role_entry),

  role_entry: $ => seq(field('key', $.identifier), ':', field('value', $.identifier)),

  // Rules, deadlines, counting, calendars.

  rule_decl: $ => seq(
    'rule',
    field('kind', choice('duty', 'power', 'liability')),
    field('name', $.identifier),
    repeat(choice($._version_clause, $.relation, $.cite)),
    field('body', $.rule_body),
  ),

  rule_body: $ => braced($._item),

  deadline_decl: $ => seq(
    'deadline',
    field('name', $.identifier),
    ':',
    field('length', $.duration),
    field('direction', choice('from', 'after', 'before')),
    field('trigger', $._expression),
    'counted_by',
    field('counting', $.ref),
    repeat(choice($._version_clause, $.relation, $.cite)),
    optional($.deadline_decl_opts),
  ),

  counting_decl: $ => seq(
    'counting',
    field('name', $.identifier),
    repeat(choice($._version_clause, $.relation, $.cite)),
    field('body', $.counting_body),
  ),

  counting_body: $ => braced($.counting_item),

  counting_item: $ => choice(
    field('key', choice('exclude_trigger_day', 'include_last_day', 'allow_hours')),
    seq(field('key', 'roll_forward_on'), field('value', $.id_list)),
    seq(field('key', 'service_extension'), field('value', $.service_map), optional($.cite)),
    seq(field('key', 'holidays'), field('value', $.ref)),
    seq(field('key', 'anchor'), field('value', $.identifier)),
    seq(field('key', 'end_of_month'), field('value', $.identifier)),
    seq(field('key', 'short_period'), field('value', $.short_period)),
  ),

  service_map: $ => braced($.service_entry),

  service_entry: $ => seq(field('method', $.identifier), ':', field('value', $.duration)),

  short_period: $ => seq(
    '{',
    'below', ':', field('below', $.duration), optional(','),
    'count', ':', field('count', $.identifier),
    '}',
  ),

  calendar_decl: $ => seq(
    'calendar',
    field('name', $.identifier),
    repeat(choice($._version_clause, $.cite)),
    field('body', $.calendar_body),
  ),

  calendar_body: $ => braced($.holiday_decl),

  holiday_decl: $ => seq(
    field('name', $.identifier),
    ':',
    field('date', $.call_expression),
    'effective',
    field('effective', $.interval),
    optional(seq('observed', field('observed', $.identifier))),
  ),

  // Versions, precedence, defeasibility.

  transition_decl: $ => seq(
    'transition',
    field('name', $.identifier),
    'amends',
    field('target', $.ref),
    repeat($.cite),
    field('body', $.transition_body),
  ),

  transition_body: $ => braced($.transition_arm),

  transition_arm: $ => choice(
    seq('unless', field('condition', $._expression), '->', field('version', $.ref)),
    seq('otherwise', '->', field('version', $.ref)),
  ),

  authority_decl: $ => seq(
    'authority',
    field('level', $.identifier),
    repeat(seq('>', field('level', $.identifier))),
  ),

  default_decl: $ => seq(
    'default',
    field('name', $.identifier),
    repeat(choice($._version_clause, $.relation, $.cite)),
    'when',
    field('condition', $._expression),
    '=>',
    field('value', $._expression),
  ),

  exception_decl: $ => seq(
    'exception',
    field('name', $.identifier),
    field('kind', choice('rebuts', 'undercuts')),
    field('target', $.ref),
    repeat($.cite),
    'when',
    field('condition', $._expression),
    optional(seq('=>', field('value', $._expression))),
    optional(seq('priority', field('priority', $.number))),
  ),

  test_decl: $ => seq(
    'test',
    field('name', $.string),
    optional($.cite),
    field('body', $.test_body),
  ),

  test_body: $ => seq('{', repeat(choice($.given_stmt, $.expect_stmt)), '}'),

  given_stmt: $ => seq('given', field('event', $._expression), 'on', field('date', $.date)),

  expect_stmt: $ => seq('expect', field('assertion', $._expression)),

  _version_clause: $ => choice($.effective_clause, $.known_at_clause),

  effective_clause: $ => seq('effective', $.interval),

  known_at_clause: $ => seq('known_at', $.date),

  interval: $ => seq(
    '[',
    field('from', $.date),
    ',',
    field('to', choice($.date, alias('..', $.open_end))),
    ')',
  ),

  relation: $ => choice(
    seq(
      field('kind', choice(
        'subject_to', 'notwithstanding', 'supersedes', 'revives', 'override',
      )),
      field('target', $.ref),
    ),
    seq(field('kind', 'delegable_to'), field('target', $.id_list)),
  ),

  // `cite ?("text")` is a cite hole: text with no citation-mode spelling.
  cite: $ => seq('cite', field('text', choice($.cite_text, $.hole))),
};
