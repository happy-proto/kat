module.exports = grammar({
  name: "fortran_namelist",

  extras: (_) => [/[\t ]+/],

  rules: {
    source_file: ($) => repeat(choice($.group, $.comment, $._blank_line)),

    group: ($) =>
      seq(
        "&",
        field("name", $.identifier),
        $._newline,
        repeat(choice($.assignment, $.comment, $._blank_line)),
        "/",
        $._newline,
      ),

    assignment: ($) =>
      seq(
        field("name", $.identifier),
        "=",
        field("value", $.value_list),
        optional(","),
        $._newline,
      ),

    value_list: ($) => prec.right(seq($.value, repeat(seq(",", $.value)))),

    value: ($) =>
      choice($.string, $.number, $.boolean, $.identifier),

    comment: ($) => seq("!", /[^\n]*/, $._newline),
    identifier: (_) => /[A-Za-z_][A-Za-z0-9_]*/,
    string: (_) => /'[^'\n]*'|"[^"\n]*"/,
    number: (_) => /-?[0-9]+(\.[0-9]+)?/,
    boolean: (_) => choice(".true.", ".false.", ".TRUE.", ".FALSE."),

    _blank_line: ($) => $._newline,
    _newline: (_) => /\r?\n/,
  },
});
