; Adapted from tree-sitter/tree-sitter-rust and zed.
; Sources:
; - https://github.com/tree-sitter/tree-sitter-rust
; - https://github.com/zed-industries/zed/blob/main/crates/grammars/src/rust/highlights.scm
; License: MIT

(identifier) @variable

(metavariable) @variable

(type_identifier) @type

(fragment_specifier) @type

(primitive_type) @type.builtin

(self) @variable.special

(field_identifier) @property

(shorthand_field_identifier) @property

(trait_item
  name: (type_identifier) @type.interface)

(impl_item
  trait: (type_identifier) @type.interface)

(abstract_type
  trait: (type_identifier) @type.interface)

(dynamic_type
  trait: (type_identifier) @type.interface)

(trait_bounds
  (type_identifier) @type.interface)

(call_expression
  function: [
    (identifier) @function
    (scoped_identifier
      name: (identifier) @function)
    (field_expression
      field: (field_identifier) @function.method)
  ])

(generic_function
  function: [
    (identifier) @function
    (scoped_identifier
      name: (identifier) @function)
    (field_expression
      field: (field_identifier) @function.method)
  ])

(function_item
  name: (identifier) @function.definition)

(function_signature_item
  name: (identifier) @function.definition)

(macro_invocation
  macro: [
    (identifier) @function.special
    (scoped_identifier
      name: (identifier) @function.special)
  ])

(macro_invocation
  "!" @function.special)

(macro_definition
  name: (identifier) @function.special.definition)

((identifier) @type
 (#match? @type "^[A-Z]"))

((identifier) @constant
 (#match? @constant "^_*[A-Z][A-Z\\d_]*$"))

(enum_variant
  name: (identifier) @type)

[
  (line_comment)
  (block_comment)
] @comment

[
  (line_comment
    (doc_comment))
  (block_comment
    (doc_comment))
] @comment.doc

[
  "("
  ")"
  "{"
  "}"
  "["
  "]"
] @punctuation.bracket

(_
  .
  "<" @punctuation.bracket
  ">" @punctuation.bracket)

[
  "."
  ";"
  ","
  "::"
] @punctuation.delimiter

"#" @punctuation.special

[
  "as"
  "async"
  "const"
  "default"
  "dyn"
  "enum"
  "extern"
  "fn"
  "impl"
  "let"
  "macro_rules!"
  "mod"
  "move"
  "pub"
  "raw"
  "ref"
  "static"
  "struct"
  "for"
  "trait"
  "type"
  "union"
  "unsafe"
  "use"
  "where"
  (crate)
  (mutable_specifier)
  (super)
] @keyword

[
  "await"
  "break"
  "continue"
  "else"
  "if"
  "in"
  "loop"
  "match"
  "return"
  "while"
  "yield"
] @keyword.control

(for_expression
  "for" @keyword.control)

[
  (string_literal)
  (raw_string_literal)
  (char_literal)
] @string

(escape_sequence) @string.escape

[
  (integer_literal)
  (float_literal)
] @number

(boolean_literal) @boolean

[
  "!="
  "%"
  "%="
  "&"
  "&="
  "&&"
  "*"
  "*="
  "+"
  "+="
  "-"
  "-="
  "->"
  ".."
  "..="
  "..."
  "/="
  ":"
  "<<"
  "<<="
  "<"
  "<="
  "="
  "=="
  "=>"
  ">"
  ">="
  ">>"
  ">>="
  "@"
  "^"
  "^="
  "|"
  "|="
  "||"
  "?"
] @operator

(unary_expression
  "!" @operator)

operator: "/" @operator

(lifetime
  "'" @lifetime
  (identifier) @lifetime)

(parameter
  (identifier) @variable.parameter)

(attribute_item
  (attribute
    [
      (identifier) @attribute
      (scoped_identifier
        name: (identifier) @attribute)
      (token_tree
        (identifier) @attribute
        (#match? @attribute "^[a-z\\d_]*$"))
      (token_tree
        (identifier) @none
        "::"
        (#match? @none "^[a-z\\d_]*$"))
    ]))

(inner_attribute_item
  (attribute
    [
      (identifier) @attribute
      (scoped_identifier
        name: (identifier) @attribute)
      (token_tree
        (identifier) @attribute
        (#match? @attribute "^[a-z\\d_]*$"))
      (token_tree
        (identifier) @none
        "::"
        (#match? @none "^[a-z\\d_]*$"))
    ]))
