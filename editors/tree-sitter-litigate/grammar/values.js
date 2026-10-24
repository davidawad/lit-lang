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
    $.invalid_date,
    $.boolean,
    alias('unit', $.unit),
  ),

  boolean: _ => choice('true', 'false'),

  // `_` only between two digits, as the compiler requires (L0005): `1__0`
  // and `1_` are not one number.
  number: _ => /\d+(_\d+)*(\.\d+(_\d+)*)?/,

  money: _ => /\d+(_\d+)*(\.\d+(_\d+)*)?\.usd/,

  duration: _ => /\d+(_\d+)*(\.\d+(_\d+)*)?\.(days|court_days|business_days|weeks|months|years|hours)/,

  // A valid proleptic Gregorian date (SPEC §2): months of 31 and 30 days,
  // February's 28, and the 29th in a leap year (divisible by 4 and not by
  // 100, or by 400).
  date: _ => token(prec(1, choice(
    /\d{4}-(0[13578]|1[02])-(0[1-9]|[12]\d|3[01])/,
    /\d{4}-(0[469]|11)-(0[1-9]|[12]\d|30)/,
    /\d{4}-02-(0[1-9]|1\d|2[0-8])/,
    /(\d\d(0[48]|[2468][048]|[13579][26])|([02468][048]|[13579][26])00)-02-29/,
  ))),

  // A date-shaped literal no calendar has (`2026-02-30`; the compiler's
  // L0004). Named so the editor view marks it instead of reading it as a
  // subtraction `2026 - 02 - 30`; elsewhere a date must be valid.
  invalid_date: _ => /\d{4}-\d{2}-\d{2}/,

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
