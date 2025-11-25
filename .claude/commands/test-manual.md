---
description: Launch interactive manual testing system that provides test cases, executes them, inspects logs, and discusses results with the user in an interactive process.
---

The user input to you can be provided directly by the agent or as a command argument - you **MUST** consider it before proceeding with the prompt (if not empty).

User input:

$ARGUMENTS

## Manual Testing Interactive Process

You are now launching the cmdai manual testing system. This is an interactive process that involves:

1. **Test Case Selection** - Present available test categories and let user choose
2. **Test Case Planning** - Show specific test cases in the selected category
3. **Command Planning** - Display the commands that will be tested
4. **Command Execution** - Run the test cases using the cmdai testing infrastructure
5. **Log Inspection** - Analyze logs and performance metrics from test execution
6. **Result Discussion** - Discuss results, improvements, and next steps with the user
7. **Iterative Enhancement** - Suggest fixes and enhancements based on findings

## Implementation Steps

### Step 1: Initialize Testing Environment

First, check if the testing infrastructure is available and properly set up:

```bash
# Ensure we're in the correct directory
pwd

# Check if the testing module exists
find . -name "testing" -type d

# Verify cmdai binary can be built
cargo check --lib
```

### Step 2: Present Test Categories

Show the user available test categories from the cmdai testing system:

- **Basic Safety** (`basic`) - Fundamental safety validation tests
- **Dangerous Commands** (`dangerous`) - Tests for dangerous command detection
- **Edge Cases** (`edge`) - Boundary condition and error handling tests  
- **Adaptive Learning** (`learning`) - Tests for system learning and adaptation
- **Performance Benchmarks** (`performance`) - Performance and timing tests
- **Integration Tests** (`integration`) - End-to-end workflow tests
- **Custom User Tests** (`custom`) - User-defined test scenarios

Ask the user which category they'd like to test, or if they want to run all categories.

### Step 3: Display Test Plan

Based on the user's selection, show:
- Specific test cases that will be executed
- Expected outcomes for each test
- Commands that will be generated and validated
- Safety levels and risk assessments involved
- Performance benchmarks that will be measured

### Step 4: Execute Test Cases

Run the selected test category using the cmdai testing infrastructure:

```bash
# Build the project first
cargo build

# Run specific test category if user specified one
# Example: cargo run -- --test
# Or use the interactive testing system built into cmdai
```

Use the Rust testing infrastructure from `src/testing/` which includes:
- `ManualTestRunner` for orchestrating tests
- `TestCase` implementations for different scenarios
- `LogAnalyzer` for parsing test outputs
- `MetricsCollector` for performance data
- `InteractiveReporter` for user communication

### Step 5: Inspect Logs and Metrics

After test execution, analyze:

1. **Test Results**: Parse the test outcomes (pass/fail/error)
2. **Performance Metrics**: Review timing, memory usage, resource consumption
3. **Safety Validation**: Check safety validation results and risk assessments
4. **Log Patterns**: Look for error patterns, warnings, or anomalies
5. **Command Generation Quality**: Assess generated command accuracy and safety

Present findings using:
- Colored output for easy reading
- Performance graphs and statistics
- Safety analysis summaries
- Detailed error reports if any issues found

### Step 6: Discuss Results and Improvements

Engage the user in discussion about:

1. **What worked well**: Highlight successful test cases and good performance
2. **Issues found**: Explain any failures, errors, or performance problems
3. **Root cause analysis**: Investigate why certain tests failed
4. **Improvement suggestions**: Propose fixes, optimizations, or enhancements
5. **Next steps**: Recommend follow-up testing or code changes

### Step 7: Interactive Feedback Loop

Offer the user options to:
- Run additional test categories
- Modify test parameters and re-run
- Focus on specific failing tests
- Implement suggested improvements
- Add new custom test cases
- Export test results for documentation

## Command Usage Examples

Handle user arguments for specific test categories:

- If `$ARGUMENTS` contains `basic`: Run basic safety tests
- If `$ARGUMENTS` contains `dangerous`: Run dangerous command tests  
- If `$ARGUMENTS` contains `performance`: Run performance benchmarks
- If `$ARGUMENTS` contains `all`: Run all test categories
- If `$ARGUMENTS` is empty: Present interactive menu

## Safety and Error Handling

Throughout the process:
- Always validate commands before execution
- Use appropriate safety levels for testing
- Handle test failures gracefully
- Provide clear error messages and recovery options
- Maintain test isolation to prevent side effects
- Log all test activities for audit trails

## Integration with cmdai Features

Leverage existing cmdai functionality:
- Safety validation system for command checking
- Backend selection for testing different inference engines
- Configuration management for test parameters
- Performance monitoring for test metrics
- Session management for interactive state

This command creates a comprehensive, interactive testing experience that helps users understand cmdai's capabilities, identify issues, and improve the system through systematic testing and analysis.