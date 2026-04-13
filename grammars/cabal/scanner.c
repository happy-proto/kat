#include "tree_sitter/parser.h"

#include <wctype.h>

enum TokenType {
  SILLY,
  INDENT,
  DEDENT,
  INDENTED,
};

typedef struct {
  int *indent_lvls;
  int current_idx;
  int size;
} Deque;

Deque indent_lvls;

void push(Deque *deque, int value) {
  if (deque->current_idx == deque->size) {
    deque->size *= 2;
    deque->indent_lvls = realloc(deque->indent_lvls, deque->size * sizeof(int));
  }
  deque->indent_lvls[deque->current_idx++] = value;
}

int pop(Deque *deque) { return deque->indent_lvls[--deque->current_idx]; }

int peek(Deque *deque) { return deque->indent_lvls[deque->current_idx - 1]; }

void init_deque(Deque *deque) {
  deque->size = 8;
  deque->current_idx = 0;
  deque->indent_lvls = malloc(deque->size * sizeof(int));
}

void delete_deque(Deque *deque) { free(deque->indent_lvls); }

int pending_dedents;

void *tree_sitter_cabal_external_scanner_create() {
  init_deque(&indent_lvls);
  return NULL;
}

void tree_sitter_cabal_external_scanner_destroy(void *p) {
  delete_deque(&indent_lvls);
}

void tree_sitter_cabal_external_scanner_reset(void *p) {
  delete_deque(&indent_lvls);
}

unsigned tree_sitter_cabal_external_scanner_serialize(void *p, char *buffer) {
  return 0;
}

void tree_sitter_cabal_external_scanner_deserialize(void *p, const char *b,
                                                    unsigned n) {}

static void advance(TSLexer *lexer) { lexer->advance(lexer, false); }

static void skip(TSLexer *lexer) { lexer->advance(lexer, true); }

static bool scan_indent(TSLexer *lexer, bool const *valid_symbols) {
  if (valid_symbols[DEDENT] && pending_dedents > 0) {
    pending_dedents--;
    lexer->result_symbol = DEDENT;
    return true;
  } else if (valid_symbols[DEDENT] && lexer->eof(lexer)) {
    lexer->result_symbol = DEDENT;
    return true;
  } else if (lexer->eof(lexer)) {
    return false;
  } else if (lexer->lookahead != '\n') {
    return false;
  }

  lexer->advance(lexer, true);
  uint8_t indent = 0;
  while (true) {
    if (lexer->lookahead == ' ') {
      indent++;
      lexer->advance(lexer, true);
    } else if (lexer->lookahead == '\n') {
      indent = 0;
      lexer->advance(lexer, true);
    } else {
      break;
    }
  }

  int cur_indent_lvl = peek(&indent_lvls);
  if (valid_symbols[INDENT] && indent > cur_indent_lvl) {
    push(&indent_lvls, indent);
    lexer->result_symbol = INDENT;
    return true;
  } else if (valid_symbols[DEDENT] && indent < cur_indent_lvl) {
    while (indent < cur_indent_lvl) {
      pop(&indent_lvls);
      cur_indent_lvl = peek(&indent_lvls);
      pending_dedents++;
    }
    pending_dedents--;

    if (indent > cur_indent_lvl) {
      push(&indent_lvls, indent);
    }

    lexer->result_symbol = DEDENT;
    return true;
  } else if (valid_symbols[INDENTED] && indent > 0) {
    lexer->result_symbol = INDENTED;
    return true;
  } else {
    return false;
  }
}

bool tree_sitter_cabal_external_scanner_scan(void *payload, TSLexer *lexer,
                                             const bool *valid_symbols) {
  if (valid_symbols[INDENT] || valid_symbols[DEDENT] ||
      valid_symbols[INDENTED]) {
    return scan_indent(lexer, valid_symbols);
  }
  return false;
}
