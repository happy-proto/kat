(line_comment) @comment @spell
(block_comment) @comment @spell

(command_name) @function @nospell

(caption
  command: _ @function)

(text) @spell

(text_mode
  command: _ @function @nospell
  content: (curly_group
    (_) @none @spell))

(placeholder) @variable.parameter

(key_value_pair
  key: (_) @variable.parameter @nospell)

(curly_group_spec
  (text) @variable.parameter)

(brack_group_argc) @variable.parameter

[
  (operator)
  "="
  "_"
  "^"
] @operator

"\\item" @punctuation.special

(delimiter) @punctuation.delimiter

(math_delimiter
  left_command: _ @punctuation.delimiter
  left_delimiter: _ @punctuation.delimiter
  right_command: _ @punctuation.delimiter
  right_delimiter: _ @punctuation.delimiter)

[
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

(begin
  command: _ @module
  name: (curly_group_text
    (text) @label @nospell))

(end
  command: _ @module
  name: (curly_group_text
    (text) @label @nospell))

(new_command_definition
  command: _ @function.macro @nospell)

(old_command_definition
  command: _ @function.macro @nospell)

(let_command_definition
  command: _ @function.macro @nospell)

(environment_definition
  command: _ @function.macro @nospell
  name: (curly_group_text
    (_) @label @nospell))

(theorem_definition
  command: _ @function.macro @nospell
  name: (curly_group_text_list
    (_) @label @nospell))

(paired_delimiter_definition
  command: _ @function.macro @nospell
  declaration: (curly_group_command_name
    (_) @function))

(label_definition
  command: _ @function.macro
  name: (curly_group_label
    (label) @text.reference @nospell))

(label_reference_range
  command: _ @function.macro
  from: (curly_group_label
    (label) @text.reference)
  to: (curly_group_label
    (label) @text.reference))

(label_reference
  command: _ @function.macro
  names: (curly_group_label_list
    (label) @text.reference))

(label_number
  command: _ @function.macro
  name: (curly_group_label
    (label) @text.reference)
  number: (_) @text.reference)

(citation
  command: _ @function.macro @nospell
  keys: (curly_group_text_list) @text.reference @nospell)

(hyperlink
  command: _ @function @nospell
  uri: (curly_group_uri
    (_) @text.uri @nospell))

(glossary_entry_definition
  command: _ @function.macro @nospell
  name: (curly_group_text
    (_) @text.reference @nospell))

(glossary_entry_reference
  command: _ @function.macro
  name: (curly_group_text
    (_) @text.reference))

(acronym_definition
  command: _ @function.macro @nospell
  name: (curly_group_text
    (_) @text.reference @nospell))

(acronym_reference
  command: _ @function.macro
  name: (curly_group_text
    (_) @text.reference))

(color_definition
  command: _ @function.macro
  name: (curly_group_text
    (_) @text.reference))

(color_reference
  command: _ @function.macro
  name: (curly_group_text
    (_) @text.reference)?)

(title_declaration
  command: _ @module
  options: (brack_group
    (_) @text.title)?
  text: (curly_group
    (_) @text.title))

(author_declaration
  command: _ @module
  authors: (curly_group_author_list
    (author)+ @text.title))

(chapter
  command: _ @module
  toc: (brack_group
    (_) @text.title)?
  text: (curly_group
    (_) @text.title))

(part
  command: _ @module
  toc: (brack_group
    (_) @text.title)?
  text: (curly_group
    (_) @text.title))

(section
  command: _ @module
  toc: (brack_group
    (_) @text.title)?
  text: (curly_group
    (_) @text.title))

(subsection
  command: _ @module
  toc: (brack_group
    (_) @text.title)?
  text: (curly_group
    (_) @text.title))

(subsubsection
  command: _ @module
  toc: (brack_group
    (_) @text.title)?
  text: (curly_group
    (_) @text.title))

(paragraph
  command: _ @module
  toc: (brack_group
    (_) @text.title)?
  text: (curly_group
    (_) @text.title))

(subparagraph
  command: _ @module
  toc: (brack_group
    (_) @text.title)?
  text: (curly_group
    (_) @text.title))

((generic_environment
  (begin
    name: (curly_group_text
      (text) @label)
    (#any-of? @label "frame"))
  .
  (curly_group
    (_) @text.title)))

((generic_command
  command: (command_name) @_name
  arg: (curly_group
    (_) @text.title))
  (#eq? @_name "\\frametitle"))

((generic_command
  command: (command_name) @_name
  arg: (curly_group
    (_) @text.emphasis))
  (#any-of? @_name "\\emph" "\\textit" "\\mathit"))

((generic_command
  command: (command_name) @_name
  arg: (curly_group
    (_) @text.strong))
  (#any-of? @_name "\\textbf" "\\mathbf"))

(generic_command
  (command_name) @keyword.conditional
  (#lua-match? @keyword.conditional "^\\\\if[a-zA-Z@]+$"))

(generic_command
  (command_name) @keyword.conditional
  (#any-of? @keyword.conditional "\\fi" "\\else"))

(class_include
  command: _ @keyword.import
  path: (curly_group_path) @string)

(package_include
  command: _ @keyword.import
  paths: (curly_group_path_list) @string)

(latex_include
  command: _ @keyword.import
  path: (curly_group_path) @string.special.path)
