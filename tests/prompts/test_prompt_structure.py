#!/usr/bin/env python3
"""
Comprehensive prompt structure validation and testing.

This test suite validates that all prompt files:
1. Have correct YAML structure
2. Include required fields
3. Have valid test cases
4. Include safety constraints
5. Follow naming conventions
"""

import yaml
import sys
import json
from pathlib import Path
from typing import Dict, List, Any


class PromptValidator:
    """Validates prompt file structure and content"""

    REQUIRED_FIELDS = ['name', 'description', 'version', 'prompt', 'parameters']
    RECOMMENDED_FIELDS = ['tags', 'test_cases', 'expected_output_format']
    SAFETY_FIELDS = ['safety_constraints', 'safety_level']

    def __init__(self):
        self.errors = []
        self.warnings = []
        self.results = {}

    def validate_prompt_file(self, filepath: Path) -> Dict[str, Any]:
        """Validate a single prompt file"""
        result = {
            'file': filepath.name,
            'valid': True,
            'errors': [],
            'warnings': [],
            'metrics': {}
        }

        try:
            with open(filepath) as f:
                data = yaml.safe_load(f)

            # Check required fields
            for field in self.REQUIRED_FIELDS:
                if field not in data:
                    result['errors'].append(f"Missing required field: {field}")
                    result['valid'] = False

            # Check recommended fields
            for field in self.RECOMMENDED_FIELDS:
                if field not in data:
                    result['warnings'].append(f"Missing recommended field: {field}")

            # Validate version format (semver)
            if 'version' in data:
                version = str(data['version'])
                if len(version.split('.')) != 3:
                    result['errors'].append(f"Invalid version format: {version} (expected X.Y.Z)")
                    result['valid'] = False

            # Validate parameters
            if 'parameters' in data:
                params = data['parameters']
                if not isinstance(params, dict):
                    result['errors'].append("Parameters must be a dictionary")
                    result['valid'] = False
                else:
                    # Check for required input parameter
                    if 'input' not in params:
                        result['warnings'].append("Missing 'input' parameter")

            # Validate test cases
            if 'test_cases' in data:
                test_count = len(data['test_cases'])
                result['metrics']['test_cases'] = test_count

                if test_count < 3:
                    result['warnings'].append(f"Only {test_count} test cases (recommend at least 3)")

                # Validate each test case structure
                for i, test in enumerate(data['test_cases']):
                    if 'input' not in test:
                        result['errors'].append(f"Test case {i}: Missing 'input' field")
                        result['valid'] = False
            else:
                result['warnings'].append("No test cases defined")
                result['metrics']['test_cases'] = 0

            # Check for safety constraints
            has_safety = any(field in data for field in self.SAFETY_FIELDS)
            prompt_text = data.get('prompt', '').lower()
            mentions_safety = any(term in prompt_text for term in [
                'safe', 'dangerous', 'never', 'security', 'destructive'
            ])

            if not has_safety and not mentions_safety:
                result['warnings'].append("No explicit safety constraints or safety language")

            # Calculate metrics
            if 'prompt' in data:
                prompt_text = data['prompt']
                result['metrics']['char_count'] = len(prompt_text)
                result['metrics']['line_count'] = len(prompt_text.splitlines())
                result['metrics']['word_count'] = len(prompt_text.split())

            # Check tags
            if 'tags' in data:
                tags = data['tags']
                if 'production' in tags and 'experimental' in tags:
                    result['warnings'].append("Has both 'production' and 'experimental' tags")
                result['metrics']['tags'] = tags

        except yaml.YAMLError as e:
            result['errors'].append(f"YAML parsing error: {e}")
            result['valid'] = False
        except Exception as e:
            result['errors'].append(f"Unexpected error: {e}")
            result['valid'] = False

        return result

    def validate_all_prompts(self, prompts_dir: Path) -> bool:
        """Validate all prompt files in directory"""
        prompt_files = list(prompts_dir.glob('*.prompt.yaml'))

        if not prompt_files:
            print("❌ No prompt files found!")
            return False

        print(f"\n{'='*70}")
        print(f"PROMPT VALIDATION REPORT")
        print(f"{'='*70}\n")

        all_valid = True

        for filepath in sorted(prompt_files):
            result = self.validate_prompt_file(filepath)
            self.results[filepath.name] = result

            # Print result
            status = "✅" if result['valid'] else "❌"
            print(f"{status} {filepath.name}")

            if result['errors']:
                all_valid = False
                for error in result['errors']:
                    print(f"   ERROR: {error}")

            if result['warnings']:
                for warning in result['warnings']:
                    print(f"   ⚠️  {warning}")

            if result['metrics']:
                metrics = result['metrics']
                print(f"   📊 Metrics:")
                print(f"      - Characters: {metrics.get('char_count', 'N/A')}")
                print(f"      - Lines: {metrics.get('line_count', 'N/A')}")
                print(f"      - Words: {metrics.get('word_count', 'N/A')}")
                print(f"      - Test cases: {metrics.get('test_cases', 0)}")
                if 'tags' in metrics:
                    print(f"      - Tags: {', '.join(metrics['tags'])}")

            print()

        # Summary
        print(f"{'='*70}")
        print(f"SUMMARY")
        print(f"{'='*70}")
        valid_count = sum(1 for r in self.results.values() if r['valid'])
        total_count = len(self.results)
        print(f"✅ Valid prompts: {valid_count}/{total_count}")

        total_errors = sum(len(r['errors']) for r in self.results.values())
        total_warnings = sum(len(r['warnings']) for r in self.results.values())
        print(f"❌ Total errors: {total_errors}")
        print(f"⚠️  Total warnings: {total_warnings}")

        total_tests = sum(r['metrics'].get('test_cases', 0) for r in self.results.values())
        print(f"🧪 Total test cases: {total_tests}")
        print(f"{'='*70}\n")

        return all_valid


def main():
    """Main entry point"""
    prompts_dir = Path('prompts')

    if not prompts_dir.exists():
        print(f"❌ Prompts directory not found: {prompts_dir}")
        sys.exit(1)

    validator = PromptValidator()
    success = validator.validate_all_prompts(prompts_dir)

    # Save results as JSON for GitHub Actions
    results_file = Path('test-results.json')
    with open(results_file, 'w') as f:
        json.dump(validator.results, f, indent=2)

    print(f"📄 Results saved to: {results_file}")

    if not success:
        print("\n❌ Validation failed!")
        sys.exit(1)
    else:
        print("\n✅ All prompts validated successfully!")
        sys.exit(0)


if __name__ == '__main__':
    main()
