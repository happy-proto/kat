;; Comments
(comment) @comment

;; Strings
(string_literal) @string
(double_quoted_string) @string.special.path

;; Keywords and operators
[
  "begin"
  "end"
  "if"
  "else"
  "case"
  "endcase"
  "for"
  "while"
  "repeat"
  "assign"
  "input"
  "output"
  "inout"
  "parameter"
  "localparam"
  "function"
  "endfunction"
  "task"
  "endtask"
  "posedge"
  "negedge"
  (always_keyword)
  (module_keyword)
] @keyword

[
  "wire"
  "reg"
] @type.builtin

[
  "="
  "<="
  "+"
  "-"
  "*"
  "/"
  "!"
  "&&"
  "||"
  "&"
  "|"
] @operator

[
  ";"
  ","
  ":"
  "."
] @punctuation.delimiter

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

;; Declarations
(module_header
  (simple_identifier) @module)

(function_body_declaration
  (function_identifier) @function)

(task_body_declaration
  (task_identifier) @function)

(module_instantiation
  (simple_identifier) @type)

(name_of_instance
  (instance_identifier) @variable)

;; Numbers
[
  (decimal_number)
  (binary_number)
  (octal_number)
  (hex_number)
  (unsigned_number)
] @number
