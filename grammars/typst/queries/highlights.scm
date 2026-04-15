(call
  item: (ident) @function)

(call
  item: (field
    field: (ident) @function.method))

(tagged
  field: (ident) @tag)

(field
  field: (ident) @tag)

(comment) @comment

(let "let" @keyword)
(branch ["if" "else"] @keyword)
(while "while" @keyword.repeat)
(for ["for" "in"] @keyword.repeat)
(import "import" @keyword.import)
(include "include" @keyword.import)
(show "show" @keyword)
(set "set" @keyword)
(return "return" @keyword.return)
(flow ["break" "continue"] @keyword)

(as "as" @keyword.operator)
(in ["in" "not"] @keyword.operator)
(context "context" @keyword)
(and "and" @keyword.operator)
(or "or" @keyword.operator)
(not "not" @keyword.operator)

(sign ["+" "-"] @operator)
(add "+" @operator)
(sub "-" @operator)
(mul "*" @operator)
(div "/" @operator)
(cmp ["==" "<=" ">=" "!=" "<" ">"] @operator)
(fraction "/" @operator)
(fac "!" @operator)
(attach ["^" "_"] @operator)
(wildcard) @operator
(align) @operator

(raw_blck) @text.literal.block
(raw_span) @text.literal.inline
(raw_blck
  lang: (ident) @tag)

(label) @tag
(ref) @text.reference
(url) @text.uri

(number) @number
(string) @string
(bool) @boolean
(none) @constant.builtin
(auto) @constant.builtin
(ident) @variable

(heading) @text.title

(item "-" @punctuation.list.markdown)
(term ["/" ":"] @punctuation.list.markdown)

[
  "="
  "=="
  "==="
  "===="
  "====="
  "======"
] @punctuation.special

(emph) @text.emphasis
(strong) @text.strong
(quote) @text.quote

(symbol) @constant
(letter) @constant
(shorthand) @constant.builtin
(linebreak) @constant.builtin

(math "$" @operator)
"#" @operator
"end" @operator

(escape) @string.escape

[
  "("
  ")"
  "{"
  "}"
] @punctuation.bracket

[
  ","
  ";"
  ".."
  ":"
  "sep"
] @punctuation.delimiter

"assign" @punctuation.delimiter

(field "." @punctuation.delimiter)
