module.exports = grammar({
  name: "git_link",

  extras: () => [],

  rules: {
    source: ($) => repeat(choice($.directive_line, $.comment_line, $._newline)),

    directive_line: ($) =>
      seq(
        field("key", $.key),
        ":",
        optional($._space),
        field("value", $.path),
        $._line_ending
      ),

    comment_line: ($) => seq($.comment, $._line_ending),

    key: () => /[A-Za-z][A-Za-z-]*/,
    path: () => /[^\s\r\n#][^\r\n#]*/,
    comment: () => /#[^\n]*/,

    _space: () => /[ \t]+/,
    _newline: () => /\r?\n/,
    _eof: () => "\0",
    _line_ending: ($) => choice($._newline, $._eof),
  },
});
