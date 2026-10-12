;; Locals for the fsharp_signature grammar (.fsi files). A signature has no
;; expression layer, so the only scopes are namespaces and modules and the
;; only definitions are the values, functions and types they declare.

(identifier) @local.reference

[
  (namespace)
  (named_module)
  (module_defn)
] @local.scope

(value_definition
  (value_declaration_left
    . (identifier_pattern
        (long_identifier_or_op
          (identifier) @local.definition))))

(type_name type_name: (long_identifier (identifier) @local.definition))
(type_name type_name: (identifier) @local.definition)
