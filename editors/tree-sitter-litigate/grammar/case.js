/**
 * @file `.case` matter rules (SPEC §3.2).
 */

/* eslint-disable no-undef */

const { braced } = require('./helpers');

module.exports = {
  // ---------------------------------------------------------------- .case

  case_file: $ => seq(
    'case',
    field('name', $.identifier),
    repeat1($.uses_decl),
    $.as_of_decl,
    optional($.known_at_decl),
    $.summary_decl,
    optional($.notes_decl),
    repeat($.source_decl),
    optional($.case_start_decl),
    optional($.perspective_decl),
    optional($.parties_block),
    optional($.events_block),
    optional($.facts_block),
    optional($.stakes_block),
    optional($.beliefs_block),
    optional($.analysis_block),
  ),

  uses_decl: $ => seq('uses', $.use_item, repeat(seq(',', $.use_item))),

  use_item: $ => seq(field('package', $.identifier), field('version', $.version_req)),

  // `^1.2.0`, `~1.2.0`, `>=1.2.0`, `=1.2.0`: adjacent tokens, no trivia.
  version_req: _ => token(seq(
    choice('^', '~', '>=', '='),
    /\d+\.\d+\.\d+(-[0-9A-Za-z]+(\.[0-9A-Za-z]+)*)?/,
  )),

  as_of_decl: $ => seq('as_of', field('date', $.date)),

  known_at_decl: $ => seq('known_at', field('date', $.date)),

  summary_decl: $ => seq('summary', field('text', $.string)),

  notes_decl: $ => seq('notes', field('text', $.string)),

  source_decl: $ => seq(
    'source',
    field('title', $.string),
    optional(field('body', $.source_body)),
  ),

  source_body: $ => braced($.source_field),

  source_field: $ => choice(
    seq(field('key', 'url'), field('value', $.string)),
    seq(field('key', 'as_of'), field('value', choice($.date, $.hole))),
  ),

  case_start_decl: $ => seq('start', field('node', $.ref)),

  perspective_decl: $ => seq(
    'perspective',
    field('value', choice($.identifier, $.role_map)),
  ),

  parties_block: $ => seq('parties', braced($.party_decl)),

  party_decl: $ => seq(
    field('name', $.identifier),
    ':',
    field('actor', $.identifier),
    field('entity', $.identifier),
  ),

  events_block: $ => seq('events', '{', repeat($.case_event), '}'),

  case_event: $ => seq(
    optional(seq(field('id', $.identifier), ':')),
    field('date', $.date),
    optional(seq('docket', field('docket', $.number))),
    field('event', $.call_expression),
    field('role', choice('by', 'on')),
    field('party', $.identifier),
    optional($.cite),
  ),

  facts_block: $ => seq('facts', braced($.fact_entry)),

  fact_entry: $ => seq(field('node', $.ref), ':', field('edge', $.ref)),

  stakes_block: $ => seq('stakes', braced($.stake_entry)),

  stake_entry: $ => seq(
    field('node', $.ref),
    ':',
    choice(
      seq(
        field('value', choice($.signed_money, alias('unvalued', $.unvalued))),
        optional($.fee_shift),
      ),
      $.fee_shift,
    ),
  ),

  // This matter's fee-shift fraction at the terminal.
  fee_shift: $ => seq('fee_shift', field('fraction', $.number)),

  beliefs_block: $ => seq('beliefs', braced(choice($.prior, $.observation))),

  // A Dirichlet prior (Beta for two outcomes): pseudo-counts per edge.
  prior: $ => seq('prior', field('node', $.ref), ':', field('counts', $.count_map)),

  observation: $ => seq('observe', field('node', $.ref), ':', field('counts', $.count_map)),

  count_map: $ => braced($.count_entry),

  count_entry: $ => seq(field('edge', $.ref), ':', field('count', $.number)),

  analysis_block: $ => seq('analysis', field('value', $.record_expression)),
};
