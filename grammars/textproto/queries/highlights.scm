(string) @string

(field_name) @property

(comment) @comment

(number) @number
; For stuff like "inf" and "-inf".
((scalar_value (identifier) @constant.builtin)
  (#any-of? @constant.builtin "true" "false" "nan" "inf"))
((scalar_value (signed_identifier) @constant.builtin)
  (#eq? @constant.builtin "-inf"))

(open_squiggly) @punctuation.bracket
(close_squiggly) @punctuation.bracket
(open_square) @punctuation.bracket
(close_square) @punctuation.bracket
(open_arrow) @punctuation.bracket
(close_arrow) @punctuation.bracket
