(int_literal) @number
(float_literal) @number.float
(bool_literal) @boolean

(attribute
  (ident) @attribute)

(struct_decl
  (ident) @type)

(struct_member
  (variable_ident_decl
    (ident) @variable.member))

(type_alias_decl
  (ident) @type)

(variable_ident_decl
  (ident) @variable.parameter
  (type_decl) @type)

(function_header
  (ident) @function)

(param
  (variable_ident_decl
    (ident) @variable.parameter
    (type_decl) @type))

(func_call_statement
  (ident) @function.call)

(callable
  (ident) @function.call)

[
  (bitcast)
  (const)
  (discard)
  (enable)
  (fallthrough)
  (let)
  (override)
  (type)
  (var)
] @keyword

(struct) @keyword.type
(fn) @keyword.function
(return) @keyword.return

[
  (private)
  (storage)
  (uniform)
  (workgroup)
  "read"
  "read_write"
  "write"
] @keyword.modifier

[
  (loop)
  (for)
  (while)
  (break)
  (continue)
  (continuing)
] @keyword.repeat

[
  (if)
  (else)
  (switch)
  (case)
  (default)
] @keyword.conditional

[
  (and)
  (and_and)
  (forward_slash)
  (bang)
  (equal)
  (equal_equal)
  (not_equal)
  (greater_than)
  (greater_than_equal)
  (shift_right)
  (less_than)
  (less_than_equal)
  (shift_left)
  (modulo)
  (minus)
  (plus)
  (or)
  (or_or)
  (star)
  (tilde)
  (xor)
  (attr)
  (plus_plus)
  (minus_minus)
] @operator

[
  (comma)
  (period)
  (colon)
  (semicolon)
  (arrow)
] @punctuation.delimiter

[
  (paren_left)
  (paren_right)
  (bracket_left)
  (bracket_right)
  (brace_left)
  (brace_right)
] @punctuation.bracket
