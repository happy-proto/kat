(shell_instruction
  (command_json_string_array
    (json_string_command) @shell.language))

[
  (run_instruction
    (shell_command) @shell.content)
  (cmd_instruction
    (shell_command) @shell.content)
  (entrypoint_instruction
    (shell_command) @shell.content)
  (healthcheck_instruction
    (cmd_instruction
      (shell_command) @shell.content))
  (run_instruction
    (heredoc_block
      (heredoc_line) @shell.content))
]
