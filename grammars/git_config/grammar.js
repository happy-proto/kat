// reference: https://git-scm.com/docs/git-config#_syntax

const NEWLINE = /\r?\n/;
const WHITE_SPACE = /[ \t\f\v]/;
const ANYTHING = /[^\r\n]+/;

module.exports = grammar({
  name: "git_config",

  extras: ($) => [WHITE_SPACE],

  rules: {
    config: ($) => repeat(choice($.section, seq(optional($.comment), NEWLINE))),

    section: ($) => seq($.section_header, $._section_body),

    section_header: ($) =>
      seq("[", $.section_name, optional(seq('"', $.subsection_name, '"')), "]"),

    section_name: ($) => /[\w\.]+/,

    _section_body: ($) =>
      prec.right(sep1(NEWLINE, seq(optional($.variable), optional($.comment)))),

    subsection_name: ($) =>
      repeat1(choice(/[^\r\n\x00\"\\]+/, $.escape_sequence)),

    variable: ($) =>
      seq(choice(seq($.name, "=", field("value", $._value)), $.name)),

    name: ($) => /[a-zA-Z][\w\-]*/,

    _value: ($) => choice($._boolean, $.integer, $.string),

    _boolean: ($) =>
      choice(
        alias(choice($.true, "yes", "on"), $.true),
        alias(choice($.false, "no", "off"), $.false)
      ),

    true: ($) => "true",
    false: ($) => "false",

    integer: ($) => /\d+[kmgtpezyKMGTPEZY]?/,

    string: ($) => choice($._shell_command_string, repeat1($._string_fragment)),

    _shell_command_string: ($) =>
      seq(
        choice(
          seq($.shell_command, $._quoted_string),
          seq($.shell_command, $._unquoted_string),
          seq('"', $.shell_command, $._quoted_string_content, '"')
        ),
        repeat($._string_fragment)
      ),

    _string_fragment: ($) =>
      choice($._quoted_string, $._unquoted_string, $._line_continuation),

    _quoted_string: ($) => seq('"', optional($._quoted_string_content), '"'),

    _quoted_string_content: ($) =>
      repeat1(choice(/[^\"]/, $.escape_sequence, $._line_continuation)),

    _unquoted_string: ($) => choice(/[^\r\n;#" \t\f\v\\!][^\r\n;#"\\]*/, "!"),

    shell_command: ($) => prec(2, "!"),

    escape_sequence: ($) => /\\([btnfr"\\]|u[0-9a-fA-F]{4}|U[0-9a-fA-F]{8})/,

    _line_continuation: ($) => seq("\\", NEWLINE),

    comment: ($) => seq(/[#;]/, optional(ANYTHING)),
  },
});

function sep1(rule, separator) {
  return seq(rule, repeat(seq(separator, rule)));
}
