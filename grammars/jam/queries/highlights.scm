(comment) @comment
(quoted) @string
(string_content) @string
(escape_sequence) @string.escape
(variable_expansion) @variable
(adjacent_variable_expansion) @variable
(variable_content) @variable
(assignment name: (word) @variable)
(local_statement (word) @variable)
(for_statement variable: (word) @variable)
(parameters (word) @variable.parameter)
(parameter_type) @type
(rule_definition name: (word) @function)
(actions_definition name: (word) @function)
(call_statement name: (word) @function.call)
(rule_call name: (word) @function.call)
(action_text) @string
["rule" "actions" "module" "class" "local" "on" "bind"] @keyword
["if" "else" "switch" "case"] @keyword.conditional
["for" "while" "break" "continue"] @keyword.repeat
["include"] @keyword.import
["return"] @keyword.return
["updated" "together" "ignore" "quietly" "piecemeal" "existing"] @keyword.modifier
["=" "+=" "?=" "default" "!" "!=" "<" "<=" ">" ">=" "&" "&&" "|" "||" "in"] @operator
["{" "}" "[" "]" "(" ")" "$(" "@("] @punctuation.bracket
[":" ";"] @punctuation.delimiter
