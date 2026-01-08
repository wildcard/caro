#!/usr/bin/env python3
"""
Pattern Gap Analyzer - Automated Detection of Safety Pattern Coverage Gaps

Analyzes safety patterns in patterns.rs and identifies missing variants that
attackers could exploit.

Usage:
    ./analyze-pattern-gaps.py src/safety/patterns.rs
    ./analyze-pattern-gaps.py src/safety/patterns.rs -o gaps.md
    ./analyze-pattern-gaps.py src/safety/patterns.rs --format json
    ./analyze-pattern-gaps.py src/safety/patterns.rs --min-severity high
"""

import argparse
import sys
import json
from datetime import datetime
from typing import List, Dict
from pathlib import Path

# Add the scripts directory to path
sys.path.insert(0, str(Path(__file__).parent))

from pattern_analyzer.parser import parse_patterns_file, get_pattern_summary
from pattern_analyzer.argument_detector import detect_argument_order_gaps
from pattern_analyzer.path_detector import detect_path_gaps
from pattern_analyzer.wildcard_detector import detect_wildcard_gaps
from pattern_analyzer.platform_detector import detect_platform_gaps


def main():
    """Main entry point for the gap analyzer."""
    args = parse_arguments()

    try:
        # Parse patterns file
        print(f"📖 Parsing patterns from {args.patterns_file}...", file=sys.stderr)
        patterns = parse_patterns_file(args.patterns_file)
        print(f"✅ Found {len(patterns)} patterns\n", file=sys.stderr)

        # Run detectors
        all_gaps = []

        if not args.detector or args.detector == 'argument':
            print("🔍 Running argument order detector...", file=sys.stderr)
            for pattern in patterns:
                gaps = detect_argument_order_gaps(pattern)
                all_gaps.extend(gaps)

        if not args.detector or args.detector == 'path':
            print("🔍 Running path variant detector...", file=sys.stderr)
            for pattern in patterns:
                gaps = detect_path_gaps(pattern)
                all_gaps.extend(gaps)

        if not args.detector or args.detector == 'wildcard':
            print("🔍 Running wildcard detector...", file=sys.stderr)
            for pattern in patterns:
                gaps = detect_wildcard_gaps(pattern)
                all_gaps.extend(gaps)

        if not args.detector or args.detector == 'platform':
            print("🔍 Running platform equivalent detector...", file=sys.stderr)
            for pattern in patterns:
                gaps = detect_platform_gaps(pattern)
                all_gaps.extend(gaps)

        # Filter by severity
        if args.min_severity:
            severity_order = {'critical': 0, 'high': 1, 'medium': 2, 'low': 3}
            min_level = severity_order[args.min_severity]
            all_gaps = [g for g in all_gaps if severity_order.get(g['severity'], 3) <= min_level]

        print(f"✅ Analysis complete. Found {len(all_gaps)} gaps\n", file=sys.stderr)

        # Generate output
        if args.format == 'json':
            output = generate_json_report(patterns, all_gaps)
        else:
            output = generate_markdown_report(patterns, all_gaps)

        # Write output
        if args.output:
            with open(args.output, 'w') as f:
                f.write(output)
            print(f"📝 Report written to {args.output}", file=sys.stderr)
        else:
            print(output)

        # Exit with appropriate code
        if all_gaps:
            sys.exit(0)  # Gaps found (not an error, just informational)
        else:
            print("🎉 No gaps detected!", file=sys.stderr)
            sys.exit(0)

    except FileNotFoundError as e:
        print(f"❌ Error: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"❌ Unexpected error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


def parse_arguments():
    """Parse command-line arguments."""
    parser = argparse.ArgumentParser(
        description='Analyze safety patterns for coverage gaps',
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )

    parser.add_argument(
        'patterns_file',
        help='Path to patterns.rs file (e.g., src/safety/patterns.rs)'
    )

    parser.add_argument(
        '-o', '--output',
        help='Output file path (default: stdout)',
        type=str,
    )

    parser.add_argument(
        '--format',
        choices=['markdown', 'json'],
        default='markdown',
        help='Output format (default: markdown)',
    )

    parser.add_argument(
        '--min-severity',
        choices=['critical', 'high', 'medium', 'low'],
        help='Only show gaps at or above this severity',
    )

    parser.add_argument(
        '--detector',
        choices=['argument', 'path', 'wildcard', 'platform'],
        help='Run only a specific detector (default: all)',
    )

    return parser.parse_args()


def generate_markdown_report(patterns: List[Dict], gaps: List[Dict]) -> str:
    """Generate a markdown-formatted report."""
    # Sort gaps by severity
    severity_order = {'critical': 0, 'high': 1, 'medium': 2, 'low': 3}
    gaps.sort(key=lambda g: severity_order.get(g['severity'], 3))

    # Group by severity
    by_severity = {
        'critical': [g for g in gaps if g['severity'] == 'critical'],
        'high': [g for g in gaps if g['severity'] == 'high'],
        'medium': [g for g in gaps if g['severity'] == 'medium'],
        'low': [g for g in gaps if g['severity'] == 'low'],
    }

    # Build report
    lines = []
    lines.append("# Safety Pattern Gap Analysis Report")
    lines.append("")
    lines.append(f"**Generated**: {datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S UTC')}")
    lines.append(f"**Patterns Analyzed**: {len(patterns)}")
    lines.append(f"**Gaps Found**: {len(gaps)} "
                 f"({len(by_severity['critical'])} critical, "
                 f"{len(by_severity['high'])} high, "
                 f"{len(by_severity['medium'])} medium, "
                 f"{len(by_severity['low'])} low)")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Pattern summary
    summary = get_pattern_summary(patterns)
    lines.append("## Pattern Summary")
    lines.append("")
    lines.append(f"- **Total Patterns**: {summary['total']}")
    lines.append(f"- **By Risk Level**:")
    lines.append(f"  - Critical: {summary['by_risk'].get('Critical', 0)}")
    lines.append(f"  - High: {summary['by_risk'].get('High', 0)}")
    lines.append(f"  - Medium: {summary['by_risk'].get('Medium', 0)}")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Gaps by severity
    for severity in ['critical', 'high', 'medium', 'low']:
        severity_gaps = by_severity[severity]
        if not severity_gaps:
            continue

        emoji = {'critical': '🔴', 'high': '🟠', 'medium': '🟡', 'low': '⚪'}[severity]
        lines.append(f"## {emoji} {severity.upper()} Severity Gaps ({len(severity_gaps)})")
        lines.append("")

        for i, gap in enumerate(severity_gaps, 1):
            lines.append(f"### Gap {i}: {gap['type'].replace('_', ' ').title()} - {gap['affected_command']}")
            lines.append("")
            lines.append(f"**Type**: {gap['type']}")
            lines.append(f"**Severity**: {gap['severity']}")
            lines.append(f"**Command**: `{gap['affected_command']}`")
            lines.append("")
            lines.append("**Original Pattern**:")
            lines.append(f"```regex")
            lines.append(gap['original_pattern'])
            lines.append("```")
            lines.append("")
            lines.append(f"**Missing Variant**: `{gap['missing_variant']}`")
            lines.append("")
            lines.append("**Example Command That Would Bypass**:")
            lines.append(f"```bash")
            lines.append(gap['example_command'])
            lines.append("```")
            lines.append("")
            lines.append("**Recommendation**:")
            lines.append(f"> {gap['recommendation']}")
            lines.append("")
            lines.append("---")
            lines.append("")

    # Summary recommendations
    lines.append("## Summary & Next Steps")
    lines.append("")
    if len(by_severity['critical']) > 0:
        lines.append("### ⚠️ Critical Gaps Require Immediate Attention")
        lines.append("")
        lines.append(f"Found {len(by_severity['critical'])} critical gaps. These should be fixed ASAP.")
        lines.append("")
    if len(gaps) == 0:
        lines.append("✅ **No gaps detected!** All patterns have comprehensive coverage.")
        lines.append("")
    else:
        lines.append("### Action Items")
        lines.append("")
        lines.append("1. Review each gap starting with Critical severity")
        lines.append("2. Add test cases for missing variants (TDD)")
        lines.append("3. Update patterns to cover gaps")
        lines.append("4. Re-run this analyzer to verify fixes")
        lines.append("")

    lines.append("---")
    lines.append("")
    lines.append("*Generated by Pattern Gap Analyzer v1.0.0*")

    return '\n'.join(lines)


def generate_json_report(patterns: List[Dict], gaps: List[Dict]) -> str:
    """Generate a JSON-formatted report."""
    summary = get_pattern_summary(patterns)

    report = {
        'metadata': {
            'generated_at': datetime.utcnow().isoformat() + 'Z',
            'patterns_analyzed': len(patterns),
            'gaps_found': len(gaps),
        },
        'pattern_summary': summary,
        'gaps': gaps,
    }

    return json.dumps(report, indent=2)


if __name__ == '__main__':
    main()
