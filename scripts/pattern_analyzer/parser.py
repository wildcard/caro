"""
Pattern Parser - Extracts and parses safety patterns from patterns.rs

Parses Rust source to extract DangerPattern structs and converts them
into structured Python dictionaries for analysis.
"""

import re
from typing import List, Dict, Optional, TypedDict


class Pattern(TypedDict):
    """Structured representation of a safety pattern."""
    regex: str
    description: str
    risk_level: str
    shell_specific: Optional[str]
    command_base: str
    line_number: int


def parse_patterns_file(filepath: str) -> List[Pattern]:
    """
    Parse patterns.rs and extract all DangerPattern structs.

    Args:
        filepath: Path to patterns.rs file

    Returns:
        List of Pattern dictionaries

    Raises:
        FileNotFoundError: If patterns.rs doesn't exist
        ValueError: If file format is unexpected
    """
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
    except FileNotFoundError:
        raise FileNotFoundError(
            f"Patterns file not found: {filepath}\n"
            f"Expected path: src/safety/patterns.rs"
        )

    patterns = []
    lines = content.split('\n')

    i = 0
    while i < len(lines):
        line = lines[i].strip()

        # Look for DangerPattern struct start
        if line.startswith('DangerPattern {'):
            pattern_start_line = i + 1
            pattern_data = _extract_pattern_block(lines, i)

            if pattern_data:
                pattern_data['line_number'] = pattern_start_line
                patterns.append(pattern_data)

            # Skip to end of this pattern block
            i = _find_pattern_end(lines, i)

        i += 1

    if not patterns:
        raise ValueError(
            f"No patterns found in {filepath}\n"
            f"Expected to find DangerPattern {{ ... }} structs"
        )

    return patterns


def _extract_pattern_block(lines: List[str], start_idx: int) -> Optional[Pattern]:
    """
    Extract a single DangerPattern block starting at start_idx.

    Args:
        lines: All lines from the file
        start_idx: Index of line containing "DangerPattern {"

    Returns:
        Pattern dict or None if parsing fails
    """
    pattern_data: Dict = {}

    # Look ahead for pattern fields
    i = start_idx + 1
    brace_count = 1

    while i < len(lines) and brace_count > 0:
        line = lines[i].strip()

        # Track braces
        brace_count += line.count('{') - line.count('}')

        # Extract pattern field
        if line.startswith('pattern:'):
            pattern_data['regex'] = _extract_rust_string(line)

        # Extract risk_level field
        elif line.startswith('risk_level:'):
            pattern_data['risk_level'] = _extract_risk_level(line)

        # Extract description field
        elif line.startswith('description:'):
            pattern_data['description'] = _extract_rust_string(line)

        # Extract shell_specific field
        elif line.startswith('shell_specific:'):
            pattern_data['shell_specific'] = _extract_shell_specific(line)

        i += 1

    # Validate we got required fields
    if 'regex' not in pattern_data or 'description' not in pattern_data:
        return None

    # Default risk level if not found
    if 'risk_level' not in pattern_data:
        pattern_data['risk_level'] = 'High'

    # Extract command base from regex
    pattern_data['command_base'] = extract_command_from_regex(pattern_data['regex'])

    return Pattern(**pattern_data)


def _find_pattern_end(lines: List[str], start_idx: int) -> int:
    """Find the index of the closing brace for a DangerPattern block."""
    brace_count = 1
    i = start_idx + 1

    while i < len(lines) and brace_count > 0:
        line = lines[i]
        brace_count += line.count('{') - line.count('}')
        i += 1

    return i


def _extract_rust_string(line: str) -> str:
    """
    Extract a Rust string literal from a line.

    Handles raw strings: r"..." and regular strings: "..."
    """
    # Try raw string first: r"pattern"
    match = re.search(r'r"([^"]*)"', line)
    if match:
        return match.group(1)

    # Try raw string with hash: r#"pattern"#
    match = re.search(r'r#"([^"]*)"#', line)
    if match:
        return match.group(1)

    # Try regular string: "description"
    match = re.search(r'"([^"]*)"', line)
    if match:
        return match.group(1)

    return ""


def _extract_risk_level(line: str) -> str:
    """Extract RiskLevel enum value from line."""
    if 'Critical' in line:
        return 'Critical'
    elif 'High' in line:
        return 'High'
    elif 'Medium' in line:
        return 'Medium'
    else:
        return 'High'  # Default


def _extract_shell_specific(line: str) -> Optional[str]:
    """Extract Shell enum value from line, or None."""
    if 'None' in line:
        return None
    elif 'Bash' in line:
        return 'bash'
    elif 'Zsh' in line:
        return 'zsh'
    elif 'PowerShell' in line:
        return 'powershell'
    elif 'Fish' in line:
        return 'fish'
    else:
        return None


def extract_command_from_regex(regex: str) -> str:
    """
    Extract the base command from a regex pattern.

    Examples:
        "rm\\s+-rf" -> "rm"
        "dd\\s+if=" -> "dd"
        "sudo\\s+rm" -> "sudo rm"

    Args:
        regex: The regex pattern string

    Returns:
        Base command (e.g., "rm", "dd", "chmod")
    """
    # Remove common regex anchors
    regex = regex.lstrip('^').rstrip('$')

    # Look for command at start of pattern
    # Handle: "rm", "sudo rm", "chmod", etc.
    match = re.match(r'^([a-zA-Z_-]+(?:\\s\+[a-zA-Z_-]+)?)', regex)

    if match:
        cmd = match.group(1)
        # Convert regex \s+ back to space
        cmd = cmd.replace('\\s+', ' ').replace('\\s', ' ')
        return cmd.strip()

    # Try to find any command-like word
    match = re.search(r'([a-z]{2,})', regex.lower())
    if match:
        return match.group(1)

    return "unknown"


def normalize_regex(regex: str) -> str:
    """
    Normalize a regex pattern for comparison.

    - Removes redundant escapes
    - Normalizes whitespace patterns
    - Makes comparison easier

    Args:
        regex: Raw regex pattern

    Returns:
        Normalized pattern
    """
    # Normalize whitespace patterns
    regex = re.sub(r'\\s\+', r'\\s+', regex)
    regex = re.sub(r'\\s\*', r'\\s*', regex)

    # Remove redundant escapes (but keep meaningful ones)
    # This is conservative to avoid breaking patterns

    return regex


def get_pattern_summary(patterns: List[Pattern]) -> Dict[str, any]:
    """
    Generate summary statistics for a list of patterns.

    Args:
        patterns: List of Pattern dicts

    Returns:
        Summary dict with counts and breakdowns
    """
    summary = {
        'total': len(patterns),
        'by_risk': {'Critical': 0, 'High': 0, 'Medium': 0},
        'by_shell': {'all': 0, 'bash': 0, 'zsh': 0, 'powershell': 0, 'fish': 0},
        'by_command': {},
    }

    for pattern in patterns:
        # Count by risk level
        risk = pattern.get('risk_level', 'High')
        summary['by_risk'][risk] = summary['by_risk'].get(risk, 0) + 1

        # Count by shell
        shell = pattern.get('shell_specific') or 'all'
        summary['by_shell'][shell] = summary['by_shell'].get(shell, 0) + 1

        # Count by command
        cmd = pattern.get('command_base', 'unknown')
        summary['by_command'][cmd] = summary['by_command'].get(cmd, 0) + 1

    return summary
