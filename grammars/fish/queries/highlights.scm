(comment) @comment

((program
  .
  (comment) @keyword.directive)
 (#match? @keyword.directive "^#![ \t]*/"))

[(double_quote_string) (single_quote_string)] @string
(escape_sequence) @string.escape

[(integer) (float)] @number

(variable_expansion) @variable

(for_statement
  variable: (variable_name) @variable)

(home_dir_expansion) @variable.special

(command_substitution) @embedded

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

(case_clause
  [
    (word)
    (concatenation)
    (single_quote_string)
    (double_quote_string)
    (glob)
  ] @string.regex)

(function_definition name: [(word) (concatenation)] @function)

(command name: [(word) (concatenation)] @function)

((command
  name: (word) @function.builtin)
 (#match? @function.builtin "^(abbr|argparse|bg|bind|block|builtin|cd|command|commandline|complete|contains|count|dirh|dirs|disown|echo|emit|eval|exec|false|fg|fish_add_path|fish_breakpoint|fish_command_not_found|fish_config|fish_indent|fish_is_root_user|fish_key_reader|fish_mode_prompt|fish_opt|fish_prompt|fish_right_prompt|fish_sigtrap_handler|fish_status_to_signal|fish_svn_prompt|fish_title|functions|history|jobs|math|printf|pwd|random|read|realpath|set|set_color|source|status|string|test|true|type|ulimit|wait)$"))

((command
  name: (word) @_builtin
  argument: (word) @keyword.directive)
 (#eq? @_builtin "status")
 (#match? @keyword.directive "^(is-(block|breakpoint|command-substitution|full-job-control|interactive|interread|login|no-job-control)|current-(command|commandline|filename|function)|filename|fish-path|function|job-control|line-number|stack-trace|terminal|test-feature)$"))

((command
  name: (word) @_builtin
  argument: (word) @keyword.directive)
 (#eq? @_builtin "string")
 (#match? @keyword.directive "^(collect|escape|join|join0|length|lower|match|pad|repeat|replace|shorten|split|split0|sub|trim|unescape|upper)$"))

((function_definition
  option: (word) @variable.parameter)
 (#not-match? @variable.parameter "^--"))

((command
  argument: (word) @constant)
 (#match? @constant "^--?[A-Za-z][A-Za-z0-9-]*$"))

(function_definition
  option: (word) @constant
  (#match? @constant "^--?[A-Za-z][A-Za-z0-9-]*$"))

((variable_expansion) @variable.special
 (#match? @variable.special "^\\$(argv|status|pipestatus|last_pid|CMD_DURATION|SHLVL|PWD|HOME|USER|hostname|version|fish_[A-Za-z0-9_]+)(\\[[^]]+\\])*$"))

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
