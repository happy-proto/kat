(comment) @comment

(group
  name: (identifier) @keyword)

(assignment
  name: (identifier) @property)

[
  "&"
  "/"
] @punctuation.bracket

[
  "="
  ","
] @punctuation.delimiter

(boolean) @boolean
(number) @number
(string) @string
(identifier) @variable
