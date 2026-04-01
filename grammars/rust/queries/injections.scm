([
  (line_comment)
  (block_comment)
] @injection.content
 (#set! injection.language "comment"))

((macro_invocation
  macro: [
    ((identifier) @_macro_name)
    (scoped_identifier (identifier) @_macro_name .)
  ]
  (#not-any-of? @_macro_name
    "view"
    "html"
    "sql"
    "query"
    "query_as"
    "query_scalar"
    "query_file"
    "query_file_as"
    "query_file_scalar"
    "regex"
    "bytes_regex")
  (token_tree) @injection.content)
 (#set! injection.language "rust")
 (#set! injection.include-children))

((macro_rule
  (token_tree) @injection.content)
 (#set! injection.language "rust")
 (#set! injection.include-children))

((macro_invocation
  macro: [
    ((identifier) @_macro_name)
    (scoped_identifier (identifier) @_macro_name .)
  ]
  (#any-of? @_macro_name
    "sql"
    "query"
    "query_as"
    "query_scalar"
    "query_file"
    "query_file_as"
    "query_file_scalar")
  (token_tree
    [
      (string_literal
        (string_content) @injection.content)
      (raw_string_literal
        (string_content) @injection.content)
    ]))
 (#set! kat.decode "rust-string")
 (#set! injection.language "sql"))

((macro_invocation
  macro: [
    ((identifier) @_macro_name)
    (scoped_identifier (identifier) @_macro_name .)
  ]
  (#any-of? @_macro_name "regex" "bytes_regex")
  (token_tree
    [
      (string_literal
        (string_content) @injection.content)
      (raw_string_literal
        (string_content) @injection.content)
    ]))
 (#set! kat.decode "rust-string")
 (#set! injection.language "regex_rust"))

((call_expression
  function: (scoped_identifier) @_fn_path
  arguments: (arguments
    [
      (string_literal
        (string_content) @injection.content)
      (raw_string_literal
        (string_content) @injection.content)
    ]))
 (#match? @_fn_path ".*Regex(Builder)?::new")
 (#set! kat.decode "rust-string")
 (#set! injection.language "regex_rust"))

((call_expression
  function: (field_expression
    field: (field_identifier) @_fn)
  arguments: (arguments
    .
    [
      (string_literal
        (string_content) @injection.content)
      (raw_string_literal
        (string_content) @injection.content)
    ]))
 (#any-of? @_fn
   "query"
   "query_one"
   "query_opt"
   "query_raw"
   "query_typed"
   "execute"
   "prepare"
   "prepare_typed"
   "simple_query"
   "batch_execute")
 (#match? @injection.content "(?is)^\\s*(select|with|insert|update|delete|create|alter|drop|truncate|merge|pragma|vacuum|explain)\\b")
 (#set! kat.decode "rust-string")
 (#set! injection.language "sql"))

((line_comment
  (doc_comment) @injection.content)
 (#set! injection.combined)
 (#set! injection.language "markdown"))

((block_comment
  (doc_comment) @injection.content)
 (#set! injection.combined)
 (#set! injection.language "markdown"))
