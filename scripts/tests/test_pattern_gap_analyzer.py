#!/usr/bin/env python3
"""
Unit Tests for Pattern Gap Analyzer

Tests all components of the gap analyzer system.
"""

import unittest
import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from pattern_analyzer.parser import (
    parse_patterns_file,
    extract_command_from_regex,
    get_pattern_summary,
)
from pattern_analyzer.argument_detector import (
    detect_argument_order_gaps,
    _extract_flags,
)
from pattern_analyzer.path_detector import (
    detect_path_gaps,
    _extract_paths_from_pattern,
)
from pattern_analyzer.wildcard_detector import (
    detect_wildcard_gaps,
    _extract_wildcards,
)
from pattern_analyzer.platform_detector import (
    detect_platform_gaps,
    get_platform_equivalents,
)


class TestParser(unittest.TestCase):
    """Test pattern parser functionality."""

    def test_extract_command_from_regex(self):
        """Test command extraction from regex patterns."""
        self.assertEqual(extract_command_from_regex(r'rm\s+-rf'), 'rm')
        self.assertEqual(extract_command_from_regex(r'dd\s+if='), 'dd')
        self.assertEqual(extract_command_from_regex(r'sudo\s+rm'), 'sudo rm')
        self.assertEqual(extract_command_from_regex(r'chmod\s+777'), 'chmod')

    def test_parse_patterns_file(self):
        """Test parsing actual patterns.rs file."""
        patterns = parse_patterns_file('src/safety/patterns.rs')

        # Should find patterns
        self.assertGreater(len(patterns), 0, "Should find at least one pattern")

        # Each pattern should have required fields
        for pattern in patterns:
            self.assertIn('regex', pattern)
            self.assertIn('description', pattern)
            self.assertIn('risk_level', pattern)
            self.assertIn('command_base', pattern)
            self.assertIn('line_number', pattern)

    def test_get_pattern_summary(self):
        """Test summary generation."""
        patterns = [
            {'regex': 'test', 'description': 'test', 'risk_level': 'Critical',
             'command_base': 'rm', 'shell_specific': None, 'line_number': 1},
            {'regex': 'test', 'description': 'test', 'risk_level': 'High',
             'command_base': 'dd', 'shell_specific': None, 'line_number': 2},
        ]

        summary = get_pattern_summary(patterns)

        self.assertEqual(summary['total'], 2)
        self.assertEqual(summary['by_risk']['Critical'], 1)
        self.assertEqual(summary['by_risk']['High'], 1)


class TestArgumentDetector(unittest.TestCase):
    """Test argument order detector."""

    def test_extract_flags(self):
        """Test flag extraction from regex."""
        flags = _extract_flags(r'rm\s+-[rfRF]+')
        self.assertIn('-r', flags)
        self.assertIn('-f', flags)

        flags = _extract_flags(r'dd\s+if=')
        self.assertIn('if=', flags)

    def test_detect_argument_order_gaps(self):
        """Test detection of argument order gaps."""
        pattern = {
            'regex': r'rm\s+-rf\s+\.\.',
            'description': 'Remove parent directory',
            'risk_level': 'Critical',
            'command_base': 'rm',
            'shell_specific': None,
        }

        gaps = detect_argument_order_gaps(pattern)

        # Should find some gaps
        self.assertGreater(len(gaps), 0)

        # Should have correct structure
        for gap in gaps:
            self.assertEqual(gap['type'], 'argument_order')
            self.assertIn('severity', gap)
            self.assertIn('example_command', gap)


class TestPathDetector(unittest.TestCase):
    """Test path variant detector."""

    def test_extract_paths(self):
        """Test path extraction from regex."""
        paths = _extract_paths_from_pattern(r'rm\s+-rf\s+\.\.')
        self.assertIn('..', paths)

        paths = _extract_paths_from_pattern(r'chmod\s+777\s+/')
        self.assertIn('/', paths)

    def test_detect_path_gaps(self):
        """Test detection of path gaps."""
        pattern = {
            'regex': r'rm\s+-rf\s+\.\.',
            'description': 'Remove parent directory',
            'risk_level': 'Critical',
            'command_base': 'rm',
            'shell_specific': None,
        }

        gaps = detect_path_gaps(pattern)

        # Should find gaps for trailing slash variants
        self.assertGreater(len(gaps), 0)

        # Check gap structure
        for gap in gaps:
            self.assertEqual(gap['type'], 'path_variant')


class TestWildcardDetector(unittest.TestCase):
    """Test wildcard detector."""

    def test_extract_wildcards(self):
        """Test wildcard extraction."""
        wildcards = _extract_wildcards(r'rm\s+\*')
        self.assertIn('*', wildcards)

        wildcards = _extract_wildcards(r'rm\s+file\?\.txt')
        self.assertIn('?', wildcards)

    def test_detect_wildcard_gaps(self):
        """Test wildcard gap detection."""
        pattern = {
            'regex': r'rm\s+\*',
            'description': 'Remove with wildcard',
            'risk_level': 'High',
            'command_base': 'rm',
            'shell_specific': None,
        }

        gaps = detect_wildcard_gaps(pattern)

        # Should find some gaps
        self.assertGreater(len(gaps), 0)


class TestPlatformDetector(unittest.TestCase):
    """Test platform equivalent detector."""

    def test_get_platform_equivalents(self):
        """Test platform equivalent lookup."""
        equivalents = get_platform_equivalents('rm')

        self.assertIn('posix', equivalents)
        self.assertIn('powershell', equivalents)
        self.assertIn('cmd', equivalents)

        self.assertIn('rm', equivalents['posix'])
        self.assertIn('Remove-Item', equivalents['powershell'])

    def test_detect_platform_gaps(self):
        """Test platform gap detection."""
        pattern = {
            'regex': r'rm\s+-rf',
            'description': 'Remove recursive',
            'risk_level': 'Critical',
            'command_base': 'rm',
            'shell_specific': None,
        }

        gaps = detect_platform_gaps(pattern)

        # Should find gaps for PowerShell and CMD
        self.assertGreater(len(gaps), 0)


class TestIntegration(unittest.TestCase):
    """Integration tests for the full system."""

    def test_analyze_real_patterns(self):
        """Test analyzing actual patterns.rs file."""
        # Parse patterns
        patterns = parse_patterns_file('src/safety/patterns.rs')

        # Run all detectors
        all_gaps = []

        for pattern in patterns:
            all_gaps.extend(detect_argument_order_gaps(pattern))
            all_gaps.extend(detect_path_gaps(pattern))
            all_gaps.extend(detect_wildcard_gaps(pattern))
            all_gaps.extend(detect_platform_gaps(pattern))

        # Should find some gaps (we know there are gaps)
        self.assertGreater(len(all_gaps), 0, "Should detect gaps in current patterns")

        # All gaps should have required fields
        for gap in all_gaps:
            self.assertIn('type', gap)
            self.assertIn('severity', gap)
            self.assertIn('original_pattern', gap)
            self.assertIn('missing_variant', gap)
            self.assertIn('example_command', gap)
            self.assertIn('recommendation', gap)
            self.assertIn('affected_command', gap)


def run_tests():
    """Run all tests."""
    unittest.main(argv=[''], verbosity=2, exit=False)


if __name__ == '__main__':
    run_tests()
