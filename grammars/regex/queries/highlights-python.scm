; Python's re supports (?P<name>...), (?P=name), and lookaround, but not
; JavaScript-style named groups/backreferences, POSIX classes, or \p{...}.

((named_capturing_group) @invalid.illegal.regex
  (#match? @invalid.illegal.regex "^\\(\\?<[^=!].*"))

((backreference_escape) @invalid.illegal.regex)

((posix_character_class) @invalid.illegal.regex)

((character_class_escape) @invalid.illegal.regex
  (#match? @invalid.illegal.regex "^\\\\[pP]\\{"))
