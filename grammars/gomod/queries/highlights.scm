; Adapted from tree-sitter-gomod and refined for kat's terminal rendering.
; Source: https://github.com/camdencheek/tree-sitter-go-mod
; License: MIT

[
  "module"
  "go"
  "toolchain"
  "require"
  "exclude"
  "replace"
  "retract"
] @keyword

(comment) @comment

"=>" @operator

[
  "("
  ")"
  "["
  "]"
] @punctuation.bracket

"," @punctuation.delimiter

[
  (module_path)
  (file_path)
] @text.uri

[
  (go_version)
  (version)
  (toolchain_name)
] @string.special

(escape_sequence) @string.escape

[
  (raw_string_literal)
  (interpreted_string_literal)
] @string
