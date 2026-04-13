module.exports = grammar({
  name: "cmakecache",

  extras: (_) => [/[\t ]+/],

  rules: {
    source_file: ($) => repeat(choice($.entry, $.comment, $._blank_line)),

    entry: ($) =>
      seq(
        field("name", $.name),
        ":",
        field("kind", $.cache_type),
        "=",
        field("value", $.value),
        $._newline,
      ),

    comment: ($) => seq(choice("//", "#"), /[^\n]*/, $._newline),

    name: (_) => /[A-Za-z0-9_+./-]+/,
    cache_type: (_) => /[A-Z_]+/,
    value: (_) => /[^\n]*/,

    _blank_line: ($) => $._newline,
    _newline: (_) => /\r?\n/,
  },
});
