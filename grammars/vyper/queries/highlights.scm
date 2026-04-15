(comment) @comment

(decorator
  name: (identifier) @attribute.builtin)

(function_definition
  "def" @keyword
  name: (identifier) @function)

(function_definition
  return_type: (type_expr) @type)

(parameter
  name: (identifier) @variable.parameter)

(parameter
  type: (type_expr) @type)

(variable_declaration
  name: (identifier) @property)

(variable_declaration
  value: (line_value) @string)

(assignment
  target: (dotted_identifier) @variable)

(assignment
  value: (line_value) @string)

(return_statement
  "return" @keyword)

(return_statement
  value: (line_value) @string)

(type_expr) @type

"@" @punctuation.special
":" @punctuation.delimiter
"=" @operator
"->" @operator
