(tag_name) @tag
(erroneous_end_tag_name) @tag.error
(attribute_name) @attribute
(attribute_value) @string
(comment) @comment
(special_block_keyword) @keyword.control
(then) @keyword.control
(as) @keyword.control

[
  "{"
  "}"
  "<"
  ">"
  "</"
  "/>"
] @punctuation.bracket

[
  "#"
  ":"
  "/"
  "@"
] @punctuation.special
