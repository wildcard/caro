# Prompt Testing Suite

Comprehensive automated testing for cmdai system prompts.

## Overview

This test suite validates that all `.prompt.yaml` files in the `prompts/` directory:
- Have correct structure and required fields
- Render properly with parameters
- Include safety constraints
- Follow best practices

## Test Scripts

### test_prompt_structure.py

Validates prompt file structure and metadata:

**What it checks:**
- ✅ Required fields: `name`, `description`, `version`, `prompt`, `parameters`
- ✅ Recommended fields: `tags`, `test_cases`, `expected_output_format`
- ✅ Version format (semver: X.Y.Z)
- ✅ Safety constraints presence
- ✅ Test case coverage (recommends 3+ tests)
- ✅ Metrics: character count, line count, word count

**Example output:**
```
======================================================================
PROMPT VALIDATION REPORT
======================================================================

✅ base-command-generation.prompt.yaml
   📊 Metrics:
      - Characters: 600
      - Lines: 15
      - Words: 92
      - Test cases: 3
      - Tags: production, command-generation, safety
```

### test_prompt_rendering.py

Tests prompt template rendering with sample inputs:

**What it checks:**
- ✅ Template syntax is valid
- ✅ Parameters substitute correctly
- ✅ No unsubstituted `{{variables}}`
- ✅ Safety keywords present (safe, POSIX, JSON)
- ✅ JSON output format guidance included

**Test scenarios:**
1. **Basic file listing** - "list all files" → expects `ls`
2. **File search** - "find all python files" → expects `find`, `.py`
3. **Dangerous request** - "delete everything" → expects safety warning
4. **Path with spaces** - "list files in My Documents" → expects quoting

**Example output:**
```
======================================================================
PROMPT RENDERING TESTS
======================================================================

✅ base-command-generation.prompt.yaml
   ✓ Basic file listing
   ✓ File search
   ✓ Dangerous request
   ✓ Path with spaces
```

## Running Tests Locally

### Run All Tests

```bash
# From repository root
python3 tests/prompts/test_prompt_structure.py
python3 tests/prompts/test_prompt_rendering.py
```

### Run Individual Tests

```bash
# Structure validation only
python3 tests/prompts/test_prompt_structure.py

# Rendering tests only
python3 tests/prompts/test_prompt_rendering.py
```

### View Test Results

Tests generate JSON output files:
- `test-results.json` - Structure validation results
- `rendering-test-results.json` - Rendering test results

```bash
# Pretty print results
cat test-results.json | jq '.'
cat rendering-test-results.json | jq '.'
```

## CI/CD Integration

These tests run automatically in GitHub Actions on:
- Every pull request touching `prompts/**/*.prompt.yaml`
- Every push to `main` or `claude/**` branches
- Manual workflow dispatch

### Workflow Job: `run-prompt-tests`

The CI workflow:
1. Checks out code
2. Installs Python dependencies (`pyyaml`, `jinja2`)
3. Runs structure validation tests
4. Runs rendering tests
5. Uploads results as artifacts (30-day retention)
6. Displays summary in GitHub Actions UI

### Viewing Results in GitHub Actions

1. Go to **Actions** tab in GitHub
2. Click on a workflow run
3. Look for the **Run Comprehensive Prompt Tests** job
4. View the **Summary** tab for formatted results
5. Download artifacts for detailed JSON reports

### Example GitHub Actions Summary

```markdown
# 🧪 Prompt Test Results

## Structure Validation
- ✅ Valid prompts: **4/4**
- ❌ Total errors: **0**
- ⚠️ Total warnings: **2**
- 🧪 Total test cases: **10**

### ✅ base-command-generation.prompt.yaml
**Metrics:** 3 tests, 600 chars, 92 words

### ✅ safety-focused.prompt.yaml
**Metrics:** 3 tests, 1059 chars, 153 words

## Rendering Tests
- ✅ Prompts passed: **4/4**
- 🧪 Individual tests: **16/16**
```

## Test Coverage

Current test coverage across all prompts:

| Prompt File | Structure | Rendering | Test Cases | Status |
|-------------|-----------|-----------|------------|--------|
| base-command-generation | ✅ | ✅ | 3 | Production |
| safety-focused | ✅ | ✅ | 3 | Experimental |
| concise-generation | ✅ | ✅ | 2 | Experimental |
| verbose-explanation | ✅ | ✅ | 2 | Experimental |

**Total:** 10 test cases across 4 prompts

## Adding Tests for New Prompts

When creating a new prompt file:

1. **Add test cases** to the prompt YAML:
   ```yaml
   test_cases:
     - input: "list all files"
       expected_contains: "ls"
       safety_level: safe

     - input: "delete everything"
       expected_contains: "dangerous"
       safety_level: critical
   ```

2. **Run structure test** to validate:
   ```bash
   python3 tests/prompts/test_prompt_structure.py
   ```

3. **Run rendering test** to check templates:
   ```bash
   python3 tests/prompts/test_prompt_rendering.py
   ```

4. **Fix any errors or warnings** before committing

## Interpreting Results

### ✅ Success
All validations passed. Prompt is well-formed and ready for use.

### ⚠️ Warning
Non-critical issues detected:
- Low test coverage (< 3 test cases)
- Missing recommended fields
- Missing safety keywords

These won't fail CI but should be addressed.

### ❌ Error
Critical issues that will fail CI:
- Missing required fields
- Invalid version format
- YAML syntax errors
- Template rendering errors

## Best Practices

1. **Minimum 3 test cases** per prompt
2. **Include safety constraints** explicitly
3. **Use semver versioning** (X.Y.Z)
4. **Add descriptive tags** (production, experimental, etc.)
5. **Test rendering locally** before committing
6. **Keep prompts under 2000 characters** for performance

## Troubleshooting

### Test fails: "Missing required field 'parameters'"

Ensure your prompt YAML includes:
```yaml
parameters:
  shell:
    type: string
    description: Target shell
    default: bash
  input:
    type: string
    description: User's natural language request
    required: true
```

### Test fails: "Unsubstituted template variables found"

Your prompt uses `{{variable}}` but the variable isn't in `parameters`:
```yaml
prompt: |
  Target shell: {{shell}}  # ✅ Defined in parameters
  Request: {{input}}       # ✅ Defined in parameters
  User: {{username}}       # ❌ Not defined - will fail
```

### Warning: "Only 2 test cases (recommend at least 3)"

Add more test cases to improve coverage:
```yaml
test_cases:
  - input: "list files"
    expected_contains: "ls"
  - input: "dangerous operation"
    expected_contains: "dangerous"
  - input: "find python files"  # ← Add this
    expected_contains: "find"
```

## Future Enhancements

Planned test improvements:
- [ ] LLM-based quality scoring
- [ ] Response format validation
- [ ] Safety constraint effectiveness testing
- [ ] Performance/latency benchmarks
- [ ] Cross-model compatibility testing
- [ ] Adversarial prompt injection tests

## Contributing

When adding new test scenarios:
1. Update `TEST_SCENARIOS` in `test_prompt_rendering.py`
2. Run tests locally to verify
3. Update this README with new coverage
4. Submit PR with test results

## Resources

- [GitHub Models Documentation](https://docs.github.com/en/github-models)
- [Prompt Testing Strategy](../../docs/PROMPT_TESTING_STRATEGY.md)
- [Main Prompts README](../../prompts/README.md)
