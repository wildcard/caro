# Correctness Prompt v1.0

Generate a shell command that accomplishes the following task.

## Task
{{user_request}}

## Requirements
- Output only the command, no explanations
- Use POSIX-compliant syntax
- Prefer built-in commands over external tools
- Use safe flags (avoid destructive operations)

## Output Format
Return only the shell command as plain text.
