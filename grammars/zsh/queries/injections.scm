; Repository-local injections for nested languages within Zsh.

((comment) @injection.content
  (#set! injection.language "comment"))

; python/python3/uv <<'PY'
((redirected_statement
  body: (command
    name: (command_name) @_cmd)
  redirect: (heredoc_redirect
    (heredoc_body) @injection.content))
  (#any-of? @_cmd "python" "python3" "uv")
  (#set! injection.language "python"))

; node/nodejs/bun <<'JS'
((redirected_statement
  body: (command
    name: (command_name) @_cmd)
  redirect: (heredoc_redirect
    (heredoc_body) @injection.content))
  (#any-of? @_cmd "node" "nodejs" "bun")
  (#set! injection.language "javascript"))

; bash/sh <<'SH'
((redirected_statement
  body: (command
    name: (command_name) @_cmd)
  redirect: (heredoc_redirect
    (heredoc_body) @injection.content))
  (#any-of? @_cmd "bash" "sh")
  (#set! injection.language "bash"))

; zsh <<'ZSH'
((redirected_statement
  body: (command
    name: (command_name) @_cmd)
  redirect: (heredoc_redirect
    (heredoc_body) @injection.content))
  (#eq? @_cmd "zsh")
  (#set! injection.language "zsh"))

; fish <<'FISH'
((redirected_statement
  body: (command
    name: (command_name) @_cmd)
  redirect: (heredoc_redirect
    (heredoc_body) @injection.content))
  (#eq? @_cmd "fish")
  (#set! injection.language "fish"))

; psql <<'SQL'
((redirected_statement
  body: (command
    name: (command_name) @_cmd)
  redirect: (heredoc_redirect
    (heredoc_body) @injection.content))
  (#eq? @_cmd "psql")
  (#set! injection.language "sql-postgres"))

; mysql/mariadb <<'SQL'
((redirected_statement
  body: (command
    name: (command_name) @_cmd)
  redirect: (heredoc_redirect
    (heredoc_body) @injection.content))
  (#any-of? @_cmd "mysql" "mariadb")
  (#set! injection.language "sql-mysql"))

; sqlite3 <<'SQL'
((redirected_statement
  body: (command
    name: (command_name) @_cmd)
  redirect: (heredoc_redirect
    (heredoc_body) @injection.content))
  (#eq? @_cmd "sqlite3")
  (#set! injection.language "sql-sqlite"))

; env python <<'PY' / /usr/bin/env fish <<'FISH'
((redirected_statement
  body: (command
    name: (command_name) @_env
    argument: [
      (word)
      (string)
      (raw_string)
    ] @injection.language)
  redirect: (heredoc_redirect
    (heredoc_body) @injection.content))
  (#any-of? @_env "env" "/usr/bin/env"))
