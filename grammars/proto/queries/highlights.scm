[
  "syntax"
  "package"
  "option"
  "import"
  "service"
  "rpc"
  "returns"
  "message"
  "enum"
  "oneof"
  "repeated"
  "reserved"
  "to"
] @keyword

[
  (key_type)
  (type)
  (message_name)
  (enum_name)
  (service_name)
  (rpc_name)
]@type

(string) @string

; Extension names and option message fields carry the structure used by
; google.api.http, buf.validate, and other protobuf annotations.
(option (full_ident) @attribute)
(field_option (full_ident) @attribute)
(field_option (identifier) @property.proto)
(block_lit (identifier) @property.proto)

[
  (int_lit)
  (float_lit)
] @number

[
  (true)
  (false)
] @constant.builtin

(comment) @comment

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
]  @punctuation.bracket
