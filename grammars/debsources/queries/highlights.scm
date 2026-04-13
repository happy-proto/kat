(comment) @comment

(entry) @keyword

(entry
  uri: (uri) @text.uri)

(entry
  suite: (suite) @type)

(entry
  component: (component) @constant)

(option
  name: (option_name) @property
  value: (option_value) @string.special)

[
  "["
  "]"
] @punctuation.bracket

"=" @operator
