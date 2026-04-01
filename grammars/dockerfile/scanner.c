/* Adapted from camdencheek/tree-sitter-dockerfile.
 * Source: https://github.com/camdencheek/tree-sitter-dockerfile
 * License: MIT
 */
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>
#include <wctype.h>

#include "tree_sitter/parser.h"

#define MAX_HEREDOCS 10
#define DEL_SPACE 512

typedef struct {
    bool in_heredoc;
    bool stripping_heredoc;
    unsigned heredoc_count;
    char *heredocs[MAX_HEREDOCS];
} scanner_state;

enum TokenType {
    HEREDOC_MARKER,
    HEREDOC_LINE,
    HEREDOC_END,
    HEREDOC_NL,
    ERROR_SENTINEL,
};

void *tree_sitter_dockerfile_external_scanner_create() {
    scanner_state *state = malloc(sizeof(scanner_state));
    memset(state, 0, sizeof(scanner_state));
    return state;
}

void tree_sitter_dockerfile_external_scanner_destroy(void *payload) {
    if (!payload)
        return;

    scanner_state *state = payload;
    for (unsigned i = 0; i < MAX_HEREDOCS; i++) {
        if (state->heredocs[i]) {
            free(state->heredocs[i]);
        }
    }

    free(state);
}

unsigned tree_sitter_dockerfile_external_scanner_serialize(void *payload,
                                                           char *buffer) {
    scanner_state *state = payload;

    unsigned pos = 0;
    buffer[pos++] = state->in_heredoc;
    buffer[pos++] = state->stripping_heredoc;

    for (unsigned i = 0; i < state->heredoc_count; i++) {
        unsigned len = strlen(state->heredocs[i]) + 1;

        if (pos + len + 1 > TREE_SITTER_SERIALIZATION_BUFFER_SIZE) {
            break;
        }

        memcpy(&buffer[pos], state->heredocs[i], len);
        pos += len;
    }

    buffer[pos++] = 0;
    return pos;
}

void tree_sitter_dockerfile_external_scanner_deserialize(void *payload,
                                                         const char *buffer,
                                                         unsigned length) {
    scanner_state *state = payload;
    for (unsigned i = 0; i < state->heredoc_count; i++) {
        free(state->heredocs[i]);
        state->heredocs[i] = NULL;
    }

    if (length == 0) {
        state->in_heredoc = false;
        state->stripping_heredoc = false;
        state->heredoc_count = 0;
    } else {
        unsigned pos = 0;
        state->in_heredoc = buffer[pos++];
        state->stripping_heredoc = buffer[pos++];

        unsigned heredoc_count = 0;
        for (unsigned i = 0; i < MAX_HEREDOCS; i++) {
            unsigned len = strlen(&buffer[pos]);

            if (len == 0)
                break;

            len++;
            char *heredoc = malloc(len);
            memcpy(heredoc, &buffer[pos], len);
            state->heredocs[i] = heredoc;
            heredoc_count++;

            pos += len;
        }

        state->heredoc_count = heredoc_count;
    }
}

static void skip_whitespace(TSLexer *lexer) {
    while (lexer->lookahead != '\0' && lexer->lookahead != '\n' &&
           iswspace(lexer->lookahead))
        lexer->advance(lexer, true);
}

static bool scan_marker(scanner_state *state, TSLexer *lexer) {
    skip_whitespace(lexer);

    if (lexer->lookahead != '<')
        return false;
    lexer->advance(lexer, false);

    if (lexer->lookahead != '<')
        return false;
    lexer->advance(lexer, false);

    bool stripping = false;
    if (lexer->lookahead == '-') {
        stripping = true;
        lexer->advance(lexer, false);
    }

    int32_t quote = 0;
    if (lexer->lookahead == '"' || lexer->lookahead == '\'') {
        quote = lexer->lookahead;
        lexer->advance(lexer, false);
    }

    char delimiter[DEL_SPACE];
    unsigned del_idx = 1;

    while (lexer->lookahead != '\0' &&
           (quote ? lexer->lookahead != quote : !iswspace(lexer->lookahead))) {
        if (lexer->lookahead == '\\') {
            lexer->advance(lexer, false);

            if (lexer->lookahead == '\0') {
                return false;
            }
        }

        if (del_idx > 0) {
            delimiter[del_idx++] = lexer->lookahead;
        }
        lexer->advance(lexer, false);

        if (del_idx >= DEL_SPACE - 2) {
            del_idx = 0;
        }
    }

    if (quote) {
        if (lexer->lookahead != quote) {
            return false;
        }
        lexer->advance(lexer, false);
    }

    if (del_idx == 0) {
        lexer->result_symbol = HEREDOC_MARKER;
        return true;
    }

    delimiter[0] = stripping ? '-' : ' ';
    delimiter[del_idx] = '\0';

    char *del_copy = malloc(del_idx + 1);
    memcpy(del_copy, delimiter, del_idx + 1);

    if (state->heredoc_count == 0) {
        state->heredoc_count = 1;
        state->heredocs[0] = del_copy;
        state->stripping_heredoc = stripping;
    } else if (state->heredoc_count >= MAX_HEREDOCS) {
        free(del_copy);
    } else {
        state->heredocs[state->heredoc_count++] = del_copy;
    }

    lexer->result_symbol = HEREDOC_MARKER;
    return true;
}

static bool scan_content(scanner_state *state, TSLexer *lexer,
                         const bool *valid_symbols) {
    if (state->heredoc_count == 0) {
        state->in_heredoc = false;
        return false;
    }

    state->in_heredoc = true;

    if (state->stripping_heredoc) {
        skip_whitespace(lexer);
    }

    if (valid_symbols[HEREDOC_END]) {
        unsigned delim_idx = 1;
        while (state->heredocs[0][delim_idx] != '\0' &&
               lexer->lookahead != '\0' &&
               lexer->lookahead == state->heredocs[0][delim_idx]) {
            lexer->advance(lexer, false);
            delim_idx++;
        }

        if (state->heredocs[0][delim_idx] == '\0') {
            lexer->result_symbol = HEREDOC_END;

            free(state->heredocs[0]);

            for (unsigned i = 1; i < state->heredoc_count; i++) {
                state->heredocs[i - 1] = state->heredocs[i];
            }
            state->heredocs[state->heredoc_count - 1] = NULL;
            state->heredoc_count--;

            if (state->heredoc_count > 0) {
                state->stripping_heredoc = state->heredocs[0][0] == '-';
            } else {
                state->in_heredoc = false;
            }

            return true;
        }
    }

    if (!valid_symbols[HEREDOC_LINE])
        return false;

    lexer->result_symbol = HEREDOC_LINE;

    for (;;) {
        switch (lexer->lookahead) {
        case '\0':
            if (lexer->eof(lexer)) {
                state->in_heredoc = false;
                return true;
            }
            lexer->advance(lexer, false);
            break;

        case '\n':
            return true;

        default:
            lexer->advance(lexer, false);
        }
    }
}

bool tree_sitter_dockerfile_external_scanner_scan(void *payload, TSLexer *lexer,
                                                  const bool *valid_symbols) {
    scanner_state *state = payload;

    if (valid_symbols[ERROR_SENTINEL]) {
        if (state->in_heredoc) {
            return scan_content(state, lexer, valid_symbols);
        } else {
            return scan_marker(state, lexer);
        }
    }

    if (valid_symbols[HEREDOC_NL]) {
        if (state->heredoc_count > 0 && lexer->lookahead == '\n') {
            lexer->result_symbol = HEREDOC_NL;
            lexer->advance(lexer, false);
            return true;
        }
    }

    if (valid_symbols[HEREDOC_MARKER]) {
        return scan_marker(state, lexer);
    }

    if (valid_symbols[HEREDOC_LINE] || valid_symbols[HEREDOC_END]) {
        return scan_content(state, lexer, valid_symbols);
    }

    return false;
}
