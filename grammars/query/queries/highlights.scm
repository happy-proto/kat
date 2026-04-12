; Repository-local Tree-sitter query highlights for kat.
; Parser source: https://github.com/nvim-treesitter/tree-sitter-query

(comment) @comment

(string) @string
(escape_sequence) @string.escape

(quantifier) @operator

[
  "("
  ")"
  "["
  "]"
] @punctuation.bracket

[
  "."
  ":"
] @punctuation.delimiter

(capture) @function.special

(named_node
  name: (identifier) @type)

(anonymous_node
  name: (identifier) @string)

(field_definition
  name: (identifier) @property)

(predicate
  name: (identifier) @function.builtin)

(predicate_type) @function.builtin

[
  (named_node name: "_")
  (anonymous_node name: "_")
] @constant.builtin
