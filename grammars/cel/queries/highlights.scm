; CEL expressions are embedded in protobuf option strings.
[(string_literal) (bytes_literal)] @string
[(int_literal) (uint_literal) (float_literal)] @number
[(true) (false) (null)] @constant.builtin
(identifier) @variable
(select_expression member: [(identifier) (reserved_keyword)] @property.cel)
(call_expression function: (identifier) @function.call)
(absolute_expression name: (identifier) @function.call)
(member_call_expression function: [(identifier) (reserved_keyword)] @function.call)
(reserved_keyword) @keyword
"in" @keyword.operator
[
  "-" "!" "*" "/" "&&" "%" "+" "<" "<=" "!=" "==" ">" ">=" "||" "?"
] @operator
["(" ")" "[" "]" "{" "}"] @punctuation.bracket
["," "." ":"] @punctuation.delimiter
(comment) @comment
