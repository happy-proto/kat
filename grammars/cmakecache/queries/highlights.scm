(comment) @comment

(entry
  name: (name) @property
  kind: (cache_type) @type.builtin)

":" @punctuation.delimiter
"=" @operator

((value) @boolean
  (#match? @boolean "^(ON|OFF|TRUE|FALSE|YES|NO)$"))

((value) @number
  (#match? @number "^-?[0-9]+$"))

((value) @string.special.path
  (#match? @string.special.path "^(/|[A-Za-z]:[\\\\/]).*"))

(value) @string
