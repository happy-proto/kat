module.exports = grammar({
  name: "debsources",

  extras: (_) => [/[\t ]+/],

  rules: {
    source_file: ($) => repeat(choice($.entry, $.comment, $._blank_line)),

    entry: ($) =>
      seq(
        field("kind", choice("deb", "deb-src")),
        optional(seq($._gap, field("options", $.options_block))),
        $._gap,
        field("uri", $.uri),
        $._gap,
        field("suite", $.suite),
        repeat(seq($._gap, field("component", $.component))),
        $._newline,
      ),

    options_block: ($) =>
      seq(
        "[",
        repeat(seq(field("option", $.option), optional($._gap))),
        "]",
      ),

    option: ($) => seq(field("name", $.option_name), "=", field("value", $.option_value)),

    comment: ($) => seq("#", /[^\n]*/, $._newline),

    option_name: (_) => /[A-Za-z0-9-]+/,
    option_value: (_) => /[^\]\s]+/,
    uri: (_) => /[A-Za-z][A-Za-z0-9+.-]*:\/\/[^\s]+/,
    suite: (_) => /[^\s\[]+/,
    component: (_) => /[^\s\[]+/,

    _gap: (_) => /[ \t]+/,
    _blank_line: ($) => $._newline,
    _newline: (_) => /\r?\n/,
  },
});
