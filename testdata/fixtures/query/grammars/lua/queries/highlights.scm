; Tree-sitter query fixture for kat

(
  (function_item
    name: (identifier) @function
    parameters: (parameters (_) @variable.parameter)+) @function.special

  (#eq? @_lang "lua")
  (#match? @function "^(render|load)$")
)

[
  (comment)
  "function"
] @keyword
