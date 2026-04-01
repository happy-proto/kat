; POSIX-like regex hosts should treat modern PCRE/Perl-style extensions as
; invalid while still allowing POSIX character classes.

((lookaround_assertion) @invalid.illegal.regex)

((backreference_escape) @invalid.illegal.regex)

((named_group_backreference) @invalid.illegal.regex)

((named_capturing_group) @invalid.illegal.regex)

((non_capturing_group) @invalid.illegal.regex)

((inline_flags_group) @invalid.illegal.regex)

((lazy) @invalid.illegal.regex)

((character_class_escape) @invalid.illegal.regex
  (#match? @invalid.illegal.regex "^\\\\[pP]\\{"))
