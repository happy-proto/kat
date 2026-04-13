module.exports = grammar({
  name: "command_help",

  extras: (_) => [/[\t ]+/],

  rules: {
    source_file: ($) =>
      repeat(
        choice(
          $.usage_line,
          $.section_heading,
          $.option_line,
          $.text_line,
          $._blank_line,
        ),
      ),

    usage_line: ($) => seq(field("usage", /[Uu]sage:/), field("body", /[^\n]*/), $._newline),

    section_heading: ($) =>
      seq(field("title", /[A-Z][A-Za-z0-9() \\/_-]*:/), $._newline),

    option_line: ($) =>
      seq(
        optional($._indent),
        field("flags", $.flag_block),
        optional(seq($._gap, field("description", /[^\n]*/))),
        $._newline,
      ),

    text_line: ($) => seq(/[^\n]+/, $._newline),

    flag_block: (_) => /-{1,2}[^\n]*/,

    _indent: (_) => /[ \t]+/,
    _gap: (_) => /[ \t]{2,}/,
    _blank_line: ($) => $._newline,
    _newline: (_) => /\r?\n/,
  },
});
