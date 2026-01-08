"""
Platform Equivalent Detector - Finds missing cross-platform command equivalents

Detects when patterns cover one platform but miss equivalent commands on other platforms.
Example: Pattern covers "rm -rf" (POSIX) but misses "Remove-Item -Recurse -Force" (PowerShell)
"""

import re
from typing import List, Dict, Optional


# Platform-specific command equivalents
PLATFORM_EQUIVALENTS = {
    'rm': {
        'posix': ['rm', 'unlink'],
        'powershell': ['Remove-Item', 'ri', 'rm', 'del', 'erase', 'rd'],
        'cmd': ['del', 'erase', 'rd', 'rmdir'],
    },
    'dd': {
        'posix': ['dd'],
        'powershell': [],  # No direct equivalent, would use Set-Content or similar
        'cmd': [],  # No direct equivalent
    },
    'chmod': {
        'posix': ['chmod'],
        'powershell': ['icacls', 'Set-Acl'],
        'cmd': ['icacls', 'attrib', 'cacls'],
    },
    'chown': {
        'posix': ['chown'],
        'powershell': ['Set-Acl', 'takeown'],
        'cmd': ['takeown', 'icacls'],
    },
    'find': {
        'posix': ['find'],
        'powershell': ['Get-ChildItem', 'gci', 'dir', 'ls'],
        'cmd': ['dir', 'where'],
    },
    'grep': {
        'posix': ['grep', 'egrep', 'fgrep'],
        'powershell': ['Select-String', 'sls', 'findstr'],
        'cmd': ['findstr', 'find'],
    },
    'kill': {
        'posix': ['kill', 'pkill', 'killall'],
        'powershell': ['Stop-Process', 'spps', 'kill'],
        'cmd': ['taskkill'],
    },
    'mv': {
        'posix': ['mv'],
        'powershell': ['Move-Item', 'mi', 'mv', 'move'],
        'cmd': ['move', 'ren', 'rename'],
    },
    'cp': {
        'posix': ['cp'],
        'powershell': ['Copy-Item', 'ci', 'cp', 'copy'],
        'cmd': ['copy', 'xcopy', 'robocopy'],
    },
}

# Flag equivalents across platforms
FLAG_EQUIVALENTS = {
    'rm': {
        '-r': {
            'posix': ['-r', '-R', '--recursive'],
            'powershell': ['-Recurse', '-r'],
            'cmd': ['/s'],
        },
        '-f': {
            'posix': ['-f', '--force'],
            'powershell': ['-Force', '-f'],
            'cmd': ['/f', '/q'],
        },
    },
    'find': {
        '-name': {
            'posix': ['-name', '-iname'],
            'powershell': ['-Filter', '-Name'],
            'cmd': None,  # Different syntax
        },
    },
}


def detect_platform_gaps(pattern: Dict) -> List[Dict]:
    """
    Detect missing cross-platform command coverage.

    Args:
        pattern: Pattern dict from parser

    Returns:
        List of Gap dicts
    """
    gaps = []
    command = pattern['command_base']
    regex = pattern['regex']
    shell_specific = pattern.get('shell_specific')

    # Get platform equivalents for this command
    equivalents = get_platform_equivalents(command)

    if not equivalents:
        # No known equivalents for this command
        return gaps

    # Check coverage for each platform
    coverage = check_pattern_platform_coverage(pattern, equivalents)

    for platform, covered in coverage.items():
        if not covered and platform != 'posix':  # We expect POSIX to be covered
            # Generate gap for missing platform
            gap = _generate_platform_gap(pattern, platform, equivalents[platform])
            if gap:
                gaps.append(gap)

    return gaps


def get_platform_equivalents(command: str) -> Dict[str, List[str]]:
    """
    Get platform-specific equivalents for a command.

    Args:
        command: Base command (e.g., 'rm', 'dd')

    Returns:
        Dict mapping platform -> list of equivalent commands
    """
    # Normalize command
    cmd_normalized = command.lower().strip()

    # Handle compound commands (e.g., "sudo rm")
    if ' ' in cmd_normalized:
        parts = cmd_normalized.split()
        cmd_normalized = parts[-1]  # Take the actual command, not sudo/etc

    return PLATFORM_EQUIVALENTS.get(cmd_normalized, {})


def check_pattern_platform_coverage(pattern: Dict, equivalents: Dict[str, List[str]]) -> Dict[str, bool]:
    """
    Check which platforms are covered by the pattern.

    Args:
        pattern: Pattern dict
        equivalents: Platform equivalents dict

    Returns:
        Dict mapping platform -> True/False (covered/not covered)
    """
    regex = pattern['regex']
    coverage = {}

    for platform, commands in equivalents.items():
        # Check if ANY of the platform-specific commands appear in the regex
        covered = False

        for cmd in commands:
            if _command_in_regex(cmd, regex):
                covered = True
                break

        coverage[platform] = covered

    return coverage


def _command_in_regex(command: str, regex: str) -> bool:
    """
    Check if a command appears in a regex pattern.

    Args:
        command: Command to look for
        regex: Regex pattern

    Returns:
        True if command is present
    """
    # Case-insensitive search for command
    cmd_lower = command.lower()
    regex_lower = regex.lower()

    # Check for exact command match
    # Look for command at word boundaries
    patterns = [
        f'^{cmd_lower}',  # Start of regex
        f'{cmd_lower}\\s',  # Followed by whitespace
        f'{cmd_lower}\\\\s',  # Followed by \s
        f'\\b{cmd_lower}\\b',  # Word boundary
    ]

    for pattern in patterns:
        if re.search(pattern, regex_lower):
            return True

    return False


def _generate_platform_gap(pattern: Dict, platform: str, commands: List[str]) -> Optional[Dict]:
    """
    Generate a gap for a missing platform.

    Args:
        pattern: Original pattern
        platform: Missing platform (powershell, cmd)
        commands: List of equivalent commands for this platform

    Returns:
        Gap dict or None
    """
    if not commands:
        return None  # No equivalents exist for this platform

    base_command = pattern['command_base']
    risk_level = pattern.get('risk_level', 'High')

    # Pick the most common equivalent command
    primary_cmd = commands[0]

    # Generate example command for this platform
    example = _generate_platform_example(base_command, platform, primary_cmd, pattern)

    # Generate recommendation
    recommendation = _generate_platform_recommendation(pattern, platform, commands)

    return {
        'type': 'platform',
        'severity': _assess_platform_severity(pattern, platform),
        'original_pattern': pattern['regex'],
        'missing_variant': f'{platform.capitalize()}: {primary_cmd}',
        'example_command': example,
        'recommendation': recommendation,
        'affected_command': base_command,
    }


def _generate_platform_example(base_cmd: str, platform: str, platform_cmd: str, pattern: Dict) -> str:
    """
    Generate an example command for the missing platform.

    Args:
        base_cmd: Original command (rm, chmod, etc.)
        platform: Target platform
        platform_cmd: Platform-specific command
        pattern: Original pattern dict

    Returns:
        Example command string
    """
    # Map common operations to platform-specific syntax
    if base_cmd == 'rm' and platform == 'powershell':
        return f'{platform_cmd} -Recurse -Force C:\\dangerous\\path'

    if base_cmd == 'rm' and platform == 'cmd':
        return f'{platform_cmd} /s /q C:\\dangerous\\path'

    if base_cmd == 'chmod' and platform == 'powershell':
        return f'{platform_cmd} C:\\file.txt /grant Everyone:F'

    if base_cmd == 'find' and platform == 'powershell':
        return f'{platform_cmd} -Path C:\\ -Recurse -Filter *.txt'

    # Default: simple command
    return f'{platform_cmd} <args>'


def _generate_platform_recommendation(pattern: Dict, platform: str, commands: List[str]) -> str:
    """
    Generate a recommendation for covering the platform.

    Args:
        pattern: Original pattern
        platform: Missing platform
        commands: List of equivalent commands

    Returns:
        Recommendation string
    """
    cmd_list = ' | '.join(commands)

    if pattern.get('shell_specific'):
        # Pattern is already shell-specific
        return (
            f'Create separate pattern for {platform}: '
            f'Use shell_specific: Shell::{platform.capitalize()}, '
            f'Pattern: {commands[0]} ...'
        )
    else:
        # Pattern is cross-platform
        return (
            f'Add alternation for {platform}: '
            f'(original | {commands[0]} ...)'
        )


def _assess_platform_severity(pattern: Dict, platform: str) -> str:
    """
    Assess severity of missing platform coverage.

    Args:
        pattern: Original pattern
        platform: Missing platform

    Returns:
        Severity level
    """
    risk_level = pattern.get('risk_level', 'High')

    # PowerShell is widely used, higher severity
    if platform == 'powershell':
        if risk_level == 'Critical':
            return 'high'
        elif risk_level == 'High':
            return 'medium'
        else:
            return 'low'

    # CMD is less common for dangerous operations
    if platform == 'cmd':
        if risk_level == 'Critical':
            return 'medium'
        else:
            return 'low'

    return 'low'


def get_platform_flag_equivalents(command: str, flag: str) -> Dict[str, Optional[List[str]]]:
    """
    Get platform-specific flag equivalents.

    Args:
        command: Base command
        flag: Flag to get equivalents for

    Returns:
        Dict mapping platform -> list of equivalent flags (or None)
    """
    cmd_flags = FLAG_EQUIVALENTS.get(command, {})
    return cmd_flags.get(flag, {})
