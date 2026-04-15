module.exports = grammar({
  name: "vyper",

  extras: (_) => [],

  rules: {
    source_file: ($) =>
      repeat(
        choice(
          $.comment,
          $.decorator,
          $.variable_declaration,
          $.function_definition,
          $.assignment,
          $.return_statement,
          $.expression_line,
          $._blank_line,
        ),
      ),

    decorator: ($) =>
      seq(optional($._indent), "@", field("name", $.identifier), $._newline),

    variable_declaration: ($) =>
      seq(
        optional($._indent),
        field("name", $.identifier),
        ":",
        optional($._gap),
        field("value", $.line_value),
        $._newline,
      ),

    function_definition: ($) =>
      seq(
        optional($._indent),
        "def",
        $._gap,
        field("name", $.identifier),
        field("parameters", $.parameter_list),
        optional(seq($._gap, "->", $._gap, field("return_type", $.type_expr))),
        ":",
        $._newline,
      ),

    parameter_list: ($) =>
      seq(
        "(",
        optional(seq($.parameter, repeat(seq(",", $.parameter)), optional(","))),
        ")",
      ),

    parameter: ($) =>
      seq(
        field("name", $.identifier),
        optional(seq(":", optional($._gap), field("type", $.type_expr))),
      ),

    assignment: ($) =>
      seq(
        optional($._indent),
        field("target", $.dotted_identifier),
        optional($._gap),
        "=",
        optional($._gap),
        field("value", $.line_value),
        $._newline,
      ),

    return_statement: ($) =>
      seq(
        optional($._indent),
        "return",
        optional(seq($._gap, field("value", $.line_value))),
        $._newline,
      ),

    expression_line: ($) => seq(optional($._indent), field("value", $.line_value), $._newline),

    dotted_identifier: ($) => seq($.identifier, repeat(seq(".", $.identifier))),
    identifier: (_) => /[A-Za-z_][A-Za-z0-9_]*/,
    type_expr: (_) => /[A-Za-z_][A-Za-z0-9_\[\]]*/,
    line_value: (_) => /[^\n]+/,
    comment: ($) => seq(optional($._indent), "#", /[^\n]*/, $._newline),

    _indent: (_) => /[ \t]+/,
    _gap: (_) => /[ \t]+/,
    _blank_line: ($) => $._newline,
    _newline: (_) => /\r?\n/,
  },
});
