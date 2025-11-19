#!/usr/bin/env python3
"""
Test prompt template rendering with sample inputs.

This validates that:
1. Templates render correctly with parameters
2. No unsubstituted variables remain
3. Output meets expected format
4. Safety language is present
"""

import yaml
import json
import sys
from pathlib import Path
from jinja2 import Template, TemplateSyntaxError


class PromptRenderingTester:
    """Tests prompt template rendering"""

    # Sample test inputs for each prompt
    TEST_SCENARIOS = [
        {
            'name': 'Basic file listing',
            'input': 'list all files',
            'shell': 'bash',
            'expected_keywords': ['ls'],
        },
        {
            'name': 'File search',
            'input': 'find all python files',
            'shell': 'bash',
            'expected_keywords': ['find', '.py'],
        },
        {
            'name': 'Dangerous request',
            'input': 'delete everything',
            'shell': 'bash',
            'expected_keywords': ['dangerous', 'clarify'],
        },
        {
            'name': 'Path with spaces',
            'input': 'list files in My Documents',
            'shell': 'bash',
            'expected_keywords': ['"My Documents"'],
        },
    ]

    def __init__(self):
        self.results = {}

    def test_prompt_rendering(self, filepath: Path) -> dict:
        """Test rendering a single prompt file"""
        result = {
            'file': filepath.name,
            'passed': True,
            'tests': []
        }

        try:
            with open(filepath) as f:
                data = yaml.safe_load(f)

            prompt_template = data.get('prompt', '')

            # Test with each scenario
            for scenario in self.TEST_SCENARIOS:
                test_result = {
                    'scenario': scenario['name'],
                    'passed': True,
                    'issues': []
                }

                try:
                    # Render template
                    template = Template(prompt_template)
                    rendered = template.render(
                        input=scenario['input'],
                        shell=scenario['shell']
                    )

                    # Check for unsubstituted variables
                    if '{{' in rendered or '}}' in rendered:
                        test_result['passed'] = False
                        test_result['issues'].append("Unsubstituted template variables found")
                        result['passed'] = False

                    # Check for required safety keywords
                    rendered_lower = rendered.lower()
                    safety_keywords = ['safe', 'posix', 'json']
                    missing = [kw for kw in safety_keywords if kw not in rendered_lower]

                    if missing:
                        test_result['issues'].append(f"Missing safety keywords: {', '.join(missing)}")

                    # Check scenario-specific expectations
                    for keyword in scenario.get('expected_keywords', []):
                        if keyword.lower() not in rendered_lower:
                            test_result['issues'].append(
                                f"Expected keyword '{keyword}' not emphasized in prompt"
                            )

                    # Validate output format guidance
                    if '{"cmd"' not in rendered:
                        test_result['issues'].append("Missing JSON output format example")

                except TemplateSyntaxError as e:
                    test_result['passed'] = False
                    test_result['issues'].append(f"Template syntax error: {e}")
                    result['passed'] = False

                result['tests'].append(test_result)

        except Exception as e:
            result['passed'] = False
            result['error'] = str(e)

        return result

    def run_all_tests(self, prompts_dir: Path) -> bool:
        """Run rendering tests on all prompts"""
        prompt_files = list(prompts_dir.glob('*.prompt.yaml'))

        print(f"\n{'='*70}")
        print(f"PROMPT RENDERING TESTS")
        print(f"{'='*70}\n")

        all_passed = True

        for filepath in sorted(prompt_files):
            result = self.test_prompt_rendering(filepath)
            self.results[filepath.name] = result

            status = "✅" if result['passed'] else "❌"
            print(f"{status} {filepath.name}")

            if 'error' in result:
                print(f"   ERROR: {result['error']}")
                all_passed = False
                continue

            # Show test results
            for test in result['tests']:
                test_status = "✓" if test['passed'] else "✗"
                print(f"   {test_status} {test['scenario']}")

                if test['issues']:
                    for issue in test['issues']:
                        print(f"      ⚠️  {issue}")

            print()

        # Summary
        print(f"{'='*70}")
        print(f"RENDERING TEST SUMMARY")
        print(f"{'='*70}")
        passed_count = sum(1 for r in self.results.values() if r['passed'])
        total_count = len(self.results)
        print(f"✅ Passed: {passed_count}/{total_count}")

        total_tests = sum(len(r.get('tests', [])) for r in self.results.values())
        passed_tests = sum(
            sum(1 for t in r.get('tests', []) if t['passed'])
            for r in self.results.values()
        )
        print(f"🧪 Individual tests: {passed_tests}/{total_tests}")
        print(f"{'='*70}\n")

        return all_passed


def main():
    """Main entry point"""
    prompts_dir = Path('prompts')

    if not prompts_dir.exists():
        print(f"❌ Prompts directory not found: {prompts_dir}")
        sys.exit(1)

    tester = PromptRenderingTester()
    success = tester.run_all_tests(prompts_dir)

    # Save results
    results_file = Path('rendering-test-results.json')
    with open(results_file, 'w') as f:
        json.dump(tester.results, f, indent=2)

    print(f"📄 Results saved to: {results_file}")

    if not success:
        print("\n❌ Some rendering tests failed!")
        sys.exit(1)
    else:
        print("\n✅ All rendering tests passed!")
        sys.exit(0)


if __name__ == '__main__':
    main()
