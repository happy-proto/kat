[
  (paragraph)
  (pipe_table)
] @text

(atx_heading
  (inline) @text.title)

(setext_heading
  (paragraph) @text.title)

[
  (atx_h1_marker)
  (atx_h2_marker)
  (atx_h3_marker)
  (atx_h4_marker)
  (atx_h5_marker)
  (atx_h6_marker)
  (setext_h1_underline)
  (setext_h2_underline)
] @text.title

[
  (link_title)
] @text.literal

(indented_code_block) @text.literal.block

(code_fence_content) @text.literal.block

(thematic_break) @punctuation.rule.markdown

[
  (fenced_code_block_delimiter)
  (info_string)
] @punctuation.delimiter.markdown

(info_string
  (language) @text.reference)

(link_destination) @text.uri

(link_label) @text.reference

(link_reference_definition
  (link_label) @text.reference
  (link_destination) @text.uri)

[
  (list_marker_plus)
  (list_marker_minus)
  (list_marker_star)
  (list_marker_dot)
  (list_marker_parenthesis)
  (task_list_marker_checked)
  (task_list_marker_unchecked)
] @punctuation.list.markdown

[
  (block_continuation)
  (block_quote_marker)
] @text.quote

(block_quote
  (paragraph) @text.quote)

[
  "|"
  (pipe_table_delimiter_cell)
  (pipe_table_align_left)
  (pipe_table_align_right)
] @punctuation.delimiter.markdown

(backslash_escape) @string.escape
