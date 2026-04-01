; Adapted from tree-sitter/tree-sitter-json and zed.
; Sources:
; - https://github.com/tree-sitter/tree-sitter-json
; - https://github.com/zed-industries/zed/blob/main/crates/grammars/src/json/highlights.scm
; License: MIT

(comment) @comment

(string) @string

(escape_sequence) @string.escape

(pair
  key: (string) @property.json_key)

(number) @number

[
  (true)
  (false)
] @boolean

(null) @constant.builtin

[
  ","
  ":"
] @punctuation.delimiter

[
  "{"
  "}"
  "["
  "]"
] @punctuation.bracket
