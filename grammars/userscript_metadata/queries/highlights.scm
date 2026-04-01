(comment_prefix) @comment

[
  (start_delimiter)
  (end_delimiter)
] @keyword.directive

(metadata_key
  "@" @punctuation.special)

(key_name) @property.userscript

(locale_suffix) @attribute

(url) @text.uri

(metadata_value
  (metadata_token) @string)

(entry
  key: (metadata_key) @_version_key
  (#match? @_version_key "^@version$")
  value: (metadata_value
    (metadata_token) @number))

(entry
  key: (metadata_key) @_pattern_key
  (#match? @_pattern_key "^@(match|include|exclude|exclude-match|connect)$")
  value: (metadata_value
    (metadata_token) @string.regex))

(entry
  key: (metadata_key) @_grant_key
  (#match? @_grant_key "^@grant$")
  value: (metadata_value
    (metadata_token) @function.builtin))

(entry
  key: (metadata_key) @_grant_none_key
  (#match? @_grant_none_key "^@grant$")
  value: (metadata_value
    (metadata_token) @constant.builtin
    (#eq? @constant.builtin "none")))

(entry
  key: (metadata_key) @_enum_key
  (#match? @_enum_key "^@(run-at|inject-into|sandbox)$")
  value: (metadata_value
    (metadata_token) @constant.builtin))

(entry
  key: (metadata_key) @_resource_key
  (#match? @_resource_key "^@resource$")
  value: (metadata_value
    (metadata_token) @variable
    (url) @text.uri))
