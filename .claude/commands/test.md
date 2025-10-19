---
description: Interactive manual testing for cmdai - provides test cases, executes them, inspects logs, and discusses results with the user.
---

User input: $ARGUMENTS

## cmdai Interactive Manual Testing

Launch the comprehensive manual testing system for cmdai that implements the complete interactive testing workflow.

### Quick Start

1. **Initialize**: Set up testing environment and verify cmdai build status
2. **Select**: Choose test category (basic, dangerous, edge, learning, performance, integration, custom)
3. **Plan**: Review test cases and expected outcomes  
4. **Execute**: Run tests using cmdai's built-in testing infrastructure
5. **Analyze**: Inspect logs, metrics, and safety validation results
6. **Discuss**: Interactive review of results and improvement recommendations
7. **Iterate**: Option to run more tests, implement fixes, or add custom tests

### Test Categories Available

- `basic` - Basic safety validation tests
- `dangerous` - Dangerous command detection tests  
- `edge` - Edge cases and error handling
- `learning` - Adaptive learning capabilities
- `performance` - Performance and timing benchmarks
- `integration` - End-to-end workflow tests
- `custom` - User-defined test scenarios
- `all` - Run all test categories

### Usage

- `/test` - Launch interactive test selection menu
- `/test basic` - Run basic safety tests directly
- `/test dangerous` - Test dangerous command detection
- `/test performance` - Run performance benchmarks
- `/test all` - Execute all test categories

### Implementation

Use the cmdai testing infrastructure from `src/testing/`:
- Build project with `cargo build`
- Execute tests using `ManualTestRunner`
- Analyze results with `LogAnalyzer` and `MetricsCollector`
- Present findings with `InteractiveReporter`
- Engage user in discussion about results and improvements

This provides the complete testing workflow: test case selection → planning → execution → log inspection → result discussion → enhancements, exactly as specified in the original request.