"""
Argument Order Detector - Finds missing flag/argument order permutations

Detects when patterns don't cover all possible orderings of flags and arguments.
Example: Pattern covers "rm -rf /path" but misses "rm /path -rf"
"""

import re
from typing import List, Dict, Set
from itertools import permutations


class Gap(Dict):
    """
    Represents a detected gap in pattern coverage.

    Fields:
        type: 'argument_order'
        severity: 'critical' | 'high' | 'medium' | 'low'
        original_pattern: Original regex
        missing_variant: The command variant not covered
        example_command: Concrete example that would bypass
        recommendation: Suggested fix
        affected_command: Base command (rm, dd, etc)
    """
    pass


def detect_argument_order_gaps(pattern: Dict) -> List[Gap]:
    """
    Detect missing argument order variations in a pattern.

    Args:
        pattern: Pattern dict from parser

    Returns:
        List of Gap dicts representing missing coverage
    """
    gaps = []
    regex = pattern['regex']
    command = pattern['command_base']

    # Extract flags from pattern
    flags = _extract_flags(regex)

    if not flags:
        # No flags detected, no argument order gaps possible
        return gaps

    # Generate flag permutations
    variants = _generate_flag_variants(command, flags, regex)

    # Check which variants are NOT covered by the pattern
    for variant in variants:
        if not _pattern_covers_variant(regex, variant['test_input']):
            gaps.append(Gap(
                type='argument_order',
                severity=_assess_severity(pattern, variant),
                original_pattern=regex,
                missing_variant=variant['variant_pattern'],
                example_command=variant['example'],
                recommendation=variant['recommendation'],
                affected_command=command,
            ))

    return gaps


def _extract_flags(regex: str) -> List[str]:
    """
    Extract flag patterns from regex.

    Examples:
        "rm\\s+-[rfRF]+" -> ['-r', '-f', '-R', '-F']
        "dd\\s+if=" -> ['if=']
        "chmod\\s+-R" -> ['-R']

    Returns:
        List of flag strings (may be single chars or full args)
    """
    flags = []

    # Pattern 1: Character class flags: -[rfRF]+
    matches = re.findall(r'-\[([a-zA-Z]+)\]\+?', regex)
    for match in matches:
        # Expand character class: [rfRF] -> ['-r', '-f', '-R', '-F']
        flags.extend([f'-{char}' for char in set(match)])

    # Pattern 2: Specific flags: -R, -f, etc.
    matches = re.findall(r'(-[a-zA-Z])\b', regex)
    flags.extend(matches)

    # Pattern 3: Long flags: --force, --recursive
    matches = re.findall(r'(--[a-z-]+)', regex)
    flags.extend(matches)

    # Pattern 4: Argument-style flags: if=, of=
    matches = re.findall(r'\b([a-z]{2,})=', regex)
    flags.extend([f'{m}=' for m in matches])

    return list(set(flags))  # Remove duplicates


def _generate_flag_variants(command: str, flags: List[str], original_regex: str) -> List[Dict]:
    """
    Generate different orderings of flags and arguments.

    Returns:
        List of variant dicts with:
            - variant_pattern: Description of variant
            - test_input: Example command to test
            - example: User-facing example
            - recommendation: Suggested regex fix
    """
    variants = []

    # Extract any path/argument from the original regex
    paths = _extract_path_patterns(original_regex)
    sample_path = paths[0] if paths else '/path'

    # Variant 1: Flags after argument
    # Original: rm -rf /path
    # Variant: rm /path -rf
    if len(flags) > 0:
        combined_flags = ''.join([f.lstrip('-') for f in flags if len(f) == 2])
        if combined_flags:
            variants.append({
                'variant_pattern': f'{command} <arg> -{combined_flags}',
                'test_input': f'{command} {sample_path} -{combined_flags}',
                'example': f'{command} {sample_path} -{combined_flags}',
                'recommendation': f'Add alternation: ({command}\\s+-.+\\s+{sample_path}|{command}\\s+{sample_path}\\s+-.+)',
            })

    # Variant 2: Separated flags
    # Original: rm -rf
    # Variant: rm -r -f
    if len(flags) >= 2:
        sep_flags = ' '.join(flags[:2])
        variants.append({
            'variant_pattern': f'{command} {sep_flags}',
            'test_input': f'{command} {sep_flags} {sample_path}',
            'example': f'{command} {sep_flags} {sample_path}',
            'recommendation': f'Allow optional whitespace between flags: -{flags[0].lstrip("-")}\\s*-{flags[1].lstrip("-")}',
        })

    # Variant 3: Different flag order
    # Original: rm -rf
    # Variant: rm -fr
    if len(flags) >= 2:
        # Take first 2 flags and swap them
        flag_chars = [f.lstrip('-') for f in flags[:2] if len(f) == 2]
        if len(flag_chars) >= 2:
            swapped = f'-{flag_chars[1]}{flag_chars[0]}'
            variants.append({
                'variant_pattern': f'{command} {swapped}',
                'test_input': f'{command} {swapped} {sample_path}',
                'example': f'{command} {swapped} {sample_path}',
                'recommendation': f'Use character class: -[{"".join(flag_chars)}]+',
            })

    # Variant 4: Flags interspersed with arguments (advanced)
    # rm -r /path -f
    if len(flags) >= 2 and sample_path:
        variants.append({
            'variant_pattern': f'{command} {flags[0]} <arg> {flags[1]}',
            'test_input': f'{command} {flags[0]} {sample_path} {flags[1]}',
            'example': f'{command} {flags[0]} {sample_path} {flags[1]}',
            'recommendation': f'Complex: Allow flags anywhere: {command}(\\s+(-[a-zA-Z]+|[^\\s]+))+',
        })

    return variants


def _extract_path_patterns(regex: str) -> List[str]:
    """
    Extract path-like patterns from regex.

    Returns sample paths for testing.
    """
    paths = []

    # Look for literal paths
    if '..' in regex or '/' in regex:
        # Extract paths
        path_matches = re.findall(r'([./][^\s\\)]+)', regex)
        for match in path_matches:
            # Clean up regex escapes
            path = match.replace('\\/', '/').replace('\\.', '.')
            paths.append(path)

    # Default paths if none found
    if not paths:
        paths = ['/path', '..', '.']

    return paths


def _pattern_covers_variant(regex: str, variant_command: str) -> bool:
    """
    Test if a regex pattern covers a specific command variant.

    Args:
        regex: The pattern regex
        variant_command: Command string to test

    Returns:
        True if pattern matches the variant
    """
    try:
        # Compile the regex
        pattern = re.compile(regex)

        # Test if it matches the variant
        return pattern.search(variant_command) is not None

    except re.error:
        # Invalid regex - can't test coverage
        return False


def _assess_severity(pattern: Dict, variant: Dict) -> str:
    """
    Assess the severity of a missing variant.

    Factors:
        - Original pattern's risk level
        - How different the variant is
        - Whether it's a common user mistake

    Returns:
        'critical' | 'high' | 'medium' | 'low'
    """
    original_risk = pattern.get('risk_level', 'High')

    # If original is Critical, missing variants are High
    if original_risk == 'Critical':
        # Flags after args is very common mistake
        if 'arg>' in variant['variant_pattern']:
            return 'critical'
        return 'high'

    # If original is High, missing variants are Medium-High
    if original_risk == 'High':
        if 'arg>' in variant['variant_pattern']:
            return 'high'
        return 'medium'

    # Original is Medium or lower
    return 'medium'


def get_argument_gap_summary(gaps: List[Gap]) -> Dict:
    """
    Generate summary statistics for argument order gaps.

    Args:
        gaps: List of Gap dicts

    Returns:
        Summary dict
    """
    if not gaps:
        return {'total': 0, 'by_severity': {}, 'by_command': {}}

    summary = {
        'total': len(gaps),
        'by_severity': {'critical': 0, 'high': 0, 'medium': 0, 'low': 0},
        'by_command': {},
    }

    for gap in gaps:
        # Count by severity
        sev = gap['severity']
        summary['by_severity'][sev] = summary['by_severity'].get(sev, 0) + 1

        # Count by command
        cmd = gap['affected_command']
        summary['by_command'][cmd] = summary['by_command'].get(cmd, 0) + 1

    return summary
