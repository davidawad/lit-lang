// External scanner for litigate: the citation text after `cite` (SPEC §2).
//
// After the reserved word `cite` the compiler's lexer switches to citation
// mode: it skips spaces and tabs, then consumes text up to, but not
// including, the first newline, `{`, `}`, `,`, or `//`. Trailing whitespace
// is not part of the citation. A regex cannot express "stop before `//` but
// not before `/`" together with trailing-whitespace trimming, so this
// scanner does it by hand.

#include "tree_sitter/parser.h"

enum TokenType {
  CITE_TEXT,
  ERROR_SENTINEL,
};

void *tree_sitter_litigate_external_scanner_create(void) { return NULL; }

void tree_sitter_litigate_external_scanner_destroy(void *payload) { (void)payload; }

unsigned tree_sitter_litigate_external_scanner_serialize(void *payload, char *buffer) {
  (void)payload;
  (void)buffer;
  return 0;
}

void tree_sitter_litigate_external_scanner_deserialize(void *payload, const char *buffer,
                                                       unsigned length) {
  (void)payload;
  (void)buffer;
  (void)length;
}

static bool is_blank(int32_t c) { return c == ' ' || c == '\t' || c == '\r'; }

static bool ends_cite(int32_t c) { return c == '\n' || c == '{' || c == '}' || c == ','; }

bool tree_sitter_litigate_external_scanner_scan(void *payload, TSLexer *lexer,
                                               const bool *valid_symbols) {
  (void)payload;
  // During error recovery every symbol is valid; never invent a citation.
  if (valid_symbols[ERROR_SENTINEL] || !valid_symbols[CITE_TEXT]) {
    return false;
  }
  // Citation mode skips spaces and tabs only: `cite` must not be followed by
  // a line break before its text (an empty citation is L0001).
  while (lexer->lookahead == ' ' || lexer->lookahead == '\t') {
    lexer->advance(lexer, true);
  }
  // `cite ?("text")` is a cite hole, lexed by the grammar (SPEC §2).
  if (lexer->lookahead == '?') {
    return false;
  }
  bool has_text = false;
  while (!lexer->eof(lexer) && !ends_cite(lexer->lookahead)) {
    if (lexer->lookahead == '/') {
      lexer->advance(lexer, false);
      if (lexer->lookahead == '/') {
        break;
      }
      has_text = true;
      lexer->mark_end(lexer);
      continue;
    }
    bool blank = is_blank(lexer->lookahead);
    lexer->advance(lexer, false);
    if (!blank) {
      has_text = true;
      lexer->mark_end(lexer);
    }
  }
  if (!has_text) {
    return false;
  }
  lexer->result_symbol = CITE_TEXT;
  return true;
}
