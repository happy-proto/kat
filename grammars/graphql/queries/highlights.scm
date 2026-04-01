; Repository-local GraphQL highlights query.
; Based on the node structure from tree-sitter-graphql and tuned for kat.

[
  "query"
  "mutation"
  "subscription"
  "fragment"
  "on"
  "schema"
  "extend"
  "directive"
  "repeatable"
  "scalar"
  "type"
  "interface"
  "union"
  "enum"
  "input"
  "implements"
] @keyword

[
  (boolean_value)
  (null_value)
] @keyword

[
  "QUERY"
  "MUTATION"
  "SUBSCRIPTION"
  "FIELD"
  "FRAGMENT_DEFINITION"
  "FRAGMENT_SPREAD"
  "INLINE_FRAGMENT"
  "VARIABLE_DEFINITION"
  "SCHEMA"
  "SCALAR"
  "OBJECT"
  "FIELD_DEFINITION"
  "ARGUMENT_DEFINITION"
  "INTERFACE"
  "UNION"
  "ENUM"
  "ENUM_VALUE"
  "INPUT_OBJECT"
  "INPUT_FIELD_DEFINITION"
] @type.builtin

(comment) @comment

[
  (string_value)
] @string

[
  (int_value)
  (float_value)
] @number

(directive
  "@" @attribute
  (name) @attribute)

(directive_definition
  "@" @attribute
  (name) @attribute)

(variable
  "$" @operator
  (name) @variable.parameter)

(field
  (name) @property)

(argument
  (name) @property)

(object_field
  (name) @property)

(alias
  (name) @property)

(fragment_definition
  (fragment_name) @function.definition)

(fragment_spread
  (fragment_name) @function.call)

(named_type) @type

(enum_value) @constant

(operation_definition
  (name) @function.definition)

(field_definition
  (name) @function.definition)

(input_value_definition
  (name) @variable.parameter)

(type_condition
  (named_type) @type)

[
  "!"
  "&"
  "|"
  "="
] @operator

[
  "("
  ")"
  "{"
  "}"
  "["
  "]"
] @punctuation.bracket

[
  ":"
  "..."
] @punctuation.delimiter
