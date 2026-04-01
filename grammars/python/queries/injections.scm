; Adapted from zed's Python SQL injections and extended with regex call support.
; Source: https://github.com/zed-industries/zed/blob/main/crates/languages/src/python/injections.scm
; License: Apache-2.0

((comment) @injection.content
  (#set! injection.language "comment"))

(
  [
    (call
      [
        (attribute attribute: (identifier) @function_name)
        (identifier) @function_name
      ]
      arguments: (argument_list
        (comment) @comment
        (string
          (string_content) @injection.content)))
    ((comment) @comment
      .
      (expression_statement
        (assignment
          right: (string
            (string_content) @injection.content))))
  ]
  (#match? @comment "^(#|#\\s+)(?i:sql)\\s*$")
  (#set! kat.decode "python-string")
  (#set! injection.language "sql"))

(
  [
    (call
      [
        (attribute attribute: (identifier) @function_name)
        (identifier) @function_name
      ]
      arguments: (argument_list
        (comment) @comment
        (string
          (string_content) @injection.content)))
    ((comment) @comment
      .
      (expression_statement
        (assignment
          right: (string
            (string_content) @injection.content))))
  ]
  (#match? @comment "^(#|#\\s+)(?i:sql:(postgres|postgresql|pgsql))\\s*$")
  (#set! kat.decode "python-string")
  (#set! injection.language "sql-postgres"))

(
  [
    (call
      [
        (attribute attribute: (identifier) @function_name)
        (identifier) @function_name
      ]
      arguments: (argument_list
        (comment) @comment
        (string
          (string_content) @injection.content)))
    ((comment) @comment
      .
      (expression_statement
        (assignment
          right: (string
            (string_content) @injection.content))))
  ]
  (#match? @comment "^(#|#\\s+)(?i:sql:(mysql|mariadb))\\s*$")
  (#set! kat.decode "python-string")
  (#set! injection.language "sql-mysql"))

(
  [
    (call
      [
        (attribute attribute: (identifier) @function_name)
        (identifier) @function_name
      ]
      arguments: (argument_list
        (comment) @comment
        (string
          (string_content) @injection.content)))
    ((comment) @comment
      .
      (expression_statement
        (assignment
          right: (string
            (string_content) @injection.content))))
  ]
  (#match? @comment "^(#|#\\s+)(?i:sql:(sqlite|sqlite3))\\s*$")
  (#set! kat.decode "python-string")
  (#set! injection.language "sql-sqlite"))

((call
  function: (attribute
    attribute: (identifier) @_fn)
  arguments: (argument_list
    [
      (string
        (string_content) @injection.content)
      (concatenated_string
        (string
          (string_content) @injection.content))
    ]))
  (#any-of? @_fn "execute" "executemany" "executescript")
  (#match? @injection.content "(?is)^\\s*(select|with|insert|update|delete|create|alter|drop|truncate|merge|pragma|vacuum|explain)\\b")
  (#set! kat.decode "python-string")
  (#set! injection.language "sql"))

((call
  function: (attribute
    object: (identifier) @_module
    attribute: (identifier) @_fn)
  arguments: (argument_list
    [
      (string
        (string_content) @injection.content)
      (concatenated_string
        (string
          (string_content) @injection.content))
    ]))
  (#any-of? @_module "re" "regex")
  (#any-of? @_fn "compile" "search" "match" "fullmatch" "findall" "finditer" "sub" "subn" "split")
  (#set! kat.decode "python-string")
  (#set! injection.language "regex_python"))

((call
  function: (identifier) @_fn
  arguments: (argument_list
    [
      (string
        (string_content) @injection.content)
      (concatenated_string
        (string
          (string_content) @injection.content))
    ]))
  (#any-of? @_fn "compile" "search" "match" "fullmatch" "findall" "finditer" "sub" "subn" "split")
  (#set! kat.decode "python-string")
  (#set! injection.language "regex_python"))
