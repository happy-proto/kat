; JavaScript RegExp supports lookaround, \p{...}, \k<name>, and (?<name>...),
; but not Python-specific named capture / backreference syntax or POSIX classes.

((named_capturing_group) @invalid.illegal.regex
  (#match? @invalid.illegal.regex "^\\(\\?P<"))

((named_group_backreference) @invalid.illegal.regex)

((posix_character_class) @invalid.illegal.regex)

((inline_flags_group) @invalid.illegal.regex)
