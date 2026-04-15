(comment) @comment

(keyword_interface) @keyword.interface
(keyword_type) @keyword.type
(keyword_method) @keyword.method
(keyword_error) @keyword.error

(interface_name) @entity.interface
(method name: (_) @entity.method)
(error name: (_) @entity.error)
(typedef name: (_) @entity.type)
(typeref (name) @entity.type)
(struct_field name: (_) @entity.attribute-name)
(enum member: (_) @entity.enum-member)

[
    (bool)
    (int)
    (float)
    (string)
    (object)
    (any)
] @type.primitive

[
    "("
    ")"
    "["
    "]"
    ","
    ":"
] @punctuation

[
    (questionmark)
    (arrow)
] @operator
