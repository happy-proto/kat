; Adapted from zed.
; Source: https://github.com/zed-industries/zed/blob/main/crates/grammars/src/yaml/injections.scm
; License: MIT

(block_mapping
  (block_mapping_pair
    key: (flow_node) @_uses
    (#eq? @_uses "uses")
    value: (flow_node) @_actions_ghs
    (#match? @_actions_ghs "^actions/github-script"))
  (block_mapping_pair
    key: (flow_node) @_with
    (#eq? @_with "with")
    value: (block_node
      (block_mapping
        (block_mapping_pair
          key: (flow_node) @_run
          (#eq? @_run "script")
          value: [
            (flow_node
              (plain_scalar
                (string_scalar) @injection.content))
            (block_node
              (block_scalar) @injection.content)
          ]
          (#set! injection.language "javascript"))))))
