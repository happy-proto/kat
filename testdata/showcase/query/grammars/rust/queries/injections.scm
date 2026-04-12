; Rich Tree-sitter query showcase for kat

(
  (macro_invocation
    macro: (identifier) @_macro
    (token_tree
      [
        (string_literal)
        (raw_string_literal)
      ] @injection.content))

  (#any-of? @_macro
    "sql"
    "sqlx::query"
    "regex")

  (#set! injection.language "sql")
) @function.special

(
  (line_comment)+ @comment.documentation

  (#match? @comment.documentation "^///\\s+sql")
)

[
  (line_comment)
  (block_comment)
] @comment
