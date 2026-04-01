(comment) @comment

((program
  .
  (comment) @keyword.directive)
 (#match? @keyword.directive "^#![ \t]*/"))

[(double_quote_string) (single_quote_string)] @string
(escape_sequence) @string.escape

[(integer) (float)] @number

(variable_expansion) @variable

(home_dir_expansion) @variable.special

(glob) @string.regex

[
  "&&"
  "||"
  "|"
  "&|"
  "2>|"
  "&"
  ".."
  (direction)
  (stream_redirect)
] @operator

; match operators of test command
(command
  name: (word) @function (#match? @function "^test$")
  argument: (word) @operator (#match? @operator "^(!?=|-[a-zA-Z]+)$"))

; match operators of [ command
(command
  name: (word) @punctuation.bracket (#match? @punctuation.bracket "^\\[$")
  argument: (word) @operator (#match? @operator "^(!?=|-[a-zA-Z]+)$"))

[
 "["
 "]"
 "{"
 "}"
 "("
 ")"
] @punctuation.bracket

"," @punctuation.delimiter

(function_definition name: [(word) (concatenation)] @function)

(command name: [(word) (concatenation)] @function)

((command
  name: (word) @function.builtin)
 (#match? @function.builtin "^(abbr|argparse|builtin|command|contains|count|math|printf|read|set|set_color|source|status|string|test)$"))

((command
  argument: (word) @constant)
 (#match? @constant "^--?[A-Za-z][A-Za-z0-9-]*$"))

(function_definition
  option: (word) @constant
  (#match? @constant "^--?[A-Za-z][A-Za-z0-9-]*$"))

[
 "function"
 "begin"
 "case"
 "in"
 "switch"
] @keyword

[
 "if"
 "else"
 "end"
 "while"
 "for"
 "not"
 "!"
 "and"
 "or"
 "return"
 (break)
 (continue)
] @keyword.control
