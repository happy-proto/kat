; Variables

(identifier) @variable

; Types

(class_declaration
  name: (identifier) @type)
(interface_declaration
  name: (identifier) @type)
(enum_declaration
  name: (identifier) @type)
(annotation_type_declaration
  name: (identifier) @type)
(record_declaration
  name: (identifier) @type)

[
  (boolean_type)
  (floating_point_type)
  (integral_type)
  (void_type)
] @type.builtin

; Functions and methods

(function_definition
  name: (identifier) @function)
(method_declaration
  name: (identifier) @function.method)
(method_invocation
  name: (identifier) @function.method)

; Constructors

(constructor_declaration
  name: (identifier) @constructor)

; Parameters

(formal_parameter
  name: (identifier) @variable.parameter)

; Modules and annotations

(package_declaration
  (identifier) @module)
(annotation
  name: (identifier) @attribute)

"@" @operator

; Constants and builtins

((identifier) @constant
 (#match? @constant "^_*[A-Z][A-Z\\d_]+$"))

[
  (true)
  (false)
  (null_literal)
] @constant.builtin

[
  (this)
  (super)
] @variable.builtin

; Literals

[
  (decimal_integer_literal)
  (decimal_floating_point_literal)
  (hex_integer_literal)
  (hex_floating_point_literal)
  (octal_integer_literal)
] @number

[
  (character_literal)
  (string_literal)
] @string

(escape_sequence) @string.escape

; Comments

[
  (line_comment)
  (block_comment)
] @comment

; Keywords

[
  "as"
  "break"
  "case"
  "catch"
  "class"
  "continue"
  "def"
  "default"
  "do"
  "else"
  "enum"
  "extends"
  "finally"
  "for"
  "if"
  "implements"
  "import"
  "in"
  "instanceof"
  "interface"
  "new"
  "package"
  "return"
  "switch"
  "throw"
  "throws"
  "try"
  "while"
  "yield"
] @keyword

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  ","
  ";"
] @punctuation.delimiter
