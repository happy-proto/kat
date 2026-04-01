; Adapted from zed's Go injections with the same runtime-reuse philosophy used
; throughout kat. Unsupported child runtimes are skipped automatically.
; Source: https://github.com/zed-industries/zed/blob/main/crates/grammars/src/go/injections.scm
; License: MIT

((call_expression
  (selector_expression) @_function
  (#any-of? @_function
    "regexp.Compile"
    "regexp.Match"
    "regexp.MatchReader"
    "regexp.MatchString"
    "regexp.MustCompile")
  (argument_list
    .
    [
      (raw_string_literal
        (raw_string_literal_content) @injection.content)
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
    ]))
  (#set! kat.decode "go-string")
  (#set! injection.language "regex_go"))

((call_expression
  (selector_expression) @_function
  (#any-of? @_function
    "regexp.CompilePOSIX"
    "regexp.MustCompilePOSIX")
  (argument_list
    .
    [
      (raw_string_literal
        (raw_string_literal_content) @injection.content)
      (interpreted_string_literal
        (interpreted_string_literal_content) @injection.content)
    ]))
  (#set! kat.decode "go-string")
  (#set! injection.language "regex_posix"))

((call_expression
  function: (selector_expression
    field: (field_identifier) @_fn)
  arguments: (argument_list
    .
    [
      (raw_string_literal
        (raw_string_literal_content) @injection.content)
      (interpreted_string_literal
        (interpreted_string_literal_content) @injection.content)
    ]))
  (#any-of? @_fn
    "Query"
    "QueryContext"
    "QueryRow"
    "QueryRowContext"
    "Exec"
    "ExecContext"
    "Prepare"
    "PrepareContext"
    "MustExec")
  (#match? @injection.content "(?is)^\\s*(select|with|insert|update|delete|create|alter|drop|truncate|merge|pragma|vacuum|explain)\\b")
  (#set! kat.decode "go-string")
  (#set! injection.language "sql"))

([
  (const_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (var_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (assignment_statement
    left: (expression_list)
    "="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (short_var_declaration
    left: (expression_list)
    ":="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (composite_literal
    body: (literal_value
      (keyed_element
        (comment) @_comment
        value: (literal_element
          [
            (interpreted_string_literal
              (interpreted_string_literal_content) @injection.content)
            (raw_string_literal
              (raw_string_literal_content) @injection.content)
          ]))))
  (expression_statement
    (call_expression
      (argument_list
        (comment) @_comment
        [
          (interpreted_string_literal
            (interpreted_string_literal_content) @injection.content)
          (raw_string_literal
            (raw_string_literal_content) @injection.content)
        ])))
]
  (#match? @_comment "^\\/\\*\\s*sql\\s*\\*\\/$")
  (#set! kat.decode "go-string")
  (#set! injection.language "sql"))

([
  (const_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (var_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (assignment_statement
    left: (expression_list)
    "="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (short_var_declaration
    left: (expression_list)
    ":="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (composite_literal
    body: (literal_value
      (keyed_element
        (comment) @_comment
        value: (literal_element
          [
            (interpreted_string_literal
              (interpreted_string_literal_content) @injection.content)
            (raw_string_literal
              (raw_string_literal_content) @injection.content)
          ]))))
  (expression_statement
    (call_expression
      (argument_list
        (comment) @_comment
        [
          (interpreted_string_literal
            (interpreted_string_literal_content) @injection.content)
          (raw_string_literal
            (raw_string_literal_content) @injection.content)
        ])))
]
  (#match? @_comment "^\\/\\*\\s*sql:(postgres|postgresql|pgsql)\\s*\\*\\/$")
  (#set! kat.decode "go-string")
  (#set! injection.language "sql-postgres"))

([
  (const_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (var_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (assignment_statement
    left: (expression_list)
    "="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (short_var_declaration
    left: (expression_list)
    ":="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (composite_literal
    body: (literal_value
      (keyed_element
        (comment) @_comment
        value: (literal_element
          [
            (interpreted_string_literal
              (interpreted_string_literal_content) @injection.content)
            (raw_string_literal
              (raw_string_literal_content) @injection.content)
          ]))))
  (expression_statement
    (call_expression
      (argument_list
        (comment) @_comment
        [
          (interpreted_string_literal
            (interpreted_string_literal_content) @injection.content)
          (raw_string_literal
            (raw_string_literal_content) @injection.content)
        ])))
]
  (#match? @_comment "^\\/\\*\\s*sql:(mysql|mariadb)\\s*\\*\\/$")
  (#set! kat.decode "go-string")
  (#set! injection.language "sql-mysql"))

([
  (const_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (var_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (assignment_statement
    left: (expression_list)
    "="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (short_var_declaration
    left: (expression_list)
    ":="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (composite_literal
    body: (literal_value
      (keyed_element
        (comment) @_comment
        value: (literal_element
          [
            (interpreted_string_literal
              (interpreted_string_literal_content) @injection.content)
            (raw_string_literal
              (raw_string_literal_content) @injection.content)
          ]))))
  (expression_statement
    (call_expression
      (argument_list
        (comment) @_comment
        [
          (interpreted_string_literal
            (interpreted_string_literal_content) @injection.content)
          (raw_string_literal
            (raw_string_literal_content) @injection.content)
        ])))
]
  (#match? @_comment "^\\/\\*\\s*sql:(sqlite|sqlite3)\\s*\\*\\/$")
  (#set! kat.decode "go-string")
  (#set! injection.language "sql-sqlite"))

([
  (const_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (var_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (assignment_statement
    left: (expression_list)
    "="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (short_var_declaration
    left: (expression_list)
    ":="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (composite_literal
    body: (literal_value
      (keyed_element
        (comment) @_comment
        value: (literal_element
          [
            (interpreted_string_literal
              (interpreted_string_literal_content) @injection.content)
            (raw_string_literal
              (raw_string_literal_content) @injection.content)
          ]))))
  (expression_statement
    (call_expression
      (argument_list
        (comment) @_comment
        [
          (interpreted_string_literal
            (interpreted_string_literal_content) @injection.content)
          (raw_string_literal
            (raw_string_literal_content) @injection.content)
        ])))
]
  (#match? @_comment "^\\/\\*\\s*json\\s*\\*\\/$")
  (#set! injection.language "json"))

([
  (const_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (var_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (assignment_statement
    left: (expression_list)
    "="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (short_var_declaration
    left: (expression_list)
    ":="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (composite_literal
    body: (literal_value
      (keyed_element
        (comment) @_comment
        value: (literal_element
          [
            (interpreted_string_literal
              (interpreted_string_literal_content) @injection.content)
            (raw_string_literal
              (raw_string_literal_content) @injection.content)
          ]))))
  (expression_statement
    (call_expression
      (argument_list
        (comment) @_comment
        [
          (interpreted_string_literal
            (interpreted_string_literal_content) @injection.content)
          (raw_string_literal
            (raw_string_literal_content) @injection.content)
        ])))
]
  (#match? @_comment "^\\/\\*\\s*yaml\\s*\\*\\/$")
  (#set! injection.language "yaml"))

([
  (const_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (var_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (assignment_statement
    left: (expression_list)
    "="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (short_var_declaration
    left: (expression_list)
    ":="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (composite_literal
    body: (literal_value
      (keyed_element
        (comment) @_comment
        value: (literal_element
          [
            (interpreted_string_literal
              (interpreted_string_literal_content) @injection.content)
            (raw_string_literal
              (raw_string_literal_content) @injection.content)
          ]))))
  (expression_statement
    (call_expression
      (argument_list
        (comment) @_comment
        [
          (interpreted_string_literal
            (interpreted_string_literal_content) @injection.content)
          (raw_string_literal
            (raw_string_literal_content) @injection.content)
        ])))
]
  (#match? @_comment "^\\/\\*\\s*html\\s*\\*\\/$")
  (#set! injection.language "html"))

([
  (const_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (var_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (assignment_statement
    left: (expression_list)
    "="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (short_var_declaration
    left: (expression_list)
    ":="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (composite_literal
    body: (literal_value
      (keyed_element
        (comment) @_comment
        value: (literal_element
          [
            (interpreted_string_literal
              (interpreted_string_literal_content) @injection.content)
            (raw_string_literal
              (raw_string_literal_content) @injection.content)
          ]))))
  (expression_statement
    (call_expression
      (argument_list
        (comment) @_comment
        [
          (interpreted_string_literal
            (interpreted_string_literal_content) @injection.content)
          (raw_string_literal
            (raw_string_literal_content) @injection.content)
        ])))
]
  (#match? @_comment "^\\/\\*\\s*js\\s*\\*\\/$")
  (#set! injection.language "javascript"))

([
  (const_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (var_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (assignment_statement
    left: (expression_list)
    "="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (short_var_declaration
    left: (expression_list)
    ":="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (composite_literal
    body: (literal_value
      (keyed_element
        (comment) @_comment
        value: (literal_element
          [
            (interpreted_string_literal
              (interpreted_string_literal_content) @injection.content)
            (raw_string_literal
              (raw_string_literal_content) @injection.content)
          ]))))
  (expression_statement
    (call_expression
      (argument_list
        (comment) @_comment
        [
          (interpreted_string_literal
            (interpreted_string_literal_content) @injection.content)
          (raw_string_literal
            (raw_string_literal_content) @injection.content)
        ])))
]
  (#match? @_comment "^\\/\\*\\s*css\\s*\\*\\/$")
  (#set! injection.language "css"))

([
  (const_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (var_spec
    name: (identifier)
    "="
    (comment) @_comment
    value: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (assignment_statement
    left: (expression_list)
    "="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (short_var_declaration
    left: (expression_list)
    ":="
    (comment) @_comment
    right: (expression_list
      [
        (interpreted_string_literal
          (interpreted_string_literal_content) @injection.content)
        (raw_string_literal
          (raw_string_literal_content) @injection.content)
      ]))
  (composite_literal
    body: (literal_value
      (keyed_element
        (comment) @_comment
        value: (literal_element
          [
            (interpreted_string_literal
              (interpreted_string_literal_content) @injection.content)
            (raw_string_literal
              (raw_string_literal_content) @injection.content)
          ]))))
  (expression_statement
    (call_expression
      (argument_list
        (comment) @_comment
        [
          (interpreted_string_literal
            (interpreted_string_literal_content) @injection.content)
          (raw_string_literal
            (raw_string_literal_content) @injection.content)
        ])))
]
  (#match? @_comment "^\\/\\*\\s*bash\\s*\\*\\/$")
  (#set! injection.language "bash"))
