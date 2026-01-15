#!/usr/bin/env python3
"""
Dataset generation utilities for CLI Tool Recommender fine-tuning.

This script generates diverse training examples by:
1. Loading the tools knowledge base
2. Generating query variations for each task category
3. Creating context variations (OS, shell, preferences)
4. Producing properly formatted training data

Usage:
    python generate_dataset.py --output ./data/training_data.json --num_examples 1000
"""

import argparse
import json
import random
from pathlib import Path
from typing import Dict, List, Any, Tuple, Optional
from dataclasses import dataclass, asdict
import itertools


@dataclass
class TrainingContext:
    """Training example context."""
    os: str
    shell: str
    prefer_modern_tools: bool = False
    network_enabled: bool = True


@dataclass
class ToolInfo:
    """Information about a CLI tool."""
    name: str
    category: str
    confidence: float
    reason: str
    version_hint: Optional[str] = None
    install_cmd: Optional[str] = None
    improvements: Optional[List[str]] = None


# Query templates for different task categories
QUERY_TEMPLATES = {
    "file_management": [
        "find all {filetype} files in {location}",
        "list files in {location}",
        "show directory structure of {location}",
        "locate all files named {pattern}",
        "find files modified in the last {timeframe}",
        "search for files larger than {size}",
        "recursively list all {filetype} files",
        "show hidden files in {location}",
        "find empty directories",
        "list files sorted by {sort_criteria}",
    ],
    "search": [
        "search for '{pattern}' in all files",
        "find occurrences of '{pattern}' in {location}",
        "grep for '{pattern}' recursively",
        "search code for '{pattern}'",
        "find files containing '{pattern}'",
        "search and replace '{pattern}' with '{replacement}'",
        "case-insensitive search for '{pattern}'",
        "find lines matching regex '{pattern}'",
        "count occurrences of '{pattern}'",
        "search in {filetype} files only",
    ],
    "text_processing": [
        "replace '{old}' with '{new}' in {file}",
        "extract column {column} from {file}",
        "sort lines in {file}",
        "remove duplicate lines from {file}",
        "count lines, words, and characters in {file}",
        "filter lines containing '{pattern}'",
        "transform text to uppercase/lowercase",
        "merge multiple files into one",
        "split file by delimiter",
        "extract specific fields from CSV",
    ],
    "file_viewing": [
        "view contents of {file}",
        "show first {n} lines of {file}",
        "show last {n} lines of {file}",
        "view file with syntax highlighting",
        "follow log file in real-time",
        "page through a large file",
        "view binary file as hex",
        "compare two files side by side",
        "show line numbers when viewing",
        "view specific line range",
    ],
    "network": [
        "download file from {url}",
        "make HTTP {method} request to {url}",
        "send JSON data to API endpoint",
        "check if {host} is reachable",
        "transfer file to remote server",
        "sync directory with remote",
        "connect to remote server via SSH",
        "test API endpoint",
        "download with resume support",
        "fetch web page content",
    ],
    "process": [
        "show running processes",
        "find process by name",
        "kill process with PID {pid}",
        "monitor system resources",
        "view CPU usage",
        "show memory usage",
        "find process using port {port}",
        "list all background jobs",
        "check process tree",
        "monitor specific process",
    ],
    "disk": [
        "show disk space usage",
        "find large files in {location}",
        "analyze directory size",
        "show filesystem usage",
        "find what's using disk space",
        "check available space on disk",
        "show size of each subdirectory",
        "identify largest files",
        "clean up disk space",
        "monitor disk I/O",
    ],
    "archive": [
        "compress {location} to archive",
        "extract archive to {destination}",
        "create tar.gz archive",
        "create zip file from {location}",
        "list contents of archive",
        "extract specific file from archive",
        "compress with best ratio",
        "create encrypted archive",
        "split large archive",
        "verify archive integrity",
    ],
    "version_control": [
        "clone repository from {url}",
        "check git status",
        "commit changes with message",
        "create new branch",
        "merge branches",
        "view commit history",
        "show diff between commits",
        "push changes to remote",
        "pull latest changes",
        "stash current changes",
    ],
    "json_processing": [
        "parse JSON from {source}",
        "extract '{field}' from JSON",
        "filter JSON array",
        "format JSON output",
        "validate JSON structure",
        "transform JSON data",
        "merge JSON files",
        "query nested JSON",
        "convert JSON to CSV",
        "pretty print JSON",
    ],
    "permissions": [
        "change file permissions to {mode}",
        "make file executable",
        "change file owner to {user}",
        "recursively change permissions",
        "view current permissions",
        "set default permissions",
        "change group ownership",
        "remove write permission",
        "add read permission for all",
        "copy permissions from another file",
    ],
}

# Variables to fill in templates
TEMPLATE_VARIABLES = {
    "filetype": ["python", "javascript", "rust", "go", "java", "c", "json", "yaml", "markdown", "text", "log", "config"],
    "location": ["current directory", "home directory", "project folder", ".", "~/", "/var/log", "src/"],
    "pattern": ["error", "TODO", "FIXME", "import", "function", "class", "def ", "const ", "var ", "let "],
    "timeframe": ["24 hours", "7 days", "30 days", "1 hour", "yesterday"],
    "size": ["100MB", "1GB", "10MB", "500KB"],
    "sort_criteria": ["size", "date", "name", "type"],
    "file": ["config.json", "app.log", "data.csv", "README.md", "output.txt"],
    "old": ["foo", "bar", "old_name", "deprecated", "v1"],
    "new": ["baz", "qux", "new_name", "current", "v2"],
    "replacement": ["baz", "updated", "fixed"],
    "column": ["1", "2", "3", "first", "last"],
    "n": ["10", "20", "50", "100"],
    "url": ["https://example.com/file.zip", "https://api.example.com/data"],
    "method": ["GET", "POST", "PUT", "DELETE"],
    "host": ["google.com", "localhost", "192.168.1.1"],
    "pid": ["1234", "5678"],
    "port": ["8080", "3000", "443", "80"],
    "destination": ["./extracted", "~/Downloads"],
    "field": ["name", "id", "status", "data"],
    "mode": ["755", "644", "777", "600"],
    "user": ["root", "www-data", "nobody"],
    "source": ["response.json", "api output", "stdin"],
}

# OS and shell combinations
OS_TYPES = ["darwin", "ubuntu", "linux", "arch", "fedora", "debian", "bsd", "windows"]
SHELLS = ["bash", "zsh", "fish", "sh", "dash", "pwsh"]


def load_tools_kb(kb_path: str) -> Dict[str, Any]:
    """Load the CLI tools knowledge base."""
    with open(kb_path, 'r') as f:
        return json.load(f)


def fill_template(template: str) -> str:
    """Fill a template string with random variables."""
    result = template

    for var_name, var_values in TEMPLATE_VARIABLES.items():
        placeholder = "{" + var_name + "}"
        if placeholder in result:
            result = result.replace(placeholder, random.choice(var_values), 1)

    return result


def generate_query(category: str) -> str:
    """Generate a random query for a category."""
    templates = QUERY_TEMPLATES.get(category, QUERY_TEMPLATES["file_management"])
    template = random.choice(templates)
    return fill_template(template)


def generate_context() -> TrainingContext:
    """Generate a random training context."""
    return TrainingContext(
        os=random.choice(OS_TYPES),
        shell=random.choice(SHELLS),
        prefer_modern_tools=random.random() > 0.5,
        network_enabled=random.random() > 0.2,
    )


def get_tools_for_category(
    tools_kb: Dict[str, Any],
    category: str,
    context: TrainingContext
) -> Tuple[List[ToolInfo], List[ToolInfo]]:
    """Get primary and alternative tools for a category and context."""
    primary = []
    alternatives = []

    tools = tools_kb.get("tools", {})

    for tool_name, tool_data in tools.items():
        if tool_data.get("category") != category:
            continue

        availability = tool_data.get("availability", {})
        os_info = availability.get(context.os, availability.get("linux", {}))

        confidence = os_info.get("confidence", 0.5)
        is_default = os_info.get("installed_by_default", False)

        if is_default or confidence >= 0.7:
            primary.append(ToolInfo(
                name=tool_name,
                category=category,
                confidence=confidence,
                reason=tool_data.get("description", "Standard tool for this task"),
                version_hint=os_info.get("version"),
            ))
        elif tool_data.get("improvements_over"):
            # This is a modern alternative
            install_cmd = os_info.get("install_cmd", "")
            if not context.network_enabled and not install_cmd:
                continue

            alternatives.append(ToolInfo(
                name=tool_name,
                category=category,
                confidence=confidence,
                reason=tool_data.get("description", "Modern alternative"),
                install_cmd=install_cmd,
                improvements=tool_data.get("improvements", []),
            ))

    return primary[:3], alternatives[:2]  # Limit to reasonable numbers


def generate_thinking(
    query: str,
    context: TrainingContext,
    primary_tools: List[ToolInfo],
    alternative_tools: List[ToolInfo]
) -> str:
    """Generate reasoning text for the model response."""
    thinking_parts = []

    # Analyze the query
    thinking_parts.append(f"The user wants to {query.lower()}. On {context.os} with {context.shell}:")

    # Explain primary tools
    for i, tool in enumerate(primary_tools, 1):
        conf = int(tool.confidence * 100)
        thinking_parts.append(
            f"{i}. `{tool.name}` is {'installed by default' if conf >= 90 else 'likely available'} "
            f"({conf}% confidence)"
        )

    # Mention alternatives if applicable
    if alternative_tools and context.prefer_modern_tools:
        thinking_parts.append("\nSince they prefer modern tools:")
        for tool in alternative_tools:
            thinking_parts.append(f"- `{tool.name}` would be a better choice but requires installation")

    if not context.network_enabled and alternative_tools:
        thinking_parts.append("\nNetwork is disabled, so focusing on installed tools only.")

    return "\n".join(thinking_parts)


def generate_function_call(
    primary_tools: List[ToolInfo],
    alternative_tools: List[ToolInfo],
    category: str
) -> str:
    """Generate the function call JSON."""
    call_data = {
        "primary_tools": [
            {
                "name": t.name,
                "category": t.category,
                "confidence": t.confidence,
                "reason": t.reason,
                **({"version_hint": t.version_hint} if t.version_hint else {})
            }
            for t in primary_tools
        ],
        "task_category": category
    }

    if alternative_tools:
        call_data["alternative_tools"] = [
            {
                "name": t.name,
                "category": t.category,
                "install_cmd": t.install_cmd or "",
                "reason": t.reason,
                **({"improvements": t.improvements} if t.improvements else {})
            }
            for t in alternative_tools
        ]

    return json.dumps(call_data, indent=2)


def generate_training_example(
    tools_kb: Dict[str, Any],
    example_id: int
) -> Dict[str, Any]:
    """Generate a single training example."""
    # Pick random category
    category = random.choice(list(QUERY_TEMPLATES.keys()))

    # Generate query and context
    query = generate_query(category)
    context = generate_context()

    # Get relevant tools
    primary_tools, alternative_tools = get_tools_for_category(tools_kb, category, context)

    # If no primary tools found, fall back to generic ones
    if not primary_tools:
        primary_tools = [ToolInfo(
            name="help",
            category=category,
            confidence=0.5,
            reason="Generic help available",
        )]

    # Generate thinking and function call
    thinking = generate_thinking(query, context, primary_tools, alternative_tools)
    function_call = generate_function_call(primary_tools, alternative_tools, category)

    # Build the model response
    model_response = f"<think>\n{thinking}\n</think>\n"
    model_response += f"<start_function_call>call:recommend_tools{function_call}<end_function_call>"

    # Build context dict for preferences
    prefs = {}
    if context.prefer_modern_tools:
        prefs["prefer_modern_tools"] = True
    if not context.network_enabled:
        prefs["network_enabled"] = False

    return {
        "id": f"{example_id:04d}",
        "context": {
            "os": context.os,
            "shell": context.shell,
            "user_preferences": prefs
        },
        "user_query": query,
        "conversation": [
            {
                "role": "developer",
                "content": (
                    "You are a CLI tool recommendation assistant. Given the user's query, "
                    "OS, shell, and preferences, recommend the most appropriate CLI tools. "
                    "Always prefer tools that are likely installed by default, but suggest "
                    "modern alternatives when beneficial. Format your response as a function "
                    "call to recommend_tools."
                )
            },
            {
                "role": "user",
                "content": f"OS: {context.os}, Shell: {context.shell}, "
                          f"Preferences: {', '.join(prefs.keys()) if prefs else 'none'}\n"
                          f"Query: {query}"
            },
            {
                "role": "model",
                "content": model_response
            }
        ]
    }


def generate_dataset(
    tools_kb: Dict[str, Any],
    num_examples: int,
    seed: int = 42
) -> Dict[str, Any]:
    """Generate a complete training dataset."""
    random.seed(seed)

    examples = []
    for i in range(num_examples):
        example = generate_training_example(tools_kb, i + 1)
        examples.append(example)

        if (i + 1) % 100 == 0:
            print(f"Generated {i + 1}/{num_examples} examples")

    return {
        "metadata": {
            "version": "1.0.0",
            "description": "Auto-generated training data for FunctionGemma CLI tool recommendation",
            "num_examples": num_examples,
            "seed": seed
        },
        "examples": examples
    }


def main():
    parser = argparse.ArgumentParser(
        description="Generate training dataset for CLI Tool Recommender"
    )
    parser.add_argument(
        "--output",
        type=str,
        default="./data/training_data.json",
        help="Output path for training data"
    )
    parser.add_argument(
        "--kb_path",
        type=str,
        default="./tools_kb/cli_tools.json",
        help="Path to CLI tools knowledge base"
    )
    parser.add_argument(
        "--num_examples",
        type=int,
        default=500,
        help="Number of training examples to generate"
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Random seed for reproducibility"
    )

    args = parser.parse_args()

    # Resolve paths relative to script directory
    script_dir = Path(__file__).parent.parent
    kb_path = script_dir / args.kb_path if not Path(args.kb_path).is_absolute() else Path(args.kb_path)
    output_path = script_dir / args.output if not Path(args.output).is_absolute() else Path(args.output)

    print(f"Loading tools knowledge base from: {kb_path}")
    tools_kb = load_tools_kb(str(kb_path))
    print(f"Loaded {len(tools_kb.get('tools', {}))} tools")

    print(f"\nGenerating {args.num_examples} training examples...")
    dataset = generate_dataset(tools_kb, args.num_examples, args.seed)

    # Ensure output directory exists
    output_path.parent.mkdir(parents=True, exist_ok=True)

    print(f"\nSaving to: {output_path}")
    with open(output_path, 'w') as f:
        json.dump(dataset, f, indent=2)

    print(f"\nDone! Generated {len(dataset['examples'])} examples")

    # Print sample
    print("\nSample example:")
    print("-" * 40)
    sample = dataset['examples'][0]
    print(f"Query: {sample['user_query']}")
    print(f"Context: {sample['context']}")


if __name__ == "__main__":
    main()
