/// <reference types="./node_modules/tree-sitter-cli/dsl.d.ts" />

export default grammar({
  name: "strace",
  rules: {
    source_file: ($) => repeat(seq(optional(choice($.pid, $.forkedPid)), choice($.line, $.signal, $.attach, $.exit, $.killSignal))),
    line: ($) =>
      seq(
        choice(
          $.syscall,
          seq("<...", $.syscall, "resumed>", optional("=>"), optional(","))
        ),
        $.parameters,
        optional(
          seq(
            "=",
            $.returnValue,
            $._newline),
        )),

    //215175 +++ killed by SIGINT +++
    killSignal: ($) => seq("+++", "killed", "by", $.value, "+++"),
    signal: ($) => seq("---", $.value, $.dict, "---"),
    attach: $ => seq("strace: Process ", $.pid, " attached"),
    pid: $ => $.integer,
    //[pid 143405]
    forkedPid: $ => seq(/\[pid\s*/, $.pid, "]"),
    syscall: () => /[a-z][a-z0-9_]*/,
    parameters: ($) =>
      seq(optional("("), repeat(seq($.parameter, optional(","),)), choice("<unfinished ...>", ")"), optional(")")),

    parameter: ($) =>
      seq(
        choice(
          $.string,
          $.list,
          $.values,
          $.dict,
          $.dictFn,
          seq(/[a-z_]+/, "=", choice("NULL", $.values)),
          "NULL",
        ),
        optional($.comment),
        optional("..."),
      ),
    integer: () => /-?\d+/,
    string: $ => seq(
      '"',
      repeat(choice(
        alias(token.immediate(prec(1, /[^\\"\n]+/)),
          $.string_content),
        $._escape_sequence,
      )),
      '"'),
    _escape_sequence: _ => token(prec(1, seq(
      '\\',
      choice(
        /[^xuU]/,
        /\d{2,3}/,
        /x[0-9a-fA-F]{2,}/,
        /u[0-9a-fA-F]{4}/,
        /U[0-9a-fA-F]{8}/,
      ),
    ))),

    list: ($) =>
      seq(
        optional('~'),
        "[",
        repeat(seq($.parameter, optional(","))),
        "]",
      ),
    pointer: () => /0x[0-9a-fA-F]+/,
    values: ($) => seq($.value, repeat(seq(choice("|", "*"), $.value))),
    value: ($) => choice(/[A-Z0-9_]+/, $.integer, $.pointer),
    comment: $ => seq(
      "/*",
      repeat(choice($.comment, /[^*]/)),
      "*/"
    ),
    _newline: () => /\r?\n/,

    returnValue: ($) =>
      seq(
        choice($.integer, $.pointer, "?"),
        optional($.errorName),
        optional($.errorDescription),
      ),
    errorName: () => /[A-Z]+/,
    errorDescription: ($) => seq(
      "(",
      repeat(choice(
        alias(token.immediate(prec(1, /[^\(\)]+/)),
          $.errorContent),
        seq("(", ")"),)),
      ")"
    ),

    dictFn: $ => seq($.dict, "=>", $.dict),
    dict: ($) => seq(
      "{",
      seq(
        $.dictElem,
        repeat(seq(",", $.dictElem))
      ),
      optional(seq(",", "...")),
      "}"),
    dictElem: ($) => choice(
      seq($.dictKey, "=", optional($._dictValue))
      ,
      seq($.macro, repeat1(seq("&&", $.macro)), "==", $.value),
    ),
    macro: () => seq(/[A-Z]+/, '(', /[a-z]+/, ')'),
    dictKey: () => /[a-z_][a-z_0-9]+/,
    _dictValue: ($) => choice(seq($.syscall, $.parameters), $.parameter),

    exit: ($) => seq("+++", "exited", "with", $.integer, "+++"),
  },
});
