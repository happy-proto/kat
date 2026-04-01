; Adapted from tree-sitter-go-work and refined for kat's terminal rendering.
; Source: https://github.com/omertuc/tree-sitter-go-work
; License: MIT

[
  "go"
  "use"
  "replace"
] @keyword

(comment) @comment

"=>" @operator

[
  "("
  ")"
] @punctuation.bracket

[
  (module_path)
  (file_path)
] @text.uri

[
  (go_version)
  (version)
] @string.special

(escape_sequence) @string.escape

[
  (raw_string_literal)
  (interpreted_string_literal)
] @string
