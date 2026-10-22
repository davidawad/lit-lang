/**
 * @file tree-sitter grammar for litigate `.lit` rule modules and `.case`
 * matters.
 *
 * One grammar, one root (`source_file`) with two entry rules: a file that
 * starts with `module` is a `lit_file` (SPEC §3.1), one that starts with
 * `case` is a `case_file` (SPEC §3.2). Both share the expression, reference
 * and literal rules (§3.3), so editors register a single parser for both
 * extensions. The compiler's hand-written parser (`crates/litigate-syntax`)
 * is the source of truth; this grammar is an editor view of the same
 * language and the conformance test keeps the two in step.
 *
 * Keywords are contextual (SPEC §2): every keyword is extracted from the
 * `identifier` word token, so a keyword is only a keyword where the grammar
 * expects one. `cite` text is lexed by the external scanner (src/scanner.c).
 */

/* eslint-disable no-undef */

const lit = require('./grammar/lit');
const cas = require('./grammar/case');
const values = require('./grammar/values');

module.exports = grammar({
  name: 'litigate',

  word: $ => $.identifier,

  externals: $ => [$.cite_text, $._error_sentinel],

  extras: $ => [/\s/, $.line_comment, $.doc_comment, $.block_comment],

  supertypes: $ => [$._item, $._expression],

  conflicts: $ => [[$.module_path], [$.edge_decl]],

  rules: {
    source_file: $ => choice($.lit_file, $.case_file),

    ...lit,
    ...cas,
    ...values,
  },
});
