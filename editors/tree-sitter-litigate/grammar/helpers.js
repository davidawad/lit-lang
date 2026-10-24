/**
 * @file Shared helpers for the litigate tree-sitter grammar.
 */

const PREC = {
  or: 1,
  and: 2,
  compare: 3,
  add: 4,
  mul: 5,
  unary: 6,
  postfix: 7,
};

/** `{ item Sep item ... }`: items juxtaposed, optionally comma-separated. */
const braced = (item) => seq('{', repeat(seq(item, optional(','))), '}');

/** `open a, b, c, close` with an optional trailing comma. */
const commaList = (open, item, close) =>
  seq(open, optional(seq(item, repeat(seq(',', item)), optional(','))), close);

/**
 * Keywords valid where an edge may start that may also name an edge
 * (`note "x" -> y`): the node-body field keywords, plus the edge-clause
 * keywords `by` and `p`, which a String after them disambiguates.
 */
const EDGE_ID_WORDS = [
  'group', 'note', 'valence', 'court_listener_url', 'tags', 'attrs', 'payoff',
  'payoff_by_flag', 'basis', 'by', 'p',
];

module.exports = { PREC, braced, commaList, EDGE_ID_WORDS };
