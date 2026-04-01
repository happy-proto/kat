(inline) @text

[
  (code_span)
] @text.literal.inline

(link_title) @text.literal

[
  (emphasis_delimiter)
  (code_span_delimiter)
] @punctuation.delimiter.markdown

(emphasis) @text.emphasis

(strong_emphasis) @text.strong

(strikethrough) @text.strikethrough

[
  (link_destination)
  (uri_autolink)
  (email_autolink)
] @text.uri

[
  (full_reference_link)
  (collapsed_reference_link)
  (shortcut_link)
  (link_label)
  (link_text)
  (image_description)
] @text.reference

[
  (backslash_escape)
  (hard_line_break)
] @string.escape

(image
  [
    "!"
    "["
    "]"
    "("
    ")"
  ] @punctuation.delimiter.markdown)

(inline_link
  [
    "["
    "]"
    "("
    ")"
  ] @punctuation.delimiter.markdown)

(shortcut_link
  [
    "["
    "]"
  ] @punctuation.delimiter.markdown)

(full_reference_link
  [
    "["
    "]"
  ] @punctuation.delimiter.markdown)

(collapsed_reference_link
  [
    "["
    "]"
  ] @punctuation.delimiter.markdown)

; NOTE: extension not enabled by default
; (wiki_link ["[" "|" "]"] @punctuation.delimiter)
