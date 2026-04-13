module.exports = grammar({
  name: "fstab",

  extras: (_) => [],

  rules: {
    source_file: ($) => repeat(choice($.entry, $.comment, $._blank_line)),

    entry: ($) =>
      seq(
        field("spec", $.field),
        $._gap,
        field("file", $.field),
        $._gap,
        field("vfstype", $.field),
        $._gap,
        field("mntops", $.field),
        optional(seq($._gap, field("freq", $.number_field), $._gap, field("passno", $.number_field))),
        $._newline,
      ),

    comment: ($) => seq("#", /[^\n]*/, $._newline),
    field: (_) => /[^\s#]+/,
    number_field: (_) => /[0-9]+/,

    _gap: (_) => /[ \t]+/,
    _blank_line: ($) => $._newline,
    _newline: (_) => /\r?\n/,
  },
});
