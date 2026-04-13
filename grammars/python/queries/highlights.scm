; Adapted from zed's Python highlights with adjustments for the current
; tree-sitter-python grammar revision used by kat.
; Source: https://github.com/zed-industries/zed/blob/main/crates/grammars/src/python/highlights.scm
; License: MIT

; Soft naming conventions should remain near the top because later rules often
; intentionally override them.
(identifier) @variable

(attribute
  attribute: (identifier) @property)

((identifier) @type.class
  (#match? @type.class "^_*[A-Z][A-Za-z0-9_]*$"))

((identifier) @constant
  (#match? @constant "^_*[A-Z][A-Z0-9_]*$"))

(type
  (identifier) @type)

(comment) @comment

(string) @string

(escape_sequence) @string.escape

; Forward references inside annotations are effectively type names.
(type
  (string) @type)

; Function and method calls.
(call
  function: (attribute
    attribute: (identifier) @function.method.call))

(call
  function: (identifier) @function.call)

; Decorators.
(decorator
  "@" @punctuation.special)

(decorator
  (identifier) @function.decorator)

(decorator
  (attribute
    attribute: (identifier) @function.decorator))

(decorator
  (call
    function: (identifier) @function.decorator.call))

(decorator
  (call
    function: (attribute
      attribute: (identifier) @function.decorator.call)))

; Function and class definitions.
(function_definition
  name: (identifier) @function)

(class_definition
  name: (identifier) @type.class.definition)

(class_definition
  superclasses: (argument_list
    (identifier) @type.class.inheritance))

((call
  function: (identifier) @type.class.call)
  (#match? @type.class.call "^_*[A-Z][A-Za-z0-9_]*$"))

((call
  function: (identifier) @_isinstance
  arguments: (argument_list
    (_)
    (identifier) @type))
  (#eq? @_isinstance "isinstance"))

((call
  function: (identifier) @_issubclass
  arguments: (argument_list
    (identifier) @type
    (identifier) @type))
  (#eq? @_issubclass "issubclass"))

; Parameters and keyword arguments.
(function_definition
  parameters: (parameters
    [
      (identifier) @variable.parameter
      (typed_parameter
        (identifier) @variable.parameter)
      (default_parameter
        name: (identifier) @variable.parameter)
      (typed_default_parameter
        name: (identifier) @variable.parameter)
    ]))

(call
  arguments: (argument_list
    (keyword_argument
      name: (identifier) @function.kwargs)))

; Builtins and builtin types.
((call
  function: (identifier) @function.builtin)
  (#any-of? @function.builtin
    "abs" "all" "any" "ascii" "bin" "bool" "breakpoint" "bytearray" "bytes" "callable" "chr"
    "classmethod" "compile" "complex" "delattr" "dict" "dir" "divmod" "enumerate" "eval" "exec"
    "filter" "float" "format" "frozenset" "getattr" "globals" "hasattr" "hash" "help" "hex" "id"
    "input" "int" "isinstance" "issubclass" "iter" "len" "list" "locals" "map" "max" "memoryview"
    "min" "next" "object" "oct" "open" "ord" "pow" "print" "property" "range" "repr" "reversed"
    "round" "set" "setattr" "slice" "sorted" "staticmethod" "str" "sum" "super" "tuple" "type"
    "vars" "zip" "__import__"))

[
  (call
    function: (identifier) @type.builtin)
  (type
    (identifier) @type.builtin)
  (#any-of? @type.builtin
    "bool" "bytearray" "bytes" "complex" "dict" "float" "frozenset" "int" "list" "memoryview"
    "object" "range" "set" "slice" "str" "tuple")
]

; Self / cls references are visually distinct from regular variables.
[
  (parameters
    (identifier) @variable.special)
  (attribute
    (identifier) @variable.special)
  (#any-of? @variable.special "self" "cls")
]

; Literals.
[
  (true)
  (false)
] @boolean

[
  (none)
  (ellipsis)
] @constant.builtin

[
  (integer)
  (float)
] @number

; Docstrings.
([
  (expression_statement
    (assignment))
]
  .
  (expression_statement
    (string) @string.doc)+)

(module
  .
  (expression_statement
    (string) @string.doc)+)

(class_definition
  body: (block
    .
    (comment) @comment*
    .
    (expression_statement
      (string) @string.doc)+))

(function_definition
  "async"?
  "def"
  name: (_)
  (parameters)?
  body: (block
    .
    (expression_statement
      (string) @string.doc)+))

(class_definition
  body: (block
    (function_definition
      name: (identifier) @function.method.constructor
      (#eq? @function.method.constructor "__init__")
      body: (block
        (expression_statement
          (assignment))
        .
        (expression_statement
          (string) @string.doc)+))))

(interpolation
  "{" @punctuation.special
  "}" @punctuation.special) @embedded

[
  "."
  ","
  ":"
] @punctuation.delimiter

[
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket

[
  "-"
  "-="
  "!="
  "*"
  "**"
  "**="
  "*="
  "/"
  "//"
  "//="
  "/="
  "&"
  "%"
  "%="
  "@"
  "^"
  "+"
  "->"
  "+="
  "<"
  "<<"
  "<="
  "<>"
  "="
  ":="
  "=="
  ">"
  ">="
  ">>"
  "|"
  "~"
  "&="
  "<<="
  ">>="
  "@="
  "^="
  "|="
] @operator

[
  "and"
  "in"
  "is"
  "not"
  "or"
] @keyword.operator

[
  "as"
  "assert"
  "async"
  "await"
  "break"
  "class"
  "continue"
  "def"
  "del"
  "elif"
  "else"
  "except"
  "exec"
  "finally"
  "for"
  "from"
  "global"
  "if"
  "import"
  "lambda"
  "nonlocal"
  "pass"
  "print"
  "raise"
  "return"
  "try"
  "while"
  "with"
  "yield"
  "match"
  "case"
] @keyword

[
  "async"
  "def"
  "class"
  "lambda"
] @keyword.definition

(decorator
  (identifier) @attribute.builtin
  (#any-of? @attribute.builtin "classmethod" "staticmethod" "property"))
