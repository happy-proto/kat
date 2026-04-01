; Adapted from zed's regex highlights and extended with literal character styling.
; Sources:
; - https://github.com/zed-industries/zed/blob/main/crates/languages/src/regex/highlights.scm
; - https://github.com/tree-sitter/tree-sitter-regex/blob/master/queries/highlights.scm
; Licenses: Apache-2.0 (Zed), MIT (tree-sitter-regex)

[
  "("
  ")"
  "(?"
  "(?:"
  "(?<"
  "(?P<"
  "(?P="
  "<"
  ">"
  "["
  "]"
  "{"
  "}"
] @punctuation.bracket.regex

(group_name) @label.regex

[
  (identity_escape)
  (control_letter_escape)
  (character_class_escape)
  (control_escape)
  (unicode_character_escape)
] @string.escape.regex

[
  "*"
  "+"
  "?"
  "|"
  "="
  "!"
  (start_assertion)
  (end_assertion)
  (any_character)
  (lazy)
] @operator.regex

[
  (boundary_assertion)
  (non_boundary_assertion)
  (backreference_escape)
  (decimal_escape)
  (named_group_backreference)
] @keyword.operator.regex

(group_name) @label.regex

[
  (unicode_property_name)
  (unicode_property_value)
] @type.builtin

(posix_character_class
  "[:"
  @punctuation.bracket.regex
  (posix_class_name) @type.builtin
  ":]"
  @punctuation.bracket.regex)

(lookaround_assertion
  [
    "(?<"
    "(?"
  ] @punctuation.bracket.regex
  [
    "="
    "!"
  ] @operator.regex
  ")" @punctuation.bracket.regex)

(named_group_backreference
  (group_name) @label.regex)

(count_quantifier
  [
    "{" @punctuation.bracket.regex
    "}" @punctuation.bracket.regex
  ])

(count_quantifier
  [
    (decimal_digits) @number.quantifier.regex
    "," @punctuation.delimiter.regex
  ])

(inline_flags_group
  [
    "(?" @punctuation.bracket.regex
    "-" @operator.regex
    ":" @punctuation.delimiter.regex
    (flags) @keyword.operator.regex
    ")" @punctuation.bracket.regex
  ])

(character_class
  [
    "^" @operator.regex
    (class_range "-" @operator.regex)
  ])

[
  (class_character)
] @string.regex

(pattern_character) @string.regex
