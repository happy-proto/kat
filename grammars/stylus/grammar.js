module.exports = grammar({
  name: "stylus",

  extras: (_) => [],

  rules: {
    source_file: ($) =>
      repeat(
        choice(
          $.comment,
          $.at_rule,
          $.variable_assignment,
          $.property_line,
          $.selector_line,
          $._blank_line,
        ),
      ),

    comment: ($) => seq("//", /[^\n]*/, $._newline),

    at_rule: ($) =>
      seq(
        optional($._indent),
        field("keyword", $.at_keyword),
        optional(seq($._gap, field("value", $.line_value))),
        $._newline,
      ),

    variable_assignment: ($) =>
      seq(
        optional($._indent),
        field("name", choice($.css_variable, $.identifier)),
        optional($._gap),
        "=",
        optional($._gap),
        field("value", $.line_value),
        $._newline,
      ),

    property_line: ($) =>
      seq(
        optional($._indent),
        field("name", $.property_name),
        $._gap,
        field("value", $.line_value),
        $._newline,
      ),

    selector_line: ($) => seq(optional($._indent), field("selector", $.selector), $._newline),

    at_keyword: (_) => /@[A-Za-z-]+/,
    css_variable: (_) => /--[A-Za-z0-9_-]+/,
    identifier: (_) => /[$A-Za-z_][A-Za-z0-9_-]*/,
    property_name: (_) => /-?[A-Za-z][A-Za-z0-9_-]*/,
    selector: (_) => /[#.&A-Za-z][^\n=]*/,
    line_value: (_) => /[^\n]+/,

    _indent: (_) => /[ \t]+/,
    _gap: (_) => /[ \t]+/,
    _blank_line: ($) => $._newline,
    _newline: (_) => /\r?\n/,
  },
});
