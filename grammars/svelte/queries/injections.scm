((style_element
  (raw_text) @injection.content)
  (#set! injection.language "css"))

((style_element
  (start_tag
    (attribute
      (attribute_name) @_attr
      (quoted_attribute_value (attribute_value) @_value)))
  (raw_text) @injection.content)
  (#eq? @_attr "lang")
  (#match? @_value "^(css|postcss|scss|sass|less)$")
  (#set! injection.language "css"))

((attribute
  (attribute_name) @_attr
  (quoted_attribute_value (attribute_value) @injection.content))
  (#eq? @_attr "style")
  (#set! injection.language "css"))

((script_element
  (raw_text) @injection.content)
  (#set! injection.language "javascript"))

((script_element
  (start_tag
    (attribute
      (attribute_name) @_attr
      (quoted_attribute_value (attribute_value) @_value)))
  (raw_text) @injection.content)
  (#eq? @_attr "lang")
  (#match? @_value "^(ts|typescript)$")
  (#set! injection.language "typescript"))

((raw_text_expr) @injection.content
  (#set! injection.language "javascript"))
