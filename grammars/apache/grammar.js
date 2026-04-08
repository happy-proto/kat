module.exports = grammar({
  name: "apache",

  extras: () => [],

  rules: {
    source: ($) =>
      repeat(choice($.block_start, $.block_end, $.directive, $.comment_line, $._newline)),

    block_start: ($) =>
      seq(
        optional($._space),
        "<",
        $.block_name,
        optional(seq($._space, $.arguments)),
        ">",
        optional(seq($._space, $.comment)),
        $._newline,
      ),

    block_end: ($) =>
      seq(
        optional($._space),
        "</",
        $.block_name,
        ">",
        optional(seq($._space, $.comment)),
        $._newline,
      ),

    directive: ($) =>
      seq(
        optional($._space),
        $.directive_name,
        repeat(seq($._space, $.argument)),
        optional(seq($._space, $.comment)),
        $._newline,
      ),

    comment_line: ($) => seq(optional($._space), $.comment, $._newline),

    arguments: ($) => repeat1(seq(optional($._space), $.argument)),
    argument: ($) => choice($.quoted_string, $.variable, $.number, $.bare_word),
    block_name: () => /[A-Za-z][A-Za-z0-9]*/,
    directive_name: () => /[A-Za-z][A-Za-z0-9]*/,
    quoted_string: () => /"([^"\\]|\\.)*"/,
    variable: () => /[%$][{(]?[A-Za-z0-9_.:-]+[})]?/,
    number: () => /[0-9]+/,
    bare_word: () => /[^\s#<>"][^\s#>]*/,
    comment: () => token(seq("#", /[^\n]*/)),

    _space: () => /[ \t]+/,
    _newline: () => /\r?\n/,
  },
});
