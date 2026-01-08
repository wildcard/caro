"""
Pattern Gap Analyzer - Safety Pattern Analysis Tool

This package provides automated detection of gaps in safety pattern coverage.
"""

__version__ = "1.0.0"
__author__ = "Caro Safety Team"

from .parser import parse_patterns_file, Pattern
from .argument_detector import detect_argument_order_gaps
from .path_detector import detect_path_gaps
from .wildcard_detector import detect_wildcard_gaps
from .platform_detector import detect_platform_gaps

__all__ = [
    'parse_patterns_file',
    'Pattern',
    'detect_argument_order_gaps',
    'detect_path_gaps',
    'detect_wildcard_gaps',
    'detect_platform_gaps',
]
