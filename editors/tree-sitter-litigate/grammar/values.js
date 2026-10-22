/**
 * @file References, records, literals, expressions (SPEC §2, §3.1, §3.3).
 */

/* eslint-disable no-undef */

const { PREC, braced, commaList } = require('./helpers');

module.exports = {
  // ------------------------------------------------- shared value syntax

  ref: $ => seq(
    $.identifier,
    optional(seq('@', field('instance', $.identifier))),
    repeat(seq(choice('::', '.'), $.identifier)),
    optional(seq('@', field('version', $.date))),
  ),

  id_list: $ => commaList('[', $.ref, ']'),

  num_record: $ => braced($.num_entry),

  num_entry: $ => seq(field('key', $.identifier), ':', field('value', $.signed_number)),

  money_record: $ => braced($.money_entry),

  money_entry: $ => seq(field('key', $.identifier), ':', field('value', $.signed_money)),

  json_record: $ => braced($.json_entry),

  json_entry: $ => seq(
    field('key', choice($.identifier, $.string)),
    ':',
    field('value', $._json_value),
  ),

  _json_value: $ => choice(
    $.signed_number,
    $.string,
    $.boolean,
    alias('unset', $.unset),
    $.json_record,
    $.json_list,
  ),

  json_list: $ => commaList('[', $._json_value, ']'),

  signed_number: $ => seq(optional('-'), $.number),

  signed_money: $ => seq(optional('-'), $.money),

  hole: $ => prec.right(seq('?', optional(seq('(', field('reason', $.string), ')')))),

  // ---------------------------------------------------------- expressions

  // SPEC §3.3 levels: Expr > OrExpr > AndExpr > CmpExpr > AddExpr >
  // MulExpr > Unary > Postfix > Primary. `match` only heads an Expr, and
  // comparison is non-associative (`a < b < c` is an error), so each level
  // is its own hidden rule; all infix forms surface as `binary_expression`.
  _expression: $ => choice($.match_expression, $._or_level),

  match_expression: $ => seq(
    'match',
    field('scrutinee', $._or_level),
    '{',
    repeat(seq($.match_arm, optional(','))),
    '}',
  ),

  match_arm: $ => seq(
    field('pattern', choice($.ref, alias('_', $.wildcard))),
    '=>',
    field('value', $._expression),
  ),

  _or_level: $ => choice(alias($._logical, $.binary_expression), $._compare_level),

  _logical: $ => choice(
    prec.left(PREC.or, seq(
      field('left', $._or_level), field('operator', '||'), field('right', $._or_level),
    )),
    prec.left(PREC.and, seq(
      field('left', $._or_level), field('operator', '&&'), field('right', $._or_level),
    )),
  ),

  _compare_level: $ => choice(alias($._comparison, $.binary_expression), $._arith_level),

  _comparison: $ => prec(PREC.compare, seq(
    field('left', $._arith_level),
    field('operator', choice('==', '!=', '<', '<=', '>', '>=')),
    field('right', $._arith_level),
  )),

  _arith_level: $ => choice(alias($._arithmetic, $.binary_expression), $._unary_level),

  _arithmetic: $ => choice(
    prec.left(PREC.add, seq(
      field('left', $._arith_level),
      field('operator', choice('+', '-')),
      field('right', $._arith_level),
    )),
    prec.left(PREC.mul, seq(
      field('left', $._arith_level),
      field('operator', choice('*', '/', '%')),
      field('right', $._arith_level),
    )),
  ),

  _unary_level: $ => choice($.unary_expression, $.field_expression, $._primary),

  unary_expression: $ => seq(
    field('operator', choice('!', '-')),
    field('operand', $._unary_level),
  ),

  field_expression: $ => prec(PREC.postfix, seq(
    field('value', choice(
      $.call_expression,
      $.parenthesized_expression,
      $.record_expression,
      $.list_expression,
      $.field_expression,
    )),
    '.',
    field('field', $.identifier),
  )),

  _primary: $ => choice(
    $._literal,
    $.hole,
    $.cite,
    $.call_expression,
    $.ref,
    $.record_expression,
    $.list_expression,
    $.parenthesized_expression,
  ),

  call_expression: $ => seq(field('function', $.ref), field('arguments', $.arguments)),

  arguments: $ => commaList('(', $._expression, ')'),

  record_expression: $ => braced($.record_field),

  record_field: $ => seq(field('key', $.identifier), ':', field('value', $._expression)),

  list_expression: $ => commaList('[', $._expression, ']'),

  parenthesized_expression: $ => seq('(', $._expression, ')'),

  // ------------------------------------------------------------- tokens

  _literal: $ => choice(
    $.number,
    $.money,
    $.duration,
    $.string,
    $.date,
    $.boolean,
    alias('unit', $.unit),
  ),

  boolean: _ => choice('true', 'false'),

  // Digits = digit { digit | "_" }; malformed grouping is a compiler L0005,
  // not a tree-sitter error, so the editor view stays lenient here.
  number: _ => /\d[\d_]*(\.\d[\d_]*)?/,

  money: _ => /\d[\d_]*(\.\d[\d_]*)?\.usd/,

  duration: _ => /\d[\d_]*(\.\d[\d_]*)?\.(days|court_days|business_days|hours)/,

  date: _ => /\d{4}-\d{2}-\d{2}/,

  string: $ => seq(
    '"',
    repeat(choice($.string_content, $.escape_sequence)),
    token.immediate('"'),
  ),

  string_content: _ => token.immediate(prec(1, /[^"\\\n]+/)),

  escape_sequence: _ => token.immediate(/\\["\\nrt]/),

  // Digits then a letter also start an identifier (`12b-motion-filed`).
  identifier: _ => /([_\p{XID_Start}]|[0-9]+\p{XID_Start})\p{XID_Continue}*(-\p{XID_Continue}+)*/,

  line_comment: _ => token(seq('//', optional(seq(/[^/\n]/, /[^\n]*/)))),

  doc_comment: _ => token(seq('///', /[^\n]*/)),

  block_comment: _ => token(seq('/*', /[^*]*\*+([^/*][^*]*\*+)*/, '/')),
};
