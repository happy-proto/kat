; Kat-owned structural bracket pairs. Match siblings in the same AST node.
("(" @open ")" @close)
("[" @open "]" @close)
("{" @open "}" @close)
(command_substitution "$(" @open ")" @close)
(expansion "${" @open "}" @close)
(test_command "[[" @open "]]" @close)
(compound_statement "((" @open "))" @close)
