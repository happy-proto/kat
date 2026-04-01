; regex crate does not support lookaround or backreferences, and uses
; Python-style named groups instead of JavaScript-style (?<name>...).

((lookaround_assertion) @invalid.illegal.regex)

((backreference_escape) @invalid.illegal.regex)

((named_group_backreference) @invalid.illegal.regex)

((decimal_escape) @invalid.illegal.regex)

((named_capturing_group) @invalid.illegal.regex
  (#match? @invalid.illegal.regex "^\\(\\?<[^=!].*"))
