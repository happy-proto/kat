(((comment) @_jsdoc_comment
  (#match? @_jsdoc_comment "(?s)^/[*][*][^*].*[*]/$")) @injection.content
  (#set! injection.language "jsdoc"))

((comment) @injection.content
  (#match? @injection.content "^//\\s*==/?UserScript==\\s*$")
  (#set! injection.language "userscript_metadata")
  (#set! injection.combined))

((comment) @injection.content
  (#match? @injection.content "^//\\s*@(name|namespace|version|description|author|match|include|exclude|exclude-match|grant|require|resource|run-at|noframes|unwrap|inject-into|sandbox|downloadURL|updateURL|supportURL|homepage(URL)?|icon(64|URL)?|connect|antifeature|compatible|license)\\b")
  (#set! injection.language "userscript_metadata")
  (#set! injection.combined))

((regex_pattern) @injection.content
  (#set! injection.language "regex_javascript"))

((new_expression
  constructor: (identifier) @_ctor
  (#eq? @_ctor "RegExp")
  arguments: (arguments
    (string) @injection.content))
  (#set! kat.decode "javascript-literal")
  (#set! injection.language "regex_javascript"))

((new_expression
  constructor: (identifier) @_ctor
  (#eq? @_ctor "RegExp")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content)))
  (#set! kat.decode "javascript-string")
  (#set! injection.language "regex_javascript"))

((new_expression
  constructor: (identifier) @_ctor
  (#eq? @_ctor "RegExp")
  arguments: (arguments
    (call_expression
      function: (member_expression
        object: (identifier) @_raw_obj
        (#eq? @_raw_obj "String")
        property: (property_identifier) @_raw_prop
        (#eq? @_raw_prop "raw"))
      arguments: (template_string
        (string_fragment) @injection.content))))
  (#set! injection.language "regex_javascript"))

((call_expression
  function: (identifier) @_ctor
  (#eq? @_ctor "RegExp")
  arguments: (arguments
    (string) @injection.content))
  (#set! kat.decode "javascript-literal")
  (#set! injection.language "regex_javascript"))

((call_expression
  function: (identifier) @_ctor
  (#eq? @_ctor "RegExp")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content)))
  (#set! kat.decode "javascript-string")
  (#set! injection.language "regex_javascript"))

((call_expression
  function: (identifier) @_ctor
  (#eq? @_ctor "RegExp")
  arguments: (arguments
    (call_expression
      function: (member_expression
        object: (identifier) @_raw_obj
        (#eq? @_raw_obj "String")
        property: (property_identifier) @_raw_prop
        (#eq? @_raw_prop "raw"))
      arguments: (template_string
        (string_fragment) @injection.content))))
  (#set! injection.language "regex_javascript"))

(call_expression
  function: (identifier) @_name
  (#eq? @_name "css")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "css"))

(call_expression
  function: (identifier) @_name
  (#eq? @_name "css")
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "css"))

(call_expression
  function: (member_expression
    object: (identifier) @_obj
    (#eq? @_obj "styled")
    property: (property_identifier))
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "css"))

(call_expression
  function: (member_expression
    object: (identifier) @_obj
    (#eq? @_obj "styled")
    property: (property_identifier))
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "css"))

(call_expression
  function: (identifier) @_name
  (#eq? @_name "html")
  arguments: (arguments
    (template_string) @injection.content)
  (#set! injection.language "html")
  (#set! injection.include-children))

(call_expression
  function: (identifier) @_name
  (#eq? @_name "html")
  arguments: (template_string) @injection.content
  (#set! injection.language "html")
  (#set! injection.include-children))

(call_expression
  function: (identifier) @_name
  (#eq? @_name "js")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "javascript"))

(call_expression
  function: (identifier) @_name
  (#eq? @_name "js")
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "javascript"))

(call_expression
  function: (identifier) @_name
  (#eq? @_name "json")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "json"))

(call_expression
  function: (identifier) @_name
  (#eq? @_name "json")
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "json"))

(call_expression
  function: (identifier) @_name
  (#eq? @_name "sql")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "sql"))

(call_expression
  function: (identifier) @_name
  (#eq? @_name "sql")
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "sql"))

(call_expression
  function: (member_expression
    property: (property_identifier) @_name)
  (#eq? @_name "sql")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "sql"))

(call_expression
  function: (member_expression
    property: (property_identifier) @_name)
  (#eq? @_name "sql")
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "sql"))

(call_expression
  function: (identifier) @_name
  (#match? @_name "^(psql|postgres|postgresql|pgsql)$")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "sql-postgres"))

(call_expression
  function: (identifier) @_name
  (#match? @_name "^(psql|postgres|postgresql|pgsql)$")
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "sql-postgres"))

(call_expression
  function: (identifier) @_name
  (#match? @_name "^(mysql|mariadb)$")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "sql-mysql"))

(call_expression
  function: (identifier) @_name
  (#match? @_name "^(mysql|mariadb)$")
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "sql-mysql"))

(call_expression
  function: (identifier) @_name
  (#match? @_name "^(sqlite|sqlite3)$")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "sql-sqlite"))

(call_expression
  function: (identifier) @_name
  (#match? @_name "^(sqlite|sqlite3)$")
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "sql-sqlite"))

(call_expression
  function: (identifier) @_name
  (#match? @_name "^ya?ml$")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "yaml"))

(call_expression
  function: (identifier) @_name
  (#match? @_name "^ya?ml$")
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "yaml"))

(call_expression
  function: (identifier) @_name
  (#match? @_name "^g(raph)?ql$")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "graphql"))

(call_expression
  function: (identifier) @_name
  (#match? @_name "^g(raph)?ql$")
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "graphql"))

(call_expression
  function: (member_expression
    property: (property_identifier) @_name)
  (#match? @_name "^g(raph)?ql$")
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content))
  (#set! injection.language "graphql"))

(call_expression
  function: (member_expression
    property: (property_identifier) @_name)
  (#match? @_name "^g(raph)?ql$")
  arguments: (template_string
    (string_fragment) @injection.content)
  (#set! injection.language "graphql"))

((call_expression
  function: (member_expression
    property: (property_identifier) @_fn)
  arguments: (arguments
    (string) @injection.content))
  (#match? @_fn "^(query|queryRaw|queryRawUnsafe|execute|executeRaw|executeRawUnsafe|prepare)$")
  (#match? @injection.content "(?is)^[\"'`]\\s*(select|with|insert|update|delete|create|alter|drop|truncate|merge|pragma|vacuum|explain)\\b")
  (#set! kat.decode "javascript-literal")
  (#set! injection.language "sql"))

((call_expression
  function: (member_expression
    property: (property_identifier) @_fn)
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content)))
  (#match? @_fn "^(query|queryRaw|queryRawUnsafe|execute|executeRaw|executeRawUnsafe|prepare)$")
  (#match? @injection.content "(?is)^\\s*(select|with|insert|update|delete|create|alter|drop|truncate|merge|pragma|vacuum|explain)\\b")
  (#set! kat.decode "javascript-string")
  (#set! injection.language "sql"))

((call_expression
  function: (member_expression
    property: (property_identifier) @_fn)
  arguments: (arguments
    (string) @injection.content))
  (#match? @_fn "^(all|get|run)$")
  (#match? @injection.content "(?is)^[\"'`]\\s*(select|with|insert|update|delete|create|alter|drop|truncate|merge|pragma|vacuum|explain)\\b")
  (#set! kat.decode "javascript-literal")
  (#set! injection.language "sql"))

((call_expression
  function: (member_expression
    property: (property_identifier) @_fn)
  arguments: (arguments
    (template_string
      (string_fragment) @injection.content)))
  (#match? @_fn "^(all|get|run)$")
  (#match? @injection.content "(?is)^\\s*(select|with|insert|update|delete|create|alter|drop|truncate|merge|pragma|vacuum|explain)\\b")
  (#set! kat.decode "javascript-string")
  (#set! injection.language "sql"))
