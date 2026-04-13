// MIT License
// Copyright (c) 2025 Tom Waddington
// See LICENSE file for full license text

module.exports = grammar({
  name: 'applescript',

  externals: $ => [
    $.method_call_label,  // Objective-C method call: stringWithString:
    $.path_to_command,    // "path to" as atomic command
    $.handler_label,      // Objective-C handler label: doJava (when followed by :)
  ],

  extras: $ => [
    $.comment,
    /[\s\f\uFEFF\u2060\u200B]|\\\r?\n|¬\r?\n/,
  ],

  conflicts: $ => [
    // Repeat statements vs expression statements
    [$.repeat_times, $.expression_statement],
    [$.transaction_statement, $.expression_statement],

    // Application command ambiguities
    [$.application_command, $.expression_statement],
    [$.application_command],

    // List vs record (both start with '{')
    [$.list, $.record],

    // Parameter and expression ambiguities
    [$.direct_parameter, $.labeled_parameter, $.application_command, $.command_parameter, $.property_name, $.expression, $._identifier_or_keyword_token],
    [$.direct_parameter, $.labeled_parameter, $.application_command, $.command_parameter, $.property_name, $.expression],
    [$.direct_parameter, $.labeled_parameter, $.command_parameter, $.property_name, $.expression],
    [$.parameter_list, $.property_name, $.expression],

    // Handler call ambiguities (only for contexts with parentheses)
    [$.direct_parameter, $.labeled_parameter, $.command_parameter, $.property_name, $.expression, $.handler_call],
    [$.direct_parameter, $.labeled_parameter, $.application_command, $.command_parameter, $.property_name, $.expression, $.handler_call],
    [$.handler_call, $.application_command, $.direct_parameter, $.labeled_parameter, $.command_parameter, $.property_name, $.expression],
    [$.argument_list, $.parenthesized_expression],

    // With/using clause ambiguities (optional value)
    [$.with_clause],
    [$.using_clause, $.expression],

    // Element vs property access (both use 's syntax)
    [$.element_access, $.property_access],

    // Property name vs expression
    [$.property_name, $.expression],
    [$.property_name],
    [$.property_name, $._identifier_or_keyword_token, $.expression],
    [$.property_name, $._identifier_or_keyword_token],

    // Property names containing keywords
    [$._identifier_or_keyword_token, $.script_statement],

    // Additional parameter ambiguities (for system_attribute and other commands)
    [$.direct_parameter, $.labeled_parameter, $.command_parameter, $.property_name, $._identifier_or_keyword_token, $.expression],

    // Record multi-word key ambiguities
    [$.property_name, $.record_entry],

    // Error handling
    [$.on_error_clause, $.property_name, $.expression],

    // If statement
    [$.if_statement],
    [$.if_statement, $.else_if_clause],
  ],

  rules: {
    source_file: $ => repeat(choice(
      $.property_statement,
      $.handler_definition,
      $._statement,
    )),

    _statement: $ => choice(
      $.tell_statement,
      $.if_statement,
      $.repeat_statement,
      $.try_statement,
      $.set_statement,
      $.return_statement,
      $.exit_statement,
      $.error_statement,
      $.considering_statement,
      $.ignoring_statement,
      $.timeout_statement,
      $.transaction_statement,
      $.use_statement,
      $.script_statement,
      $.application_command,
      $.expression_statement,
    ),

    // Comments
    comment: $ => choice(
      seq('--', /.*/),
      seq('#', /.*/),
      seq('(*', repeat(choice(/[^*]/, /\*[^)]/)), '*)'),
    ),

    // Tell Statement
    tell_statement: $ => prec.right(choice(
      seq(
        'tell',
        field('target', $.expression),
        'to',
        field('statement', $._statement),
      ),
      seq(
        'tell',
        field('target', $.expression),
        repeat1($._statement),
        'end',
        optional('tell'),
      ),
      seq(
        'tell',
        field('target', $.expression),
      ),
    )),

    // If Statement
    if_statement: $ => seq(
      'if',
      field('condition', $.expression),
      'then',
      choice(
        $._statement,
        seq(
          repeat1($._statement),
          repeat($.else_if_clause),
          optional($.else_clause),
          'end',
          optional('if'),
        ),
      ),
    ),

    else_if_clause: $ => seq(
      'else',
      'if',
      field('condition', $.expression),
      'then',
      repeat1($._statement),
    ),

    else_clause: $ => seq(
      'else',
      repeat1($._statement),
    ),

    // Repeat Statement
    repeat_statement: $ => choice(
      $.repeat_forever,
      $.repeat_times,
      $.repeat_until,
      $.repeat_while,
      $.repeat_with,
      $.repeat_with_in,
    ),

    repeat_forever: $ => prec.right(seq(
      'repeat',
      repeat1($._statement),
      'end',
      optional('repeat'),
    )),

    repeat_times: $ => prec.right(seq(
      'repeat',
      field('count', $.expression),
      optional('times'),
      repeat1($._statement),
      'end',
      optional('repeat'),
    )),

    repeat_until: $ => prec.right(seq(
      'repeat',
      'until',
      field('condition', $.expression),
      repeat1($._statement),
      'end',
      optional('repeat'),
    )),

    repeat_while: $ => prec.right(seq(
      'repeat',
      'while',
      field('condition', $.expression),
      repeat1($._statement),
      'end',
      optional('repeat'),
    )),

    repeat_with: $ => prec.right(seq(
      'repeat',
      'with',
      field('variable', $.identifier),
      'from',
      field('start', $.expression),
      'to',
      field('end', $.expression),
      optional(seq('by', field('step', $.expression))),
      repeat1($._statement),
      'end',
      optional('repeat'),
    )),

    repeat_with_in: $ => prec.right(seq(
      'repeat',
      'with',
      field('variable', $.identifier),
      'in',
      field('list', $.expression),
      repeat1($._statement),
      'end',
      optional('repeat'),
    )),

    // Try Statement
    try_statement: $ => prec.right(seq(
      'try',
      repeat1($._statement),
      repeat($.on_error_clause),
      'end',
      optional('try'),
    )),

    on_error_clause: $ => seq(
      'on',
      'error',
      optional(field('message_var', $.identifier)),
      optional(seq('number', field('number_var', $.identifier))),
      optional(seq('from', field('from_var', $.identifier))),
      optional(seq('partial', 'result', field('partial_var', $.identifier))),
      optional(seq('to', field('to_var', $.identifier))),
      repeat1($._statement),
    ),

    // Set Statement
    set_statement: $ => seq(
      choice('set', 'copy'),
      field('target', $.expression),
      'to',
      field('value', $.expression),
    ),

    // Return Statement
    return_statement: $ => prec.right(seq(
      'return',
      optional(field('value', $.expression)),
    )),

    // Exit Statement
    exit_statement: $ => prec.right(seq(
      'exit',
      optional('repeat'),
    )),

    // Error Statement
    error_statement: $ => prec.right(seq(
      'error',
      optional(choice(
        seq(field('message', $.expression), repeat($.error_clause)),
        repeat1($.error_clause),
      )),
    )),

    error_clause: $ => choice(
      seq('number', field('number', $.expression)),
      seq('from', field('from', $.expression)),
      seq('partial', 'result', field('partial', $.expression)),
      seq('to', field('to', $.expression)),
    ),

    // Considering/Ignoring Statements
    considering_statement: $ => prec.right(seq(
      'considering',
      $.considering_terms,
      repeat1($._statement),
      'end',
      optional('considering'),
    )),

    ignoring_statement: $ => prec.right(seq(
      'ignoring',
      $.considering_terms,
      repeat1($._statement),
      'end',
      optional('ignoring'),
    )),

    considering_terms: $ => seq(
      $.considering_term,
      repeat(seq(optional(','), optional('and'), $.considering_term)),
    ),

    considering_term: $ => choice(
      'case',
      'diacriticals',
      'expansion',
      'hyphens',
      'punctuation',
      'white space',
      'numeric strings',
    ),

    // Timeout Statement
    timeout_statement: $ => seq(
      'with',
      'timeout',
      optional('of'),
      field('duration', $.expression),
      optional('seconds'),
      repeat1($._statement),
      'end',
      optional('timeout'),
    ),

    // Transaction Statement
    transaction_statement: $ => seq(
      'with',
      'transaction',
      optional(seq(optional('of'), field('session', $.expression))),
      repeat1($._statement),
      'end',
      optional('transaction'),
    ),

    // Use Statement
    use_statement: $ => seq(
      'use',
      choice(
        seq('scripting', 'additions'),
        seq('AppleScript', 'version', $.string),
        seq('framework', $.string),
        seq(
          optional(seq(field('alias', $.identifier), ':')),
          'script',
          $.string,
        ),
        seq(
          optional('application'),
          field('name', $.string),
          optional(seq('version', field('version', $.string))),
        ),
      ),
    ),

    // Script Statement
    script_statement: $ => prec.right(seq(
      'script',
      field('name', $.identifier),
      repeat(choice(
        $.property_statement,
        $.handler_definition,
        $._statement,
      )),
      'end',
      optional('script'),
    )),

    // Property Statement
    property_statement: $ => seq(
      'property',
      field('name', $.identifier),
      ':',
      field('value', $.expression),
    ),

    // Handler Definition
    handler_definition: $ => choice(
      $.simple_handler,
      $.positional_handler,
      $.labeled_handler,
    ),

    simple_handler: $ => prec.right(10, seq(
      'on',
      field('name', $.identifier),
      '(',
      optional($.parameter_list),
      ')',
      repeat1($._statement),
      'end',
      optional(field('end_name', $.identifier)),
    )),

    positional_handler: $ => prec.right(10, choice(
      // Regular positional handler
      seq(
        'on',
        field('name', $.identifier),
        optional($.direct_parameter),
        repeat($.labeled_parameter),
        repeat1($._statement),
        'end',
        optional(field('end_name', $.identifier)),
      ),
      // Objective-C style positional handler: on label:param label:param
      seq(
        'on',
        field('name', alias($.handler_label, $.identifier)),
        alias($.objc_first_parameter, $.labeled_parameter),
        repeat(alias($.objc_parameter, $.labeled_parameter)),
        repeat1($._statement),
        'end',
        optional(field('end_name', $.identifier)),
      ),
    )),

    labeled_handler: $ => prec.right(10, seq(
      'to',
      field('name', $.identifier),
      optional($.direct_parameter),
      repeat($.labeled_parameter),
      repeat1($._statement),
      'end',
      optional(field('end_name', $.identifier)),
    )),

    direct_parameter: $ => prec(1, field('parameter', choice(
      $.identifier,
      $.string,
      $.number,
      $.every_document,
    ))),

    labeled_parameter: $ => prec(2, choice(
      seq(
        field('label', $.identifier),
        field('parameter', choice($.identifier, $.expression)),
      ),
      seq(
        field('label', $.identifier),
        ':',
        field('parameter', choice($.identifier, $.expression)),
      ),
      // Objective-C style: handler_label appears as identifier in parse tree
      seq(
        field('label', alias($.handler_label, $.identifier)),
        ':',
        field('parameter', choice($.identifier, $.expression)),
      ),
    )),

    // Objective-C style first parameter (label is implicit from handler name)
    objc_first_parameter: $ => seq(
      ':',
      field('parameter', $.identifier),
    ),

    // Objective-C style subsequent parameters
    objc_parameter: $ => seq(
      field('label', alias($.handler_label, $.identifier)),
      ':',
      field('parameter', $.identifier),
    ),

    parameter_list: $ => seq(
      $.identifier,
      repeat(seq(',', $.identifier)),
    ),

    // Known multi-word commands (tokens for commands without keywords, sequences for those with keywords)
    system_attribute: $ => token(prec(10, seq('system', /\s+/, 'attribute'))),
    do_shell_script: $ => token(prec(10, seq('do', /\s+/, 'shell', /\s+/, 'script'))),
    do_javascript: $ => token(prec(10, seq('do', /\s+/, /[Jj]ava[Ss]cript/))),
    // path_to: Disabled for now due to conflicts with "POSIX path of" expressions
    write_text: $ => token(prec(10, seq('write', /\s+/, 'text'))),
    create_window: $ => token(prec(10, seq('create', /\s+/, 'window'))),
    create_tab: $ => token(prec(10, seq('create', /\s+/, 'tab'))),

    // Known multi-word property names (common Terminal.app and other app properties)
    current_session: $ => token(prec(10, seq('current', /\s+/, 'session'))),
    current_window: $ => token(prec(10, seq('current', /\s+/, 'window'))),
    current_tab: $ => token(prec(10, seq('current', /\s+/, 'tab'))),
    first_window: $ => token(prec(10, seq('first', /\s+/, 'window'))),
    first_process: $ => token(prec(10, seq('first', /\s+/, 'process'))),
    first_button: $ => token(prec(10, seq('first', /\s+/, 'button'))),
    first_document: $ => token(prec(10, seq('first', /\s+/, 'document'))),
    first_item: $ => token(prec(10, seq('first', /\s+/, 'item'))),
    front_document: $ => token(prec(10, seq('front', /\s+/, 'document'))),
    front_window: $ => token(prec(10, seq('front', /\s+/, 'window'))),
    selected_tab: $ => token(prec(10, seq('selected', /\s+/, 'tab'))),
    selected_window: $ => token(prec(10, seq('selected', /\s+/, 'window'))),
    used_script_library_files: $ => token(prec(100, seq('used', /\s+/, 'script', /\s+/, 'library', /\s+/, 'files'))),
    every_item: $ => token(prec(10, seq('every', /\s+/, 'item'))),
    every_window: $ => token(prec(10, seq('every', /\s+/, 'window'))),
    every_document: $ => token(prec(10, seq('every', /\s+/, 'document'))),
    // UI Scripting patterns
    scroll_area: $ => token(prec(10, seq('scroll', /\s+/, 'area'))),
    radio_group: $ => token(prec(10, seq('radio', /\s+/, 'group'))),
    radio_button: $ => token(prec(10, seq('radio', /\s+/, 'button'))),

    // Application Command
    application_command: $ => choice(
      // Known multi-word commands (highest precedence)
      prec.right(5, seq(
        field('command', choice(
          $.system_attribute,
          $.do_shell_script,
          $.do_javascript,
          // Note: path_to_command is only used in path_to_expression, not as a command
          $.write_text,
          $.create_window,
          $.create_tab,
        )),
        optional(repeat1(choice(
          $.direct_parameter,
          $.labeled_parameter,
          $.command_parameter,
          $.expression,
        ))),
        optional(choice(
          $.with_clause,
          $.without_clause,
          $.using_clause,
        )),
      )),
      // Generic command with parameters (medium precedence)
      prec.right(2, seq(
        field('command', choice(
          $.identifier,
          alias(seq($.identifier, repeat1($.identifier)), $.property_name),
        )),
        repeat1(choice(
          $.direct_parameter,
          $.labeled_parameter,
          $.command_parameter,
          $.expression,
        )),
        optional(choice(
          $.with_clause,
          $.without_clause,
          $.using_clause,
        )),
      )),
      // Multi-word command without parameters (lower precedence)
      prec.right(1, seq(
        field('command', alias(seq($.identifier, repeat1($.identifier)), $.property_name)),
        optional(choice(
          $.with_clause,
          $.without_clause,
          $.using_clause,
        )),
      )),
    ),

    command_parameter: $ => seq(
      field('label', $.identifier),
      field('value', choice($.identifier, $.expression)),
    ),

    with_clause: $ => seq(
      'with',
      field('option', $.property_name),
      optional(field('value', $.expression)),
    ),

    without_clause: $ => seq(
      'without',
      field('option', $.property_name),
    ),

    using_clause: $ => seq(
      'using',
      field('modifiers', choice(
        prec.dynamic(1, $.property_name),  // Prefer multi-word like "command down"
        $.expression,
      )),
    ),

    // Property name (can be multi-word)
    property_name: $ => choice(
      // Known multi-word property tokens (highest precedence)
      $.current_session,
      $.current_window,
      $.current_tab,
      $.first_window,
      $.first_process,
      $.first_button,
      $.first_document,
      $.first_item,
      $.front_document,
      $.front_window,
      $.selected_tab,
      $.selected_window,
      $.every_item,
      $.every_window,
      $.every_document,
      $.used_script_library_files,
      $.scroll_area,
      $.radio_group,
      $.radio_button,
      // Generic multi-word properties (any sequence of 2+ identifiers/keywords)
      // Use dynamic precedence to prefer this over keyword matches in expression contexts
      prec.dynamic(2, seq($._identifier_or_keyword_token, repeat1($._identifier_or_keyword_token))),
      // Single identifier (lowest precedence)
      prec.dynamic(-1, $.identifier),
    ),

    // Identifier or keyword used as identifier (for property names that contain keywords)
    _identifier_or_keyword_token: $ => choice(
      $.identifier,
      alias('script', $.identifier),
      alias('text', $.identifier),
      alias('library', $.identifier),
      alias('used', $.identifier),
      alias('source', $.identifier),
      alias('files', $.identifier),
      alias('line', $.identifier),
      alias('endings', $.identifier),
    ),

    // Expression Statement
    expression_statement: $ => $.expression,

    // Objective-C Method Call (AppleScriptObjC)
    method_call: $ => prec.right(16, seq(
      field('object', $.expression),
      '\'s',
      field('method', $.method_call_label),
      ':',
      field('argument', $.expression),
      // TODO: Support multiple labeled parameters
    )),

    // Path to expression (special case of command used as expression)
    path_to_expression: $ => prec(14, seq(
      $.path_to_command,
      choice(
        prec.dynamic(1, $.property_name),  // Prefer multi-word properties
        $.expression,                       // Fallback to any expression
      ),
    )),

    // Expressions
    expression: $ => choice(
      $.binary_expression,
      $.unary_expression,
      $.type_cast,  // Type cast with 'as' keyword
      $.parenthesized_expression,
      $.application_expression,
      $.path_to_expression,  // "path to" command in expression context
      $.method_call,  // Must be before property_access (higher precedence)
      $.handler_call,  // Must be before property_access (uses same 's syntax)
      $.property_access,
      $.element_access,
      $.list,
      $.record,
      $.property_name,  // Multi-word properties - no negative precedence
      $.identifier,
      $.string,
      $.number,
      $.boolean,
      $.missing_value,
      $.current_application,
      $.script_reference,
      $.reference_expression,
    ),

    binary_expression: $ => {
      const table = [
        [prec.left, 12, '^'],
        [prec.left, 11, choice('*', '/', 'div', 'mod', '÷')],
        [prec.left, 10, choice('+', '-', '&')],
        [prec.left, 7, choice('and', 'or')],
        [prec.left, 6, choice(
          '=', '≠', '>', '<', '≥', '≤', '>=', '<=',
          'equals', 'is equal to', 'is equal', 'equal to',
          'is not', 'is not equal to', 'is not equal', 'isn\'t', 'isn\'t equal to',
          'is greater than', 'greater than', 'comes after',
          'is less than', 'less than', 'comes before',
          'is greater than or equal to', 'is greater than or equal',
          'is less than or equal to', 'is less than or equal',
          'is',
        )],
        [prec.left, 5, choice(
          'contains', 'contain', 'does not contain', 'doesn\'t contain',
          'is in', 'is contained by',
          'is not in', 'is not contained by',
          'starts with', 'start with', 'begins with', 'begin with',
          'ends with', 'end with',
        )],
      ];

      return choice(...table.map(([fn, precedence, operator]) =>
        fn(precedence, seq(
          field('left', $.expression),
          field('operator', operator),
          field('right', $.expression),
        ))
      ));
    },

    // Type cast expression (e.g., "x as text", "(path to home) as alias")
    type_cast: $ => prec.left(4, seq(
      field('value', $.expression),
      'as',
      field('type', $.identifier),
    )),

    unary_expression: $ => choice(
      prec(13, seq('-', field('operand', $.expression))),
      prec(13, seq('+', field('operand', $.expression))),
      prec(8, seq('not', field('operand', $.expression))),
    ),

    parenthesized_expression: $ => seq(
      '(',
      choice(
        $.expression,
        $.application_command,  // Allow commands in parentheses (e.g., for command results)
      ),
      ')',
    ),

    application_expression: $ => prec(14, seq(
      'application',
      field('name', choice($.string, $.identifier)),
    )),

    property_access: $ => prec.left(15, seq(
      field('object', $.expression),
      '\'s',
      field('property', $.property_name),
    )),

    element_access: $ => prec.left(15, choice(
      // Possessive form: object's element [index]
      seq(
        field('object', $.expression),
        '\'s',
        field('element', $.property_name),
        optional(field('index', $.number)),
      ),
      // Of form: element [index] of object
      seq(
        field('element', $.property_name),
        choice(
          seq(field('index', $.number), 'of', field('object', $.expression)),
          seq('of', field('object', $.expression)),
          field('index', $.number),  // Allow implicit object (e.g., "window 1")
        ),
      ),
    )),

    handler_call: $ => prec.dynamic(5, choice(
      // With object: obj's handler(args)
      prec.left(16, seq(
        field('object', $.expression),
        '\'s',
        field('name', $.identifier),
        '(',
        optional($.argument_list),
        ')',
      )),
      // Without object: handler(args)
      prec(3, seq(
        field('name', $.identifier),
        '(',
        optional($.argument_list),
        ')',
      )),
    )),

    argument_list: $ => seq(
      $.expression,
      repeat(seq(',', $.expression)),
    ),

    reference_expression: $ => seq(
      choice('a ref to', 'a reference to', 'ref to', 'reference to'),
      field('target', $.expression),
    ),

    // Literals
    identifier: $ => choice(
      /[A-Za-z_][A-Za-z0-9_]*/,
      seq('|', /[^|]+/, '|'),
    ),

    string: $ => seq(
      '"',
      repeat(choice(
        /[^"\\]+/,
        /\\./,
      )),
      '"',
    ),

    number: $ => choice(
      $.integer,
      $.real,
    ),

    integer: $ => /[0-9]+/,

    real: $ => choice(
      /[0-9]+\.[0-9]+/,
      /[0-9]+(\.[0-9]+)?[eE][+-]?[0-9]+/,
    ),

    boolean: $ => choice('true', 'false', 'yes', 'no'),

    missing_value: $ => seq('missing', 'value'),

    current_application: $ => token(seq('current', /\s+/, 'application')),

    script_reference: $ => seq('script', $.string),

    list: $ => seq(
      '{',
      optional(seq(
        $.expression,
        repeat(seq(',', $.expression)),
        optional(','),
      )),
      '}',
    ),

    record: $ => seq(
      '{',
      optional(seq(
        $.record_entry,
        repeat(seq(',', $.record_entry)),
        optional(','),
      )),
      '}',
    ),

    record_entry: $ => prec.dynamic(3, seq(
      field('key', choice(
        prec.dynamic(1, $.identifier),  // Prefer plain identifier for single-word keys
        $.property_name,                // Use property_name for multi-word keys
        $.string
      )),
      ':',
      field('value', $.expression),
    )),
  },
});
