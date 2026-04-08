(comment) @comment

(funcdef
  (identifier) @function)

(binding
  (variable) @variable.parameter)

[
  (identifier)
  (keyword)
] @variable

(objectkey
  (identifier) @property)

(index
  (identifier) @property)

(variable) @variable.parameter
(format) @string.special
(number) @number
(string) @string
(recurse) @keyword

[
  "module"
  "import"
  "include"
  "as"
  "def"
  "if"
  "then"
  "elif"
  "else"
  "end"
  "reduce"
  "foreach"
  "try"
  "catch"
  "label"
  "break"
  "or"
  "and"
  "true"
  "false"
  "null"
] @keyword

[
  "|"
  ","
  "//"
  "="
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "+"
  "-"
  "*"
  "/"
  "%"
] @operator

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  "."
  ":"
  ";"
] @punctuation.delimiter
