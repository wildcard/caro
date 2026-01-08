"""
Wildcard Detector - Finds missing wildcard pattern coverage

Detects when patterns don't cover shell glob/wildcard variations.
Example: Pattern covers "*" but misses ".*" or "**"
"""

import re
from typing import List, Dict, Set


def detect_wildcard_gaps(pattern: Dict) -> List[Dict]:
    """
    Detect missing wildcard variant coverage.

    Args:
        pattern: Pattern dict from parser

    Returns:
        List of Gap dicts
    """
    gaps = []
    regex = pattern['regex']
    command = pattern['command_base']
    shell = pattern.get('shell_specific')

    # Check if pattern deals with wildcards at all
    if '*' not in regex and '?' not in regex and '[' not in regex:
        return gaps  # No wildcards, no gaps

    # Extract wildcard patterns
    wildcards = _extract_wildcards(regex)

    if not wildcards:
        return gaps

    # Generate variants for each wildcard
    for wildcard in wildcards:
        variants = _generate_wildcard_variants(wildcard, shell)

        for variant in variants:
            if not _pattern_covers_wildcard(regex, variant['pattern']):
                gaps.append({
                    'type': 'wildcard',
                    'severity': _assess_wildcard_severity(pattern, variant),
                    'original_pattern': regex,
                    'missing_variant': variant['variant'],
                    'example_command': f"{command} {variant['example']}",
                    'recommendation': variant['recommendation'],
                    'affected_command': command,
                })

    return gaps


def _extract_wildcards(regex: str) -> Set[str]:
    """
    Extract wildcard patterns from regex.

    Returns:
        Set of wildcard strings ('*', '?', etc.)
    """
    wildcards = set()

    if '*' in regex or r'\*' in regex:
        wildcards.add('*')

    if '?' in regex or r'\?' in regex:
        wildcards.add('?')

    if '[' in regex and ']' in regex:
        wildcards.add('[...]')

    # Check for brace expansion patterns
    if '{' in regex and '}' in regex:
        wildcards.add('{...}')

    return wildcards


def _generate_wildcard_variants(base_wildcard: str, shell: str = None) -> List[Dict]:
    """
    Generate variants of wildcard patterns.

    Args:
        base_wildcard: Base wildcard ('*', '?', etc.)
        shell: Optional shell type

    Returns:
        List of variant dicts
    """
    variants = []

    if base_wildcard == '*':
        # Single asterisk variants
        variants.extend([
            {
                'variant': './*',
                'pattern': r'\./\*',
                'example': './*',
                'recommendation': 'Add: (\\./)? before \\*',
            },
            {
                'variant': '*.txt',
                'pattern': r'\*\.txt',
                'example': '*.txt',
                'recommendation': 'Add: \\*\\.\\w+',
            },
            {
                'variant': '**',
                'pattern': r'\*\*',
                'example': '**',
                'recommendation': 'Add: \\*+ for recursive globs',
            },
            {
                'variant': 'file*',
                'pattern': r'\\w+\*',
                'example': 'file*',
                'recommendation': 'Add: [a-zA-Z0-9_]+\\*',
            },
        ])

        # Bash/Zsh specific: globstar
        if shell in ['bash', 'zsh', None]:
            variants.append({
                'variant': '**/*',
                'pattern': r'\*\*/\*',
                'example': '**/*',
                'recommendation': 'Add: \\*\\*/\\* for recursive directory match',
            })

    elif base_wildcard == '?':
        # Single character match variants
        variants.extend([
            {
                'variant': 'file?.txt',
                'pattern': r'\\w+\?\\.\w+',
                'example': 'file?.txt',
                'recommendation': 'Add: [a-zA-Z0-9_]+\\?',
            },
            {
                'variant': '???',
                'pattern': r'\?\?\?',
                'example': '???',
                'recommendation': 'Add: \\?+ for multiple single-char matches',
            },
        ])

    elif base_wildcard == '[...]':
        # Character class variants
        variants.extend([
            {
                'variant': '[abc]*',
                'pattern': r'\[[a-z]+\]\*',
                'example': '[abc]*',
                'recommendation': 'Add: \\[[a-zA-Z0-9]+\\]\\*',
            },
            {
                'variant': '[0-9]*',
                'pattern': r'\[0-9\]\*',
                'example': '[0-9]*',
                'recommendation': 'Add: \\[[0-9\\-]+\\]\\*',
            },
        ])

    elif base_wildcard == '{...}':
        # Brace expansion (bash/zsh)
        if shell in ['bash', 'zsh', None]:
            variants.extend([
                {
                    'variant': '{a,b}',
                    'pattern': r'\{[a-z,]+\}',
                    'example': '{a,b}',
                    'recommendation': 'Add: \\{[a-zA-Z0-9,]+\\}',
                },
                {
                    'variant': '{1..10}',
                    'pattern': r'\{\d+\.\.\d+\}',
                    'example': '{1..10}',
                    'recommendation': 'Add: \\{\\d+\\.\\.\\d+\\}',
                },
            ])

    return variants


def _pattern_covers_wildcard(regex: str, test_pattern: str) -> bool:
    """
    Test if regex covers a wildcard pattern.

    Args:
        regex: Original pattern regex
        test_pattern: Wildcard pattern to test

    Returns:
        True if covered
    """
    # Simple substring check (approximation)
    # In a real implementation, this would be more sophisticated
    return test_pattern in regex or test_pattern.replace('\\', '') in regex


def _assess_wildcard_severity(pattern: Dict, variant: Dict) -> str:
    """
    Assess severity of missing wildcard variant.

    Args:
        pattern: Original pattern dict
        variant: Missing variant

    Returns:
        Severity level
    """
    risk_level = pattern.get('risk_level', 'High')
    variant_str = variant['variant']

    # Recursive globs (**) are high risk for deletion commands
    if '**' in variant_str:
        if 'rm' in pattern.get('command_base', ''):
            if risk_level == 'Critical':
                return 'critical'
            return 'high'
        return 'medium'

    # Hidden files (.*) are often overlooked
    if '.*' in variant_str or './' in variant_str:
        if risk_level == 'Critical':
            return 'high'
        return 'medium'

    # Other wildcard variants: one level below original risk
    if risk_level == 'Critical':
        return 'high'
    elif risk_level == 'High':
        return 'medium'
    else:
        return 'low'


def check_shell_glob_support(shell: str, glob_pattern: str) -> bool:
    """
    Check if a shell supports a specific glob pattern.

    Args:
        shell: Shell name (bash, zsh, fish, powershell)
        glob_pattern: Glob pattern to check

    Returns:
        True if shell supports the pattern
    """
    # Bash globstar support (requires shopt -s globstar)
    if shell == 'bash':
        if '**' in glob_pattern:
            return False  # Requires explicit enablement

    # Zsh has better glob support by default
    if shell == 'zsh':
        return True  # Zsh supports most patterns

    # Fish has limited glob support
    if shell == 'fish':
        if '**' in glob_pattern or '{' in glob_pattern:
            return False

    # PowerShell uses different wildcard syntax
    if shell == 'powershell':
        if '[' in glob_pattern or '{' in glob_pattern:
            return False  # Different syntax

    return True  # Default: assume supported
