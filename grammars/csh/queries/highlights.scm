(comment) @comment
[(single_quoted_string) (double_quoted_string) (string_content)] @string
(escape_sequence) @string.escape
(substitution_content) @embedded
(variable) @variable
(number) @number
(command name: (word) @function)
((command name: (word) @function.builtin)
 (#match? @function.builtin "^(@|alias|alloc|bg|bindkey|break|breaksw|builtins|bye|case|cd|chdir|complete|continue|dirs|echo|echotc|eval|exec|exit|fg|filetest|glob|goto|hashstat|history|hup|jobs|kill|limit|log|login|logout|ls-F|nice|nohup|notify|onintr|popd|printenv|pushd|rehash|repeat|sched|set|setenv|settc|setty|shift|source|stop|suspend|telltc|time|umask|unalias|uncomplete|unhash|unlimit|unset|unsetenv|wait|warp|watchlog|where|which)$"))
(foreach_statement variable: (word) @variable)
((command name: (word) @_binding . (word) @variable)
 (#match? @_binding "^(set|setenv|unset|unsetenv|@)$"))
((command name: (word) @_alias . (word) @function)
 (#eq? @_alias "alias"))
(case_pattern) @string.regex
(expression_operator) @operator
((expression (word) @operator)
 (#match? @operator "^(-[A-Za-z]+|[=!]=|[=!~]=|=~|!~)$"))
((word) @operator
 (#match? @operator "^(=|[+*/%&|^-]=|[+][+]|--)$"))
["if" "then" "else" "endif" "switch" "case" "default:" "endsw"] @keyword.conditional
["foreach" "while" "end"] @keyword.repeat
["|" "|&" "&&" "||" "&" "<" "<<" ">" ">!" ">&" ">&!" ">>" ">>!" ">>&" ">>&!"] @operator
["(" ")" "`"] @punctuation.bracket
";" @punctuation.delimiter
