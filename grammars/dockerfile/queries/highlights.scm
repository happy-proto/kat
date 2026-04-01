; Adapted from camdencheek/tree-sitter-dockerfile and tuned for kat.
; Source: https://github.com/camdencheek/tree-sitter-dockerfile
; License: MIT

[
  "FROM"
  "AS"
  "RUN"
  "CMD"
  "LABEL"
  "EXPOSE"
  "ENV"
  "ADD"
  "COPY"
  "ENTRYPOINT"
  "VOLUME"
  "USER"
  "WORKDIR"
  "ARG"
  "ONBUILD"
  "STOPSIGNAL"
  "HEALTHCHECK"
  "SHELL"
  "MAINTAINER"
  "CROSS_BUILD"
  (heredoc_marker)
  (heredoc_end)
] @keyword

(comment) @comment

(image_name) @string
(image_alias) @type

(arg_instruction
  name: (unquoted_string) @type)

(env_pair
  name: (unquoted_string) @type)

(label_pair
  key: [
    (unquoted_string)
    (double_quoted_string)
    (single_quoted_string)
  ] @type)

(expose_port) @number

(path) @string

[
  (double_quoted_string)
  (single_quoted_string)
  (json_string)
  (heredoc_line)
] @string

(json_string_command) @type

((json_string_command) @string
  (#match? @string "^\"(?:\\.?\\.?/|/|~/).*\"$"))

(command_json_string_array
  (json_string) @attribute
  (#match? @attribute "^\"-[^\"]*\"$"))

(escape_sequence) @string.escape

(param
  "--" @attribute
  name: (param_name) @attribute)

(mount_param
  "--" @attribute
  name: (param_name) @attribute)

(param_value) @string
(mount_param_key) @type
(mount_param_value_part) @string
(mount_param_flag) @constant
(mount_param_enum_value) @type
(mount_param_identifier_value) @property
(mount_param_string_value) @string
(mount_param_numeric_value) @number
(mount_param_boolean_value) @constant

(variable) @variable

(expansion
  [
    "$"
    "{"
    "}"
  ] @punctuation.special
) @none

[
  ":"
  "@"
  "="
] @operator

[
  "["
  "]"
] @punctuation.bracket

[
  ","
  (line_continuation)
] @punctuation.delimiter

((variable) @constant
  (#match? @constant "^[A-Z][A-Z_0-9]*$"))
