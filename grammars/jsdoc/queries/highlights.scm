; Adapted from zed's JSDoc highlights.
; Source: https://github.com/zed-industries/zed/blob/main/crates/grammars/src/jsdoc/highlights.scm
; License: MIT

(tag_name) @keyword.jsdoc

(type) @type.jsdoc

(inline_tag
  [
    "{"
    "}"
  ] @punctuation.bracket)

(optional_identifier
  [
    "["
    "]"
  ] @punctuation.bracket)

(array_expression
  [
    "["
    "]"
  ] @punctuation.bracket
  "," @punctuation.delimiter)

(member_expression
  [
    "."
    "#"
    "~"
  ] @punctuation.delimiter)

(path_expression
  "/" @punctuation.delimiter)

(qualified_expression
  ":" @punctuation.delimiter)

(optional_identifier
  "=" @operator)

(number) @number

(code_block_language) @string.special

(code_block
  [
    "```"
  ] @punctuation.delimiter)

(optional_identifier
  (identifier) @variable.jsdoc)

(tag
  (optional_identifier
    value: (identifier) @variable.jsdoc))

(tag
  (optional_identifier
    value: (member_expression) @variable.jsdoc))

(tag
  (optional_identifier
    value: (qualified_expression) @variable.jsdoc))

(tag
  (optional_identifier
    value: (path_expression) @text.uri))

(tag
  (optional_identifier
    value: (number) @number))

(identifier) @variable.jsdoc
