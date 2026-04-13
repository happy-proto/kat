module.exports = grammar({
  name: 'authorized_keys',

  extras: () => [
    /[ \t\r]/,
  ],

  rules: {
    source_file: $ => repeat(choice(
      $.entry,
      $.comment_line,
      /\n/,
    )),

    comment_line: $ => seq(
      '#',
      optional($.comment_text),
      optional('\n'),
    ),

    entry: $ => seq(
      optional(field('options', $.option_list)),
      field('key_type', $.key_type),
      field('key_blob', $.key_blob),
      optional(field('comment', $.entry_comment)),
      optional('\n'),
    ),

    option_list: $ => seq(
      $.option,
      repeat(seq(',', $.option)),
    ),

    option: $ => choice(
      seq(
        field('name', $.option_name),
        '=',
        field('value', $.option_value),
      ),
      field('name', $.option_name),
    ),

    option_name: _ => /[A-Za-z][A-Za-z0-9-]*/,
    option_value: $ => choice($.quoted_value, $.bare_value),
    quoted_value: _ => /"([^"\\]|\\.)*"/,
    bare_value: _ => /[^,\s#]+/,
    key_type: _ => /[A-Za-z0-9][A-Za-z0-9@._+-]*/,
    key_blob: _ => /[A-Za-z0-9+/=]+/,
    entry_comment: _ => /[^\n]+/,
    comment_text: _ => /[^\n]+/,
  },
});
