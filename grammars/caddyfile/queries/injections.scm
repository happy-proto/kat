((cel_expression) @injection.content
  (#set! injection.language "cel"))

(heredoc
  (heredoc_body) @injection.content
  (heredoc_end) @injection.language)

; Caddy examples conventionally use uppercase HTML heredoc markers.
((heredoc
  (heredoc_body) @injection.content
  (heredoc_end) @_end)
  (#eq? @_end "HTML")
  (#set! injection.language "html"))
