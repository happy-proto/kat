; Comments
(comment_line) @comment

; Options
(option
  name: (option_name) @property)

(option
  value: (option_value
    (quoted_value) @string))

(option
  value: (option_value
    (bare_value) @string.special))

"," @punctuation.delimiter
"=" @operator

; Key payload
(key_type) @type
(key_blob) @constant

(entry
  comment: (entry_comment) @string.special)
