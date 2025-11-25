# PRD: Command Accuracy Evaluation Framework

## Executive Summary

**Project**: cmdai Command Accuracy Evaluation Framework  
**Goal**: Create a comprehensive, automated evaluation system to measure and improve command generation accuracy using organized datasets and systematic testing.  
**Priority**: High - Essential for maintaining command quality as we transition from hardcoded patterns to real LLM inference  

## Problem Statement

### Current Challenges
1. **Manual Testing Burden**: Currently relying on manual CLI testing and ad-hoc test cases
2. **Limited Coverage**: Test cases don't cover the full spectrum of shell command scenarios
3. **No Baseline Metrics**: Lack of standardized accuracy metrics and benchmarks
4. **Regression Risk**: As we improve models/prompts, we need to ensure we don't break existing functionality
5. **Cross-Platform Gaps**: Testing across different shells (Bash, PowerShell, Cmd) is inconsistent

### Success Criteria
- **Automated Evaluation**: Run comprehensive accuracy tests with single command
- **90%+ Accuracy**: Achieve >90% accuracy on core command generation tasks
- **Cross-Platform**: Validate across Bash, PowerShell, and Cmd environments
- **Regression Detection**: Automatically detect accuracy regressions in CI/CD
- **Performance Benchmarks**: Track inference time and quality metrics

## Solution Overview

### Core Components

#### 1. Evaluation Dataset (`qa/datasets/`)
**Structured test cases covering all command categories:**

```
qa/datasets/
├── core-commands/           # Basic shell operations
│   ├── file-operations.yaml
│   ├── directory-navigation.yaml
│   └── system-information.yaml
├── file-search/            # File finding and filtering
│   ├── by-extension.yaml
│   ├── by-size.yaml
│   ├── by-date.yaml
│   └── complex-queries.yaml
├── text-processing/        # grep, awk, sed operations
│   ├── pattern-matching.yaml
│   ├── text-transformation.yaml
│   └── log-analysis.yaml
├── system-admin/           # System administration
│   ├── process-management.yaml
│   ├── network-operations.yaml
│   └── permissions.yaml
├── shell-specific/         # Platform-specific commands
│   ├── bash-specific.yaml
│   ├── powershell-specific.yaml
│   └── cmd-specific.yaml
└── edge-cases/            # Complex and ambiguous cases
    ├── ambiguous-requests.yaml
    ├── multi-step-operations.yaml
    └── error-cases.yaml
```

#### 2. Evaluation Engine (`src/evaluation/`)
**Automated testing and scoring system:**

```rust
pub struct EvaluationEngine {
    dataset_loader: DatasetLoader,
    accuracy_scorer: AccuracyScorer,
    performance_monitor: PerformanceMonitor,
    regression_detector: RegressionDetector,
}

pub struct TestCase {
    pub id: String,
    pub category: CommandCategory,
    pub shell: ShellType,
    pub input: String,
    pub expected_commands: Vec<String>,  // Multiple valid answers
    pub explanation: String,
    pub difficulty: DifficultyLevel,
    pub tags: Vec<String>,
}

pub struct EvaluationResult {
    pub accuracy_score: f64,
    pub performance_metrics: PerformanceMetrics,
    pub category_breakdown: HashMap<CommandCategory, f64>,
    pub shell_compatibility: HashMap<ShellType, f64>,
    pub failed_cases: Vec<FailedTestCase>,
}
```

#### 3. Scoring Metrics
**Multi-dimensional accuracy measurement:**

- **Exact Match**: Command exactly matches expected output
- **Semantic Equivalence**: Commands achieve same result (e.g., `find . -name "*.txt"` vs `find . -iname "*.txt"`)
- **Functional Correctness**: Command safely accomplishes the requested task
- **POSIX Compliance**: Follows POSIX standards for portability
- **Safety Score**: Avoids dangerous operations

#### 4. Benchmarking Suite
**Performance and quality benchmarks:**

```yaml
benchmarks:
  accuracy_targets:
    core_commands: 95%
    file_search: 90%
    text_processing: 85%
    system_admin: 80%
    edge_cases: 70%
  
  performance_targets:
    avg_inference_time: <2s
    p95_inference_time: <5s
    startup_time: <100ms
    memory_usage: <50MB
  
  regression_thresholds:
    accuracy_drop: -5%
    performance_degradation: +50%
```

## Technical Implementation

### Phase 1: Dataset Creation (Week 1)
1. **Core Command Dataset**: 200+ test cases covering basic operations
2. **File Search Dataset**: 150+ test cases with size/type/date filters
3. **Shell-Specific Dataset**: 100+ test cases per shell (Bash, PowerShell, Cmd)
4. **YAML Schema**: Standardized format for test case definition

### Phase 2: Evaluation Engine (Week 2)
1. **Dataset Loader**: Parse YAML test cases into structured data
2. **Command Executor**: Run generated commands in sandboxed environment
3. **Accuracy Scorer**: Implement multiple scoring algorithms
4. **Report Generator**: Create detailed accuracy reports

### Phase 3: CI/CD Integration (Week 3)
1. **GitHub Actions**: Automated evaluation on every PR
2. **Performance Tracking**: Track accuracy trends over time
3. **Regression Detection**: Alert on accuracy drops
4. **Benchmark Dashboard**: Web interface for viewing results

### Phase 4: Advanced Features (Week 4)
1. **Semantic Equivalence**: AI-powered command similarity detection
2. **Adversarial Testing**: Generate challenging edge cases
3. **Cross-Model Comparison**: Compare different backend models
4. **Human Annotation**: Crowdsource validation of edge cases

## Dataset Schema

### Test Case Format
```yaml
test_cases:
  - id: "pdf_size_filter_001"
    category: "file_search"
    subcategory: "size_filtering"
    shell: "bash"
    difficulty: "intermediate"
    input: "list all pdf files size less than 5mb"
    expected_commands:
      - "find . -type f -iname \"*.pdf\" -size -5M"
      - "find . -name \"*.pdf\" -type f -size -5M"  # Alternative valid form
    explanation: "Find PDF files under 5MB using size constraint"
    tags: ["pdf", "size", "file_search", "find"]
    safety_level: "safe"
    
  - id: "img_size_filter_002" 
    category: "file_search"
    subcategory: "size_filtering"
    shell: "bash"
    difficulty: "advanced"
    input: "find img files greater than 10mb"
    expected_commands:
      - "find . -type f \\( -iname \"*.jpg\" -o -iname \"*.jpeg\" -o -iname \"*.png\" -o -iname \"*.gif\" -o -iname \"*.bmp\" -o -iname \"*.tiff\" \\) -size +10M"
    explanation: "Find image files over 10MB with multiple file extensions"
    tags: ["image", "size", "file_search", "find", "multiple_extensions"]
    safety_level: "safe"
```

### Category Taxonomy
```yaml
categories:
  core_commands:
    - file_operations      # ls, cp, mv, rm
    - directory_navigation  # cd, pwd, mkdir
    - system_information   # whoami, date, uname
    
  file_search:
    - by_extension         # *.txt, *.pdf, etc.
    - by_size             # -size +10M, -size -5M
    - by_date             # -mtime, -newer
    - complex_queries     # Combined criteria
    
  text_processing:
    - pattern_matching    # grep, egrep
    - text_transformation # sed, awk
    - log_analysis       # tail, head, sort
    
  system_admin:
    - process_management  # ps, kill, top
    - network_operations  # ping, wget, curl
    - permissions        # chmod, chown
    
  shell_specific:
    - bash_features      # Bash-specific syntax
    - powershell_cmdlets # PowerShell commands
    - cmd_operations     # Windows CMD commands
```

## Success Metrics

### Accuracy Targets
- **Core Commands**: 95% accuracy (ls, cd, pwd, etc.)
- **File Search**: 90% accuracy (find with various filters)
- **Text Processing**: 85% accuracy (grep, sed, awk)
- **System Admin**: 80% accuracy (ps, chmod, etc.)
- **Edge Cases**: 70% accuracy (ambiguous/complex requests)

### Performance Targets
- **Inference Time**: <2s average, <5s p95
- **Memory Usage**: <50MB peak
- **Startup Time**: <100ms
- **Test Suite Runtime**: <30s for full evaluation

### Quality Gates
- **Regression Threshold**: <5% accuracy drop on any category
- **Safety Requirements**: 0% dangerous command generation
- **POSIX Compliance**: 100% compliance for portable commands

## Risk Assessment

### Technical Risks
- **Semantic Equivalence Complexity**: Determining if two different commands achieve the same result
- **Shell Environment Differences**: Commands may behave differently across platforms
- **Dataset Bias**: Test cases may not represent real-world usage patterns

### Mitigation Strategies
- **Multiple Expected Answers**: Allow several valid command variants per test case
- **Sandboxed Execution**: Test commands in isolated environments
- **Community Validation**: Crowdsource validation of edge cases
- **Continuous Dataset Updates**: Regular addition of real user queries

## Implementation Timeline

### Week 1: Dataset Foundation
- Day 1-2: Define YAML schema and category taxonomy
- Day 3-5: Create core command dataset (200+ cases)
- Day 6-7: Add file search and shell-specific datasets

### Week 2: Evaluation Engine
- Day 1-3: Implement dataset loader and test case parsing
- Day 4-5: Build accuracy scoring algorithms
- Day 6-7: Create evaluation runner and report generator

### Week 3: Automation & CI/CD
- Day 1-3: GitHub Actions integration
- Day 4-5: Performance tracking and regression detection
- Day 6-7: Dashboard and alerting system

### Week 4: Advanced Features
- Day 1-3: Semantic equivalence detection
- Day 4-5: Adversarial testing and edge case generation
- Day 6-7: Cross-model comparison framework

## Future Enhancements

### Advanced Evaluation
- **LLM-as-a-Judge**: Use GPT-4 to evaluate semantic equivalence
- **Execution Validation**: Actually run commands and verify outputs
- **User Feedback Loop**: Integrate real user corrections
- **Continuous Learning**: Update datasets based on user interactions

### Specialized Testing
- **Security Testing**: Red-team testing for injection attacks
- **Accessibility Testing**: Voice-to-command evaluation
- **Multilingual Support**: Non-English command requests
- **Domain-Specific**: Specialized datasets for DevOps, Data Science, etc.

## Conclusion

This evaluation framework will provide the foundation for maintaining and improving cmdai's command generation accuracy. By implementing comprehensive testing, automated evaluation, and continuous monitoring, we can ensure that cmdai consistently produces high-quality, safe, and accurate shell commands across all supported platforms.

The structured approach will enable data-driven improvements to our models and prompts while providing confidence that changes don't introduce regressions. This foundation will be critical as we scale from hardcoded patterns to real LLM inference.