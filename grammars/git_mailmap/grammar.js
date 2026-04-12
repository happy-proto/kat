module.exports = grammar({
  name: "git_mailmap",

  extras: () => [],

  rules: {
    source: ($) => repeat(choice($.entry_line, $.comment_line, $._newline)),

    entry_line: ($) =>
      seq(
        field("canonical", $.mapping),
        optional(seq($._space, field("alias", $.mapping))),
        optional($._space),
        $._line_ending
      ),

    mapping: ($) =>
      seq(
        optional(seq(field("name", $.person_name), $._space)),
        field("email", $.email)
      ),

    comment_line: ($) => seq($.comment, $._line_ending),

    person_name: () => /(?:[^\s<#][^<\n]*[^\s<]|[^\s<#])/,
    email: () => /<[^>\n]+>/,
    comment: () => /#[^\n]*/,

    _space: () => /[ \t]+/,
    _newline: () => /\r?\n/,
    _eof: () => "\0",
    _line_ending: ($) => choice($._newline, $._eof),
  },
});
