(line_comment) @comment
(block_comment) @comment

(annotation
  (identifier) @attribute)

(class_declaration
  name: (identifier) @type)

(object_declaration
  name: (identifier) @type)

(function_declaration
  name: (identifier) @function)

(parameter
  (identifier) @variable.parameter)

(variable_declaration
  (identifier) @variable)

(user_type
  (identifier) @type)

[
  (string_literal)
  (multiline_string_literal)
] @string

(escape_sequence) @string.escape

[
  (number_literal)
  (float_literal)
  (character_literal)
] @number

(identifier) @variable
