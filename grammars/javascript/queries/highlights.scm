; Adapted from zed's JavaScript highlights with adjustments for the current
; tree-sitter-javascript grammar revision used by kat.
; Source: https://github.com/zed-industries/zed/blob/main/crates/grammars/src/javascript/highlights.scm
; License: MIT

; Variables.
(identifier) @variable

(call_expression
  function: (member_expression
    object: (identifier) @type
    (#any-of? @type
      "Promise" "Array" "Object" "Map" "Set" "WeakMap" "WeakSet" "Date" "Error" "TypeError"
      "RangeError" "SyntaxError" "ReferenceError" "EvalError" "URIError" "RegExp" "Function"
      "Number" "String" "Boolean" "Symbol" "BigInt" "Proxy" "ArrayBuffer" "DataView")))

; Properties.
(property_identifier) @property
(shorthand_property_identifier) @property
(shorthand_property_identifier_pattern) @property
(private_property_identifier) @property

; Function and method calls.
(call_expression
  function: (identifier) @function.call)

(call_expression
  function: (member_expression
    property: [
      (property_identifier)
      (private_property_identifier)
    ] @function.method))

(new_expression
  constructor: (identifier) @type.class.call)

; Function and method definitions.
(function_expression
  name: (identifier) @function.definition)

(function_declaration
  name: (identifier) @function.definition)

(method_definition
  name: [
    (property_identifier)
    (private_property_identifier)
  ] @function.method)

(method_definition
  name: (property_identifier) @constructor
  (#eq? @constructor "constructor"))

(pair
  key: [
    (property_identifier)
    (private_property_identifier)
  ] @function.method
  value: [
    (function_expression)
    (arrow_function)
  ])

(assignment_expression
  left: (member_expression
    property: [
      (property_identifier)
      (private_property_identifier)
    ] @function.method)
  right: [
    (function_expression)
    (arrow_function)
  ])

(variable_declarator
  name: (identifier) @function.definition
  value: [
    (function_expression)
    (arrow_function)
  ])

(assignment_expression
  left: (identifier) @function.definition
  right: [
    (function_expression)
    (arrow_function)
  ])

; Parameters.
(formal_parameters
  [
    (identifier) @variable.parameter
    (array_pattern
      (identifier) @variable.parameter)
    (object_pattern
      [
        (pair_pattern
          value: (identifier) @variable.parameter)
        (shorthand_property_identifier_pattern) @variable.parameter
      ])
  ])

(arrow_function
  parameter: (identifier) @variable.parameter)

(catch_clause
  parameter: (identifier) @variable.parameter)

; Special identifiers and naming conventions.
(class_declaration
  name: (identifier) @type.class.definition)

(class_heritage
  (identifier) @type.class.inheritance)

((identifier) @constructor
  (#match? @constructor "^[A-Z]"))

([
  (identifier)
  (shorthand_property_identifier)
  (shorthand_property_identifier_pattern)
] @constant
  (#match? @constant "^_*[A-Z_][A-Z\\d_]*$"))

((identifier) @variable.builtin
  (#match? @variable.builtin "^(arguments|module|console|window|document|globalThis)$")
  (#is-not? local))

((identifier) @function.builtin
  (#eq? @function.builtin "require")
  (#is-not? local))

; Literals.
(this) @variable.special
(super) @variable.special

[
  (null)
  (undefined)
] @constant.builtin

[
  (true)
  (false)
] @boolean

(comment) @comment
(hash_bang_line) @comment

[
  (string)
  (template_string)
] @string

(escape_sequence) @string.escape

(regex) @string.regex
(regex_flags) @keyword.operator.regex

(number) @number

; Tokens.
[
  ";"
  (optional_chain)
  "."
  ","
  ":"
] @punctuation.delimiter

[
  "-"
  "--"
  "-="
  "+"
  "++"
  "+="
  "*"
  "*="
  "**"
  "**="
  "/"
  "/="
  "%"
  "%="
  "<"
  "<="
  "<<"
  "<<="
  "="
  "=="
  "==="
  "!"
  "!="
  "!=="
  "=>"
  ">"
  ">="
  ">>"
  ">>="
  ">>>"
  ">>>="
  "~"
  "^"
  "&"
  "|"
  "^="
  "&="
  "|="
  "&&"
  "||"
  "??"
  "&&="
  "||="
  "??="
  "..."
] @operator

(regex
  "/" @string.regex)

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

(ternary_expression
  [
    "?"
    ":"
  ] @operator)

[
  "as"
  "async"
  "debugger"
  "default"
  "delete"
  "extends"
  "get"
  "in"
  "instanceof"
  "new"
  "of"
  "set"
  "static"
  "target"
  "typeof"
  "void"
  "with"
] @keyword

[
  "const"
  "let"
  "var"
  "function"
  "class"
] @keyword.declaration

[
  "export"
  "from"
  "import"
] @keyword.import

[
  "await"
  "break"
  "case"
  "catch"
  "continue"
  "do"
  "else"
  "finally"
  "for"
  "if"
  "return"
  "switch"
  "throw"
  "try"
  "while"
  "yield"
] @keyword.control

(switch_default
  "default" @keyword.control)

(template_substitution
  "${" @punctuation.special
  "}" @punctuation.special) @embedded

(decorator
  "@" @punctuation.special)

(decorator
  [
    (identifier) @attribute
    (call_expression
      function: (identifier) @attribute)
    (member_expression
      property: (property_identifier) @attribute)
    (call_expression
      function: (member_expression
        property: (property_identifier) @attribute))
  ])

; JSX elements.
(jsx_opening_element
  (identifier) @tag.jsx
  (#match? @tag.jsx "^[a-z][^.]*$"))

(jsx_closing_element
  (identifier) @tag.jsx
  (#match? @tag.jsx "^[a-z][^.]*$"))

(jsx_self_closing_element
  (identifier) @tag.jsx
  (#match? @tag.jsx "^[a-z][^.]*$"))

(jsx_opening_element
  (identifier) @type.class.call
  (#match? @type.class.call "^[A-Z]"))

(jsx_closing_element
  (identifier) @type.class.call
  (#match? @type.class.call "^[A-Z]"))

(jsx_self_closing_element
  (identifier) @type.class.call
  (#match? @type.class.call "^[A-Z]"))

(jsx_attribute
  (property_identifier) @attribute.jsx)

(jsx_opening_element
  ([
    "<"
    ">"
  ]) @punctuation.bracket.jsx)

(jsx_closing_element
  ([
    "</"
    ">"
  ]) @punctuation.bracket.jsx)

(jsx_self_closing_element
  ([
    "<"
    "/>"
  ]) @punctuation.bracket.jsx)
