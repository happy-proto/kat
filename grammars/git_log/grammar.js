module.exports = grammar({
  name: "git_log",

  extras: () => [],

  conflicts: ($) => [[$.commit_entry], [$.diff_block], [$.diff_hunk]],

  rules: {
    source: ($) => repeat(choice($.commit_entry, $._blank_line)),

    commit_entry: ($) =>
      seq(
        $.commit_header,
        repeat(choice($.merge_line, $.identity_line, $.date_line)),
        repeat(choice($._blank_line, $.message_line)),
        repeat($.diff_block),
        repeat($._blank_line)
      ),

    commit_header: ($) =>
      seq(
        field("keyword", $.commit_keyword),
        $._space,
        field("commit", $.commit_hash),
        optional(seq($._space, field("refs", $.ref_list))),
        $._line_ending
      ),

    merge_line: ($) =>
      seq(
        field("key", $.merge_key),
        $._space,
        repeat1(seq($.commit_hash, optional($._space))),
        $._line_ending
      ),

    identity_line: ($) =>
      seq(
        field("key", $.identity_key),
        $._space,
        optional(seq(field("name", $.person_name), $._space)),
        field("email", $.email),
        $._line_ending
      ),

    date_line: ($) =>
      seq(
        field("key", $.date_key),
        $._space,
        field("value", $.metadata_value),
        $._line_ending
      ),

    message_line: ($) =>
      seq("    ", field("message", $.message_text), $._line_ending),

    diff_block: ($) =>
      seq(
        $.diff_header_line,
        $._line_ending,
        repeat(choice($.diff_metadata_line, $.diff_hunk, $._blank_line))
      ),

    diff_metadata_line: ($) =>
      seq(
        choice(
          $.diff_index_line,
          $.diff_from_line,
          $.diff_to_line,
          $.diff_mode_line,
          $.diff_similarity_line,
          $.diff_rename_line,
          $.diff_binary_line
        ),
        $._line_ending
      ),

    diff_hunk: ($) =>
      seq(
        $.diff_hunk_header_line,
        $._line_ending,
        repeat(
          choice(
            $.diff_added_line,
            $.diff_removed_line,
            $.diff_context_line,
            $.diff_no_newline_line,
            $._blank_line
          )
        )
      ),

    commit_keyword: () => "commit",
    merge_key: () => "Merge:",
    identity_key: () => choice("Author:", "Commit:"),
    date_key: () => choice("Date:", "AuthorDate:", "CommitDate:"),
    commit_hash: () => /[0-9a-fA-F]{7,40}/,
    ref_list: ($) =>
      seq(
        "(",
        optional(
          seq($.reference, repeat(seq(optional($._space), ",", optional($._space), $.reference)))
        ),
        ")"
      ),
    reference: () => /[^(),\r\n]+/,
    person_name: () => /(?:[^\s<#][^<\n]*[^\s<]|[^\s<#])/,
    email: () => /<[^>\n]+>/,
    metadata_value: () => /[^\r\n]+/,
    message_text: () => /[^\r\n]*/,

    diff_header_line: () => /diff --git [^\r\n]+/,
    diff_index_line: () => /index [^\r\n]+/,
    diff_from_line: () => /--- [^\r\n]+/,
    diff_to_line: () => /\+\+\+ [^\r\n]+/,
    diff_mode_line: () =>
      choice(
        /old mode [^\r\n]+/,
        /new mode [^\r\n]+/,
        /deleted file mode [^\r\n]+/,
        /new file mode [^\r\n]+/
      ),
    diff_similarity_line: () =>
      choice(
        /similarity index [^\r\n]+/,
        /dissimilarity index [^\r\n]+/
      ),
    diff_rename_line: () => choice(/rename from [^\r\n]+/, /rename to [^\r\n]+/),
    diff_binary_line: () => /Binary files [^\r\n]+/,
    diff_hunk_header_line: () => /@@[^\r\n]*@@(?: [^\r\n]+)?/,
    diff_added_line: () => /\+[^\r\n]*/,
    diff_removed_line: () => /-[^\r\n]*/,
    diff_context_line: () => / [^\r\n]*/,
    diff_no_newline_line: () => /\\ No newline at end of file/,

    _space: () => /[ \t]+/,
    _blank_line: () => /\r?\n/,
    _eof: () => "\0",
    _line_ending: ($) => choice($._blank_line, $._eof),
  },
});
