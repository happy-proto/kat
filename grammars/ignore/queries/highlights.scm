; Adapted from shunsambongi/tree-sitter-gitignore and tuned for kat.
; Source: https://github.com/shunsambongi/tree-sitter-gitignore
; License: MIT

(comment) @comment

(pattern_char) @string
(pattern_char_escaped) @string.escape
(bracket_char) @string
(bracket_char_escaped) @string.escape

[
  (wildcard_char_single)
  (wildcard_chars)
  (wildcard_chars_allow_slash)
  (bracket_expr)
  (bracket_range)
] @string.regex

(bracket_char_class) @constant.builtin

[
  (negation)
  (bracket_negation)
] @keyword.operator

[
  (directory_separator)
  (directory_separator_escaped)
] @punctuation.delimiter

[
  "["
  "]"
] @punctuation.bracket
