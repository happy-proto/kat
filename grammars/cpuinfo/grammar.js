module.exports = grammar({
  name: "cpuinfo",

  extras: (_) => [/[\t ]+/],

  rules: {
    source_file: ($) => repeat(choice($.entry, $._blank_line)),

    entry: ($) =>
      seq(
        field("key", $.key),
        ":",
        field("value", $.value),
        $._newline,
      ),

    key: (_) => /[A-Za-z0-9_().\/ -]+/,
    value: (_) => /[^\n]*/,

    _blank_line: ($) => $._newline,
    _newline: (_) => /\r?\n/,
  },
});
