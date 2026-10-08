; Adapted from apple/tree-sitter-pkl (Apache-2.0).
; Generic captures precede structural roles, which take precedence in kat.
(identifier) @variable

; Types

(clazz (identifier) @type)
(typeAlias (identifier) @type)
(declaredType (qualifiedIdentifier (identifier) @type))
(classExtendsClause (qualifiedIdentifier (identifier) @type))

(typeArgumentList
  "<" @punctuation.bracket
  ">" @punctuation.bracket)

; Method definitions

(classMethod (methodHeader (identifier) @function.method))
(objectMethod (methodHeader (identifier) @function.method))

; Identifiers

(classProperty (identifier) @property.pkl)
(objectProperty (identifier) @property.pkl)

(parameterList (typedIdentifier (identifier) @variable.parameter))
(objectBodyParameters (typedIdentifier (identifier) @variable.parameter))

(unqualifiedAccessExpr (identifier) @function.call (argumentList))
(qualifiedAccessExpr (identifier) @property)
(qualifiedAccessExpr (identifier) @function.method (argumentList))

; Literals

(stringConstant) @string
(slStringLiteralPart) @string
(mlStringLiteralPart) @string
(slStringLiteralExpr ["\"" "#\"" "##\"" "###\"" "####\"" "#####\"" "######\""
  "\"#" "\"##" "\"###" "\"####" "\"#####" "\"######"] @string)
(mlStringLiteralExpr ["\"\"\"" "#\"\"\"" "##\"\"\"" "###\"\"\"" "####\"\"\"" "#####\"\"\"" "######\"\"\""
  "\"\"\"#" "\"\"\"##" "\"\"\"###" "\"\"\"####" "\"\"\"#####" "\"\"\"######"] @string)

(escapeSequence) @string.escape

(intLiteralExpr) @number
(floatLiteralExpr) @number

(stringInterpolation
  "\\(" @punctuation.special
  ")" @punctuation.special)

(stringInterpolation
 "\\#(" @punctuation.special
 ")" @punctuation.special)

(stringInterpolation
  "\\##(" @punctuation.special
  ")" @punctuation.special)

(lineComment) @comment
(blockComment) @comment
(docComment) @comment
(shebangComment) @comment

; Operators

"??" @operator
"!!" @operator
"->" @operator
"..." @operator
"...?" @operator
"@"  @operator
"="  @operator
"<"  @operator
">"  @operator
"!"  @operator
"==" @operator
"!=" @operator
"<=" @operator
">=" @operator
"&&" @operator
"||" @operator
"+"  @operator
"-"  @operator
"**" @operator
"*"  @operator
"/"  @operator
"~/" @operator
"%"  @operator
"|>" @operator

"," @punctuation.delimiter
":" @punctuation.delimiter
"." @punctuation.delimiter
"?." @punctuation.delimiter

"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket

; Keywords

"abstract" @keyword
"amends" @keyword
"as" @keyword
"class" @keyword
"const" @keyword
"else" @keyword
"extends" @keyword
"external" @keyword
"fixed" @keyword
(falseLiteralExpr) @constant.builtin
"for" @keyword
"function" @keyword
"hidden" @keyword
"if" @keyword
(importExpr "import" @function.builtin)
(importExpr "import*" @function.builtin)
"import" @keyword
"import*" @keyword
"in" @keyword
"is" @keyword
"let" @keyword
"local" @keyword
(moduleExpr "module" @type.builtin)
(thisType) @type.builtin
"module" @keyword
"new" @keyword
(nullLiteralExpr) @constant.builtin
"open" @keyword
"out" @keyword
(outerExpr) @variable.builtin
"read" @function.builtin
"read?" @function.builtin
"read*" @function.builtin
"super" @variable.builtin
(thisExpr) @variable.builtin
"throw" @function.builtin
"trace" @function.builtin
(trueLiteralExpr) @constant.builtin
"typealias" @keyword
"when" @keyword

; kat: cover the remaining custom string delimiter counts.
(stringInterpolation ["\\###(" "\\####(" "\\#####(" "\\######("] @punctuation.special ")" @punctuation.special)
