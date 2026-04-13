(entry
  key: (key) @property)

":" @punctuation.delimiter

((value) @number
  (#match? @number "^-?[0-9.]+$"))

(value) @string
