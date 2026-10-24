/**
 * @file tree-sitter grammar for litigate `.lit` rule modules and `.case`
 * matters.
 *
 * One grammar, one root (`source_file`) with two entry rules: a file that
 * starts with `module` is a `lit_file` (SPEC §3.1), one that starts with
 * `case` is a `case_file` (SPEC §3.2), either one optionally led by an
 * `edition` marker (SPEC §2). Both share the expression, reference
 * and literal rules (§3.3), so editors register a single parser for both
 * extensions. The compiler's hand-written parser (`crates/litigate-syntax`)
 * is the source of truth; this grammar is an editor view of the same
 * language and the conformance test keeps the two in step.
 *
 * Keywords are contextual (SPEC §2): every keyword is extracted from the
 * `identifier` word token, so a keyword is only a keyword where the grammar
 * expects one. Citation text is an ordinary string, so the grammar needs no
 * external scanner.
 */

/* eslint-disable no-undef */

const lit = require('./grammar/lit');
const cas = require('./grammar/case');
const values = require('./grammar/values');

module.exports = grammar({
  name: 'litigate',

  word: $ => $.identifier,

  // Whitespace is exactly the compiler's (SPEC §2): a no-break or other
  // Unicode space is an invalid character (L0003), an ERROR here.
  extras: $ => [/[ \t\r\n]/, $.line_comment, $.doc_comment, $.block_comment],

  supertypes: $ => [$._item, $._expression],

  conflicts: $ => [[$.module_path], [$.edge_decl]],

  rules: {
    source_file: $ => seq(
      optional($.edition_decl),
      choice($.lit_file, $.case_file),
    ),

    edition_decl: $ => seq('edition', field('edition', $.number)),

    ...lit,
    ...cas,
    ...values,
  },
});
