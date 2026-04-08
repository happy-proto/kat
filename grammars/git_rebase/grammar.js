module.exports = grammar({
  name: "git_rebase",

  extras: () => [],

  rules: {
    source: ($) => repeat(choice($.instruction_line, $.comment_line, $._newline)),

    instruction_line: ($) =>
      seq(
        optional($._space),
        $.command,
        optional(seq($._space, choice($.commit, $.label))),
        optional(seq($._space, $.subject)),
        optional(seq($._space, $.comment)),
        $._newline,
      ),

    comment_line: ($) => seq(optional($._space), $.comment, $._newline),

    command: () =>
      choice(
        "pick",
        "p",
        "reword",
        "r",
        "edit",
        "e",
        "squash",
        "s",
        "fixup",
        "f",
        "exec",
        "x",
        "break",
        "b",
        "drop",
        "d",
        "label",
        "l",
        "reset",
        "t",
        "merge",
        "m",
        "update-ref",
        "u",
      ),

    commit: () => /[0-9a-fA-F]{4,40}/,
    label: () => /[^\s#]+/,
    subject: () => /[^\n#]+/,
    comment: () => token(seq("#", /[^\n]*/)),

    _space: () => /[ \t]+/,
    _newline: () => /\r?\n/,
  },
});
