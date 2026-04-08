module.exports = grammar({
  name: "requirements",

  extras: () => [],

  rules: {
    source: ($) =>
      repeat(choice($.requirement_line, $.option_line, $.comment_line, $._newline)),

    requirement_line: ($) =>
      seq(
        optional($._space),
        optional(seq($.editable_flag, $._space)),
        $.package,
        optional($.extras),
        repeat($.version_clause),
        optional($.direct_reference),
        optional($.marker),
        optional(seq($._space, $.comment)),
        $._newline,
      ),

    option_line: ($) =>
      seq(
        optional($._space),
        $.option,
        optional(seq($._space, $.value)),
        optional(seq($._space, $.comment)),
        $._newline,
      ),

    comment_line: ($) => seq(optional($._space), $.comment, $._newline),

    extras: ($) =>
      seq("[", $.extra, repeat(seq(",", $.extra)), optional(","), "]"),

    version_clause: ($) =>
      seq(optional($._space), $.version_operator, optional($._space), $.version),

    direct_reference: ($) =>
      seq(optional($._space), "@", optional($._space), $.url),

    marker: ($) =>
      seq(optional($._space), ";", optional($._space), $.marker_expression),

    editable_flag: () => "-e",
    option: () =>
      choice(
        "-r",
        "--requirement",
        "-c",
        "--constraint",
        "--index-url",
        "--extra-index-url",
        "--find-links",
        "--trusted-host",
        "--pre",
      ),
    package: () => /[A-Za-z0-9][A-Za-z0-9._-]*/,
    extra: () => /[A-Za-z0-9._-]+/,
    version_operator: () => choice("===", "~=", "==", "!=", "<=", ">=", "<", ">"),
    version: () => /[A-Za-z0-9.*+!_-]+/,
    url: () => /[^\s#;]+/,
    value: () => /[^\n#]+/,
    marker_expression: () => /[^\n#]+/,
    comment: () => token(seq("#", /[^\n]*/)),

    _space: () => /[ \t]+/,
    _newline: () => /\r?\n/,
  },
});
