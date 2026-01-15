"""
Function schemas for CLI tool recommendation.

These schemas define the function-calling interface that FunctionGemma
will be fine-tuned to use for recommending CLI tools.
"""

from typing import List, Optional, Dict, Any, Literal

# OS Types supported
OS_TYPES = Literal[
    "posix",      # Generic POSIX
    "linux",      # Linux (generic)
    "darwin",     # macOS
    "ubuntu",     # Ubuntu GNU/Linux
    "debian",     # Debian GNU/Linux
    "fedora",     # Fedora
    "arch",       # Arch Linux
    "alpine",     # Alpine Linux
    "bsd",        # BSD variants
    "freebsd",    # FreeBSD
    "openbsd",    # OpenBSD
    "windows",    # Windows (PowerShell/CMD)
]

# Shell Types supported
SHELL_TYPES = Literal[
    "sh",         # Bourne shell (POSIX)
    "bash",       # Bash
    "zsh",        # Z shell
    "fish",       # Fish shell
    "dash",       # Debian Almquist shell
    "ksh",        # Korn shell
    "tcsh",       # TENEX C shell
    "pwsh",       # PowerShell Core
    "cmd",        # Windows CMD
]

# Tool categories
TOOL_CATEGORIES = Literal[
    "file_management",      # ls, find, locate, tree, fd
    "text_processing",      # grep, sed, awk, cut, sort, uniq
    "file_viewing",         # cat, less, head, tail, bat
    "search",               # grep, rg, ag, ack, fzf
    "archive",              # tar, zip, unzip, gzip, 7z
    "network",              # curl, wget, ssh, scp, rsync
    "process",              # ps, top, htop, kill, pgrep
    "disk",                 # df, du, ncdu, duf
    "package_manager",      # apt, brew, dnf, pacman, npm, pip
    "version_control",      # git, gh, svn, hg
    "development",          # make, cmake, gcc, rustc, node
    "containers",           # docker, podman, kubectl, k9s
    "json_processing",      # jq, yq, fx
    "system_info",          # uname, hostname, whoami, id
    "permissions",          # chmod, chown, chgrp, umask
    "editor",               # vim, nvim, nano, emacs, code
    "multiplexer",          # tmux, screen, zellij
    "shell_utils",          # alias, export, source, which
]


# Function Schema Definitions (FunctionGemma format)
FUNCTION_SCHEMAS = {
    "recommend_tools": {
        "name": "recommend_tools",
        "description": "Recommend CLI tools for a given task based on user's OS, shell, and preferences",
        "parameters": {
            "type": "object",
            "properties": {
                "primary_tools": {
                    "type": "array",
                    "description": "Primary recommended tools that are most likely installed",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string", "description": "Tool name"},
                            "category": {"type": "string", "description": "Tool category"},
                            "confidence": {"type": "number", "description": "Confidence 0.0-1.0 that this tool is installed"},
                            "version_hint": {"type": "string", "description": "Expected version range"},
                            "reason": {"type": "string", "description": "Why this tool is recommended"},
                        },
                        "required": ["name", "category", "confidence", "reason"]
                    }
                },
                "alternative_tools": {
                    "type": "array",
                    "description": "Alternative tools that may provide better functionality but might not be installed",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string", "description": "Tool name"},
                            "category": {"type": "string", "description": "Tool category"},
                            "install_cmd": {"type": "string", "description": "Command to install this tool"},
                            "reason": {"type": "string", "description": "Why this is a better alternative"},
                            "improvements": {"type": "array", "items": {"type": "string"}, "description": "Specific improvements over primary tools"},
                        },
                        "required": ["name", "category", "install_cmd", "reason"]
                    }
                },
                "task_category": {
                    "type": "string",
                    "description": "The primary category of the user's task"
                }
            },
            "required": ["primary_tools", "task_category"]
        }
    },

    "check_tool_availability": {
        "name": "check_tool_availability",
        "description": "Check if specific tools are available on the system",
        "parameters": {
            "type": "object",
            "properties": {
                "tools": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "List of tool names to check"
                },
                "os_type": {
                    "type": "string",
                    "description": "Operating system type"
                },
                "shell_type": {
                    "type": "string",
                    "description": "Shell type"
                }
            },
            "required": ["tools", "os_type"]
        }
    },

    "get_install_command": {
        "name": "get_install_command",
        "description": "Get the installation command for a tool on a specific OS",
        "parameters": {
            "type": "object",
            "properties": {
                "tool_name": {
                    "type": "string",
                    "description": "Name of the tool to install"
                },
                "os_type": {
                    "type": "string",
                    "description": "Operating system type"
                },
                "package_manager": {
                    "type": "string",
                    "description": "Preferred package manager (optional)"
                }
            },
            "required": ["tool_name", "os_type"]
        }
    }
}


def format_function_declaration(schema: Dict[str, Any]) -> str:
    """Format a function schema as FunctionGemma declaration."""
    import json
    return f"<start_function_declaration>declaration:{schema['name']}{json.dumps(schema, indent=2)}"


def format_function_call(func_name: str, args: Dict[str, Any]) -> str:
    """Format a function call in FunctionGemma format."""
    import json
    args_str = json.dumps(args, indent=2)
    return f"<start_function_call>call:{func_name}{args_str}<end_function_call>"


def get_all_declarations() -> str:
    """Get all function declarations formatted for FunctionGemma."""
    declarations = []
    for schema in FUNCTION_SCHEMAS.values():
        declarations.append(format_function_declaration(schema))
    return "\n".join(declarations)
