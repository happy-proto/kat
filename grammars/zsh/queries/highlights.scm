; Adapted from georgeharker/tree-sitter-zsh and tree-sitter/tree-sitter-bash.

[
  (string)
  (raw_string)
  (heredoc_body)
  (heredoc_start)
  (heredoc_end)
  (ansi_c_string)
  (word)
] @string

(comment) @comment

((program
  .
  (comment) @keyword.directive)
 (#match? @keyword.directive "^#![ \t]*/"))

(function_definition
  name: (word) @function)

[
  (command_name)
  (declaration_command)
] @function

[
  (variable_name)
  (simple_variable_name)
] @variable

(special_variable_name) @variable.special

[
  "export"
  "function"
  "repeat"
  "unset"
  "local"
  "typeset"
  "declare"
  "readonly"
  "integer"
  "float"
] @keyword

[
  "case"
  "coproc"
  "do"
  "done"
  "elif"
  "else"
  "esac"
  "fi"
  "for"
  "if"
  "in"
  "select"
  "then"
  "until"
  "while"
] @keyword.control

[
  (file_descriptor)
  (number)
] @number

(regex) @string.regex

[
  (command_substitution)
  (process_substitution)
  (expansion)
  (arithmetic_expansion)
] @embedded

[
  "&&"
  "||"
  ">"
  ">>"
  "<"
  "|"
  "="
  "=~"
  "=="
] @operator

(test_operator) @keyword.operator

";" @punctuation.delimiter

[
  "("
  ")"
  "{"
  "}"
  "["
  "]"
  "[["
  "]]"
] @punctuation.bracket

[
  (zsh_glob_qualifier)
  (case_item
    value: (_))
] @string.regex

((command
  (_) @constant)
 (#match? @constant "^-"))

((command_name) @function.builtin
 (#match? @function.builtin "^(autoload|compdef|emulate|print|setopt|source|unsetopt|zmodload)$"))

((command
  name: (command_name) @_builtin
  (_) @constant)
 (#match? @_builtin "^(autoload|compdef|emulate|setopt|source|typeset|unsetopt|zmodload)$")
 (#match? @constant "^--?[A-Za-z][A-Za-z0-9-]*$"))

((command
  name: (command_name) @_builtin
  (_) @keyword.directive)
 (#match? @_builtin "^(setopt|unsetopt)$")
 (#match? @keyword.directive "^[A-Za-z_][A-Za-z0-9_]*$"))
