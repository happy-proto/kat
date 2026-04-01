; Adapted from Helix's HCL query and tuned for kat's runtime.
; Sources:
; - https://github.com/helix-editor/helix/blob/035450a2de62142b4117c01b89fff3d4f1b4d51f/runtime/queries/hcl/highlights.scm
; - https://github.com/tree-sitter-grammars/tree-sitter-hcl

[
  "if"
  "else"
  "endif"
] @keyword.control.conditional

[
  "for"
  "endfor"
  "in"
] @keyword.control.repeat

[
  ":"
  "="
] @none

[
  (ellipsis)
  "?"
  "=>"
] @punctuation.special

[
  "."
  ".*"
  ","
  "[*]"
] @punctuation.delimiter

[
  "{"
  "}"
  "["
  "]"
  "("
  ")"
] @punctuation.bracket

[
  "!"
  "*"
  "/"
  "%"
  "+"
  "-"
  ">"
  ">="
  "<"
  "<="
  "=="
  "!="
  "&&"
  "||"
] @operator

(identifier) @variable

(object_elem
  val: (expression
    (variable_expr
      (identifier) @type.builtin
      (#match? @type.builtin "^(bool|string|number|object|tuple|list|map|set|any)$"))))

(get_attr
  (identifier) @variable.builtin
  (#match? @variable.builtin "^(root|cwd|module)$"))

(variable_expr
  (identifier) @variable.builtin
  (#match? @variable.builtin "^(var|local|path)$"))

((identifier) @type.builtin
 (#match? @type.builtin "^(bool|string|number|object|tuple|list|map|set|any)$"))

; TODO: Keep one generic HCL keyword set for now. If Terraform or Nomad later
; need more precise semantics, move those dialect-specific names into overlays
; instead of making the base HCL runtime ecosystem-specific.
((identifier) @keyword
 (#match? @keyword "^(job|group|task|service|template|network|volume|constraint|locals|variable|data|module|root|cwd|resource|terraform|provider|output)$"))

(object_elem key: (expression (variable_expr (identifier) @variable.other.member)))

(attribute (identifier) @variable.other.member)
(function_call (identifier) @function.method)
(block (identifier) @type.builtin)

(comment) @comment
(null_lit) @constant.builtin
(numeric_lit) @constant.numeric
(bool_lit) @constant.builtin.boolean

[
  (template_interpolation_start)
  (template_interpolation_end)
  (template_directive_start)
  (template_directive_end)
  (strip_marker)
] @punctuation.special

[
  (heredoc_identifier)
  (heredoc_start)
] @punctuation.delimiter

[
  (quoted_template_start)
  (quoted_template_end)
  (template_literal)
] @string
