; Adapted from tree-sitter/tree-sitter-toml.
; Source: https://github.com/tree-sitter/tree-sitter-toml
; License: MIT

; Properties
;-----------

(bare_key) @property.toml
(quoted_key) @property.toml

; Literals
;---------

(boolean) @boolean
(comment) @comment
(string) @string
(escape_sequence) @string.escape
(integer) @number
(float) @number
(offset_date_time) @datetime
(local_date_time) @datetime
(local_date) @datetime
(local_time) @datetime

; Punctuation
;------------

"." @punctuation.delimiter
"," @punctuation.delimiter

"=" @operator

"[" @punctuation.bracket
"]" @punctuation.bracket
"[[" @punctuation.bracket
"]]" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket
