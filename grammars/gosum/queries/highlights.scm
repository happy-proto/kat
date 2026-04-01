; Adapted from tree-sitter-go-sum and refined for kat's terminal rendering.
; Source: https://github.com/amaanq/tree-sitter-go-sum
; License: MIT

[
  "alpha"
  "beta"
  "dev"
  "pre"
  "rc"
  "+incompatible"
] @keyword

(module_path) @text.uri
(module_version) @string.special
(hash_version) @attribute
(hash) @string

[
  (number)
  (number_with_decimal)
  (hex_number)
] @number

(checksum
  "go.mod" @string.special)

[
  ":"
  "."
  "-"
  "/"
] @punctuation.delimiter
