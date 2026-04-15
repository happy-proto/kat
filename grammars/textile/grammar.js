module.exports = grammar({
  name: "textile",

  extras: (_) => [],

  rules: {
    source_file: ($) =>
      repeat(choice($.heading, $.code_block, $.list_item, $.paragraph, $._blank_line)),

    heading: ($) =>
      seq(
        field("marker", choice("h1.", "h2.", "h3.", "h4.", "h5.", "h6.")),
        /[ \t]+/,
        field("content", /[^\n]*/),
        $._newline,
      ),

    code_block: ($) =>
      seq(
        field("marker", choice("bc.", "pre.")),
        /[ \t]+/,
        field("content", /[^\n]*/),
        $._newline,
      ),

    list_item: ($) =>
      seq(
        field("marker", choice(/\*+/, /#+/)),
        /[ \t]+/,
        field("content", /[^\n]*/),
        $._newline,
      ),

    paragraph: ($) => seq(/[^\n]+/, $._newline),

    _blank_line: ($) => $._newline,
    _newline: (_) => /\r?\n/,
  },
});
