(commit_keyword) @keyword
(commit_hash) @constant

(merge_line
  key: (merge_key) @keyword)

(merge_line
  (commit_hash) @constant)

(identity_line
  key: (identity_key) @keyword
  name: (person_name) @function
  email: (email) @function.builtin)

(identity_line
  key: (identity_key) @keyword
  email: (email) @function.builtin)

(date_line
  key: (date_key) @keyword
  value: (metadata_value) @string)

(ref_list
  (reference) @string.special)

(message_text) @string

[
  "("
  ")"
] @punctuation.bracket

"," @punctuation.delimiter
