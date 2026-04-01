module.exports = grammar({
  name: "userscript_metadata",

  extras: _ => [/[ \t\r\n]+/],

  rules: {
    source_file: $ => repeat(choice(
      $.start_marker,
      $.end_marker,
      $.entry,
      $.comment_line,
    )),

    start_marker: $ => prec(2, seq(
      field("prefix", $.comment_prefix),
      field("marker", $.start_delimiter),
    )),

    end_marker: $ => prec(2, seq(
      field("prefix", $.comment_prefix),
      field("marker", $.end_delimiter),
    )),

    entry: $ => prec(1, seq(
      field("prefix", $.comment_prefix),
      field("key", $.metadata_key),
      optional(field("value", $.metadata_value)),
    )),

    comment_line: $ => prec(-1, seq(
      field("prefix", $.comment_prefix),
      repeat(choice($.url, $.metadata_token)),
    )),

    comment_prefix: _ => "//",

    start_delimiter: _ => "==UserScript==",
    end_delimiter: _ => "==/UserScript==",

    metadata_key: $ => seq(
      "@",
      field("name", $.key_name),
      optional(field("locale", $.locale_suffix)),
    ),

    key_name: _ => /[A-Za-z][A-Za-z0-9-]*/,
    locale_suffix: _ => /:[A-Za-z0-9_-]+/,

    metadata_value: $ => repeat1(choice($.url, $.metadata_token)),

    url: _ => /[A-Za-z][A-Za-z0-9+.-]*:\/\/[^\s]+/,
    metadata_token: _ => /[^@\s][^\s]*/,
  },
});
