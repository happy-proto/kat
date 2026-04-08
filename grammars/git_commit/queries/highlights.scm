(subject) @string
(path) @string.special
(branch) @string.special
(commit) @constant
(item) @number
(header) @keyword
(message) @string

(change kind: "new file" @diff.plus)
(change kind: "deleted" @diff.minus)
(change kind: "modified" @keyword)
(change kind: "renamed" @keyword)

(trailer
  key: (trailer_key) @property
  value: (trailer_value) @string)

[":" "=" "->" (scissors)] @punctuation.delimiter
(comment) @comment
