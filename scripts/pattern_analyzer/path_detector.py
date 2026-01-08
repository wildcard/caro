"""
Path Variant Detector - Finds missing path representation variants

Detects when patterns don't cover all path representations.
Example: Pattern covers ".." but misses "../" or "../."
"""

import re
from typing import List, Dict


def detect_path_gaps(pattern: Dict) -> List[Dict]:
    """
    Detect missing path variant coverage in a pattern.

    Args:
        pattern: Pattern dict from parser

    Returns:
        List of Gap dicts
    """
    gaps = []
    regex = pattern['regex']
    command = pattern['command_base']

    # Extract paths from the pattern
    paths = _extract_paths_from_pattern(regex)

    if not paths:
        # No paths detected
        return gaps

    # Generate variants for each path
    for path in paths:
        variants = _generate_path_variants(path)

        for variant in variants:
            if not _pattern_covers_path(regex, variant['test_pattern']):
                gaps.append({
                    'type': 'path_variant',
                    'severity': _assess_path_severity(pattern, path, variant),
                    'original_pattern': regex,
                    'missing_variant': variant['variant'],
                    'example_command': f"{command} {variant['example']}",
                    'recommendation': variant['recommendation'],
                    'affected_command': command,
                })

    return gaps


def _extract_paths_from_pattern(regex: str) -> List[str]:
    """
    Extract path patterns from regex.

    Returns:
        List of path strings found in the regex
    """
    paths = []

    # Pattern 1: Parent directory references
    if '..' in regex:
        paths.append('..')

    # Pattern 2: Current directory
    if re.search(r'\\\.(?![*+?])', regex):  # \. but not followed by * + ?
        paths.append('.')

    # Pattern 3: Root/absolute paths
    if '/' in regex or re.search(r'\\\/', regex):
        # Extract specific paths
        abs_paths = re.findall(r'(/[a-z/]*)', regex.replace('\\/', '/'))
        paths.extend(abs_paths)

    # Pattern 4: Wildcards in paths
    if '*' in regex:
        paths.append('*')

    return list(set(paths))


def _generate_path_variants(base_path: str) -> List[Dict]:
    """
    Generate variants of a path.

    Args:
        base_path: Base path extracted from regex

    Returns:
        List of variant dicts
    """
    variants = []

    if base_path == '..':
        # Parent directory variants
        variants.extend([
            {
                'variant': '../',
                'test_pattern': r'\.\.\/',
                'example': '-rf ../',
                'recommendation': 'Add: \\.\\.\\/?',
            },
            {
                'variant': '../.',
                'test_pattern': r'\.\./\.',
                'example': '-rf ../.',
                'recommendation': 'Add: \\.\\./\\.?',
            },
            {
                'variant': '../../',
                'test_pattern': r'\.\./\.\.\/',
                'example': '-rf ../../',
                'recommendation': 'Add: (\\.\\./)+',
            },
            {
                'variant': './..',
                'test_pattern': r'\.\/\.\.',
                'example': '-rf ./..',
                'recommendation': 'Add: \\.\\/(\\.\\./?)*',
            },
        ])

    elif base_path == '.':
        # Current directory variants
        variants.extend([
            {
                'variant': './',
                'test_pattern': r'\./',
                'example': '-rf ./',
                'recommendation': 'Add: \\.\\/?',
            },
            {
                'variant': './*',
                'test_pattern': r'\./\*',
                'example': '-rf ./*',
                'recommendation': 'Add: \\.\\/(\\*|\\*\\*)',
            },
        ])

    elif base_path == '/':
        # Root directory variants
        variants.extend([
            {
                'variant': '/*',
                'test_pattern': r'/\*',
                'example': '-rf /*',
                'recommendation': 'Add: /\\*+',
            },
            {
                'variant': '/.',
                'test_pattern': r'/\.',
                'example': '-rf /.',
                'recommendation': 'Add: /\\.?',
            },
        ])

    elif base_path.startswith('/'):
        # Specific absolute path variants
        variants.extend([
            {
                'variant': f'{base_path}/',
                'test_pattern': base_path + '/',
                'example': f'-rf {base_path}/',
                'recommendation': f'Add trailing slash: {base_path}\\/?',
            },
            {
                'variant': f'{base_path}/*',
                'test_pattern': base_path + '/\\*',
                'example': f'-rf {base_path}/*',
                'recommendation': f'Add wildcard: {base_path}(/\\*)?',
            },
        ])

    elif base_path == '*':
        # Wildcard variants
        variants.extend([
            {
                'variant': './*',
                'test_pattern': r'\./\*',
                'example': '-rf ./*',
                'recommendation': 'Add: (\\./)? before \\*',
            },
            {
                'variant': '**',
                'test_pattern': r'\*\*',
                'example': '-rf **',
                'recommendation': 'Add: \\*+ to allow recursive globs',
            },
        ])

    return variants


def _pattern_covers_path(regex: str, test_pattern: str) -> bool:
    """
    Test if regex covers a specific path pattern.

    Args:
        regex: The pattern regex
        test_pattern: Path pattern to test (as regex)

    Returns:
        True if the original regex would match the test pattern
    """
    try:
        # Simple heuristic: check if test_pattern substring is in regex
        # This is approximate but works for most cases

        # Normalize both patterns
        norm_regex = regex.replace('\\s+', ' ').replace('\\s*', ' ')
        norm_test = test_pattern.replace('\\s+', ' ').replace('\\s*', ' ')

        return norm_test in norm_regex

    except Exception:
        return False


def _assess_path_severity(pattern: Dict, original_path: str, variant: Dict) -> str:
    """
    Assess severity of missing path variant.

    Args:
        pattern: Original pattern dict
        original_path: The base path
        variant: The missing variant

    Returns:
        Severity level
    """
    risk_level = pattern.get('risk_level', 'High')

    # Trailing slash variants are very common mistakes
    if variant['variant'].endswith('/'):
        if risk_level == 'Critical':
            return 'critical'
        return 'high'

    # Recursive parent directory (.., ../../) are critical for file deletion
    if '..' in variant['variant'] and 'rm' in pattern.get('command_base', ''):
        if risk_level == 'Critical':
            return 'critical'
        return 'high'

    # Current directory wildcards (./*) are common
    if './*' in variant['variant']:
        if risk_level == 'Critical':
            return 'high'
        return 'medium'

    # Default: one level below original risk
    if risk_level == 'Critical':
        return 'high'
    elif risk_level == 'High':
        return 'medium'
    else:
        return 'low'
