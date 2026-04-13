; Adapted from zed's Go highlights with additional Go-specific refinements for
; Dracula-oriented terminal rendering in kat.
; Source: https://github.com/zed-industries/zed/blob/main/crates/grammars/src/go/highlights.scm
; License: MIT

(identifier) @variable

(package_identifier) @namespace

(type_identifier) @type

((type_identifier) @type.builtin
  (#any-of? @type.builtin
    "any"
    "bool"
    "byte"
    "comparable"
    "complex64"
    "complex128"
    "error"
    "float32"
    "float64"
    "int"
    "int8"
    "int16"
    "int32"
    "int64"
    "rune"
    "string"
    "uint"
    "uint8"
    "uint16"
    "uint32"
    "uint64"
    "uintptr"))

(type_spec
  name: (type_identifier) @type)

(type_parameter_declaration
  name: (identifier) @type)

(parameter_declaration
  name: (identifier) @variable.parameter)

(variadic_parameter_declaration
  name: (identifier) @variable.parameter)

(field_identifier) @property

(keyed_element
  .
  (literal_element
    (identifier) @property))

(call_expression
  function: (identifier) @function.call)

((call_expression
  function: (identifier) @function.builtin)
  (#any-of? @function.builtin
    "append"
    "cap"
    "clear"
    "close"
    "complex"
    "copy"
    "delete"
    "imag"
    "len"
    "make"
    "max"
    "min"
    "new"
    "panic"
    "print"
    "println"
    "real"
    "recover"))

(call_expression
  function: (selector_expression
    field: (field_identifier) @function.method.call))

(function_declaration
  name: (identifier) @function)

(method_declaration
  name: (field_identifier) @function.method)

(method_elem
  name: (field_identifier) @function.method)

(const_spec
  name: (identifier) @constant)

[
  (true)
  (false)
] @boolean

[
  (nil)
  (iota)
] @constant.builtin

[
  (interpreted_string_literal)
  (raw_string_literal)
  (rune_literal)
] @string

(escape_sequence) @string.escape

[
  (int_literal)
  (float_literal)
  (imaginary_literal)
] @number

(comment) @comment

((comment) @keyword.directive
  (#match? @keyword.directive "^//go:"))

((comment) @keyword.directive
  (#match? @keyword.directive "^// \\+build"))

[
  ";"
  "."
  ","
  ":"
] @punctuation.delimiter

[
  "("
  ")"
  "{"
  "}"
  "["
  "]"
] @punctuation.bracket

[
  "--"
  "-"
  "-="
  ":="
  "!"
  "!="
  "..."
  "*"
  "*="
  "/"
  "/="
  "&"
  "&&"
  "&="
  "%"
  "%="
  "^"
  "^="
  "+"
  "++"
  "+="
  "<-"
  "<"
  "<<"
  "<<="
  "<="
  "="
  "=="
  ">"
  ">="
  ">>"
  ">>="
  "|"
  "|="
  "||"
  "~"
] @operator

[
  "break"
  "case"
  "chan"
  "continue"
  "default"
  "defer"
  "else"
  "fallthrough"
  "for"
  "go"
  "goto"
  "if"
  "range"
  "return"
  "select"
  "switch"
] @keyword.control

[
  "const"
  "func"
  "import"
  "interface"
  "map"
  "package"
  "struct"
  "type"
  "var"
] @keyword.declaration
