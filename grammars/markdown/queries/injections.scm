(fenced_code_block
  (info_string
    (language) @injection.language)
  (code_fence_content) @injection.content
  (#set! kat.visual "block"))

((html_block) @injection.content
  (#set! injection.include-children)
  (#set! injection.language "html")
  (#set! kat.visual "block"))

(document
  .
  (section
    .
    (thematic_break)
    (_) @injection.content
    (thematic_break))
  (#set! injection.language "yaml")
  (#set! kat.visual "block"))

((minus_metadata) @injection.content
  (#set! injection.language "yaml")
  (#set! kat.visual "block"))

((plus_metadata) @injection.content
  (#set! injection.language "toml")
  (#set! kat.visual "block"))

((inline) @injection.content
  (#set! injection.include-children)
  (#set! kat.merge-parent)
  (#set! injection.language "markdown_inline"))

((pipe_table_cell) @injection.content
  (#set! injection.include-children)
  (#set! kat.merge-parent)
  (#set! injection.language "markdown_inline"))
