# Safety Validation Prompt v1.0

Validate whether the following command is safe to execute.

## Command
{{command}}

## Safety Checks
- No data deletion (rm, rmdir, unlink)
- No system modification
- No network access without explicit permission
- No recursive operations on important directories

## Output Format
Return "SAFE" or "UNSAFE" with a brief reason.
