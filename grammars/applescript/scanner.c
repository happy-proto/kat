// MIT License
// Copyright (c) 2025 Tom Waddington
// See LICENSE file for full license text

#include <tree_sitter/parser.h>
#include <wctype.h>
#include <string.h>
#include <stdio.h>

// External token types - must match order in grammar.js externals array
enum TokenType {
    METHOD_CALL_LABEL,  // Objective-C style method call: stringWithString:
    PATH_TO_COMMAND,    // "path to" as atomic command
    HANDLER_LABEL,      // Objective-C style handler label: doJava (when followed by :)
};

// Parser contexts for state tracking
typedef enum {
    CONTEXT_NORMAL,
    CONTEXT_PROPERTY_ACCESS,  // After "current application's" or similar
    CONTEXT_EXPRESSION,       // Inside expression (not handler definition)
} ParserContext;

// Scanner state
typedef struct {
    ParserContext context;
    bool in_property_chain;    // Tracking 's property access chains
    uint8_t property_depth;    // How deep in property access chain
} Scanner;

// Helper: skip whitespace and comments
static void skip_whitespace(TSLexer *lexer) {
    while (iswspace(lexer->lookahead)) {
        lexer->advance(lexer, true);
    }
}

// Helper: check if we're looking at a specific keyword
static bool check_keyword(TSLexer *lexer, const char *keyword) {
    size_t len = strlen(keyword);
    int32_t start_pos = lexer->get_column(lexer);

    for (size_t i = 0; i < len; i++) {
        if (lexer->lookahead != keyword[i]) {
            return false;
        }
        lexer->advance(lexer, false);
    }

    // Make sure it's a complete word (not part of longer identifier)
    if (iswalnum(lexer->lookahead) || lexer->lookahead == '_') {
        return false;
    }

    return true;
}

// Helper: check if looking at identifier
static bool is_identifier_start(int32_t c) {
    return iswalpha(c) || c == '_';
}

static bool is_identifier_char(int32_t c) {
    return iswalnum(c) || c == '_';
}

// Helper: consume an identifier and check if it matches
static bool consume_identifier(TSLexer *lexer, const char *expected) {
    if (!is_identifier_start(lexer->lookahead)) {
        return false;
    }

    const char *p = expected;
    while (*p && lexer->lookahead == *p) {
        lexer->advance(lexer, false);
        p++;
    }

    // If we didn't match the full expected string, fail
    if (*p != '\0') {
        return false;
    }

    // Make sure identifier ends (not part of longer identifier)
    if (is_identifier_char(lexer->lookahead)) {
        return false;
    }

    return true;
}

// Detect "path to" command
static bool scan_path_to_command(Scanner *scanner, TSLexer *lexer) {
    // Pattern match: "path" followed by "to"
    // The grammar will enforce this appears in appropriate context

    // Try to match "path" followed by "to"
    if (!consume_identifier(lexer, "path")) {
        return false;
    }

    // Skip whitespace between "path" and "to"
    skip_whitespace(lexer);

    // Check for "to" keyword
    if (!consume_identifier(lexer, "to")) {
        return false;
    }

    // Success - mark end of "path to" token
    lexer->mark_end(lexer);
    lexer->result_symbol = PATH_TO_COMMAND;
    return true;
}

// Detect Objective-C method call label (identifier:)
static bool scan_method_call_label(Scanner *scanner, TSLexer *lexer) {
    // Simple pattern matching: identifier followed by ':'
    // The grammar will enforce that this only appears in property access context

    // Consume identifier
    if (!is_identifier_start(lexer->lookahead)) {
        return false;
    }

    while (is_identifier_char(lexer->lookahead)) {
        lexer->advance(lexer, false);
    }

    // Mark end of identifier
    lexer->mark_end(lexer);

    // Skip whitespace
    skip_whitespace(lexer);

    // Check for colon
    if (lexer->lookahead != ':') {
        return false;
    }

    // Success - the identifier is a method label
    lexer->result_symbol = METHOD_CALL_LABEL;
    return true;
}

// Detect Objective-C handler label (identifier:)
static bool scan_handler_label(Scanner *scanner, TSLexer *lexer) {
    // Pattern matching: identifier followed by ':'
    // Used in handler definitions: on doJava:action onType:type
    // The grammar will enforce that this only appears in handler context

    // Consume identifier
    if (!is_identifier_start(lexer->lookahead)) {
        return false;
    }

    while (is_identifier_char(lexer->lookahead)) {
        lexer->advance(lexer, false);
    }

    // Mark end of identifier (before the colon)
    lexer->mark_end(lexer);

    // Skip whitespace
    skip_whitespace(lexer);

    // Check for colon
    if (lexer->lookahead != ':') {
        return false;
    }

    // Success - the identifier is a handler label
    lexer->result_symbol = HANDLER_LABEL;
    return true;
}

// Main scanner function
bool tree_sitter_applescript_external_scanner_scan(
    void *payload,
    TSLexer *lexer,
    const bool *valid_symbols
) {
    Scanner *scanner = (Scanner *)payload;

    // Skip whitespace
    skip_whitespace(lexer);

    // Try each token type in priority order

    // Priority 1: METHOD_CALL_LABEL (only in property access context)
    if (valid_symbols[METHOD_CALL_LABEL]) {
        if (scan_method_call_label(scanner, lexer)) {
            return true;
        }
    }

    // Priority 2: PATH_TO_COMMAND (in expression context)
    if (valid_symbols[PATH_TO_COMMAND]) {
        if (scan_path_to_command(scanner, lexer)) {
            return true;
        }
    }

    // Priority 3: HANDLER_LABEL (in handler definition context)
    if (valid_symbols[HANDLER_LABEL]) {
        if (scan_handler_label(scanner, lexer)) {
            return true;
        }
    }

    return false;
}

// Scanner lifecycle functions

void *tree_sitter_applescript_external_scanner_create() {
    Scanner *scanner = (Scanner *)calloc(1, sizeof(Scanner));
    scanner->context = CONTEXT_NORMAL;
    scanner->in_property_chain = false;
    scanner->property_depth = 0;
    return scanner;
}

void tree_sitter_applescript_external_scanner_destroy(void *payload) {
    Scanner *scanner = (Scanner *)payload;
    free(scanner);
}

unsigned tree_sitter_applescript_external_scanner_serialize(
    void *payload,
    char *buffer
) {
    Scanner *scanner = (Scanner *)payload;

    if (buffer == NULL) {
        return sizeof(Scanner);
    }

    memcpy(buffer, scanner, sizeof(Scanner));
    return sizeof(Scanner);
}

void tree_sitter_applescript_external_scanner_deserialize(
    void *payload,
    const char *buffer,
    unsigned length
) {
    Scanner *scanner = (Scanner *)payload;

    if (length == 0) {
        scanner->context = CONTEXT_NORMAL;
        scanner->in_property_chain = false;
        scanner->property_depth = 0;
        return;
    }

    memcpy(scanner, buffer, sizeof(Scanner));
}
