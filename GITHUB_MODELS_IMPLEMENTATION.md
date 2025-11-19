# GitHub Models Integration - Implementation Summary

## Overview

This implementation adds comprehensive prompt testing infrastructure using GitHub Models, enabling systematic evaluation and optimization of cmdai's natural language to shell command conversion prompts.

## What Was Added

### 1. Prompt Files (`prompts/` directory)

Four production-ready `.prompt.yml` files for GitHub Models:

#### `base-command-generation.prompt.yml`
- **Status**: Production
- **Purpose**: Current system prompt used across all backends
- **Features**: JSON-only output, POSIX compliance, basic safety
- **Use case**: Default prompt for general command generation

#### `safety-focused.prompt.yml`
- **Status**: Experimental
- **Purpose**: Enhanced safety-first variant
- **Features**: Stricter safety rules, explicit risk assessment, read-only preference
- **Use case**: High-security environments, enterprise deployments

#### `concise-generation.prompt.yml`
- **Status**: Experimental
- **Purpose**: Performance-optimized minimal token variant
- **Features**: Reduced prompt length, faster inference (<1s target)
- **Use case**: Performance-critical applications, resource-constrained environments

#### `verbose-explanation.prompt.yml`
- **Status**: Experimental
- **Purpose**: Educational variant with explanations
- **Features**: Extended output format: `{"cmd": "...", "explanation": "..."}`
- **Use case**: User-facing applications, educational tools, debugging

### 2. Documentation

#### `prompts/README.md`
Comprehensive prompt documentation including:
- Detailed description of each variant
- Usage guidelines and when to use each prompt
- GitHub Models testing instructions (UI and CLI)
- Integration architecture with cmdai backends
- Prompt versioning strategy
- Benchmark results template
- Contributing guidelines

#### `prompts/QUICKSTART.md`
5-minute getting started guide:
- Step-by-step GitHub UI walkthrough
- Command-line testing with `gh` CLI
- Common test scenarios
- Output evaluation criteria
- Troubleshooting tips

#### `docs/PROMPT_TESTING_STRATEGY.md`
In-depth testing strategy document:
- Why prompt testing matters
- Complete testing infrastructure overview
- Evaluation methodology (functional, safety, format, edge cases, performance)
- Metrics and targets (accuracy >90%, safety >95%, etc.)
- Continuous integration approach
- Best practices for prompt engineering
- Troubleshooting common issues

### 3. CI/CD Automation

#### `.github/workflows/prompt-evaluation.yml`
Comprehensive GitHub Actions workflow that runs on:
- **Pull requests** modifying prompt files
- **Pushes** to main or claude/* branches
- **Manual dispatch** with custom parameters

**Jobs:**
1. **validate-prompts** - YAML syntax, required fields, version format
2. **test-prompt-rendering** - Parameter substitution, template validation
3. **analyze-prompt-metrics** - Character/token counts, safety keyword analysis
4. **security-check** - Prompt injection vulnerability detection
5. **github-models-eval** - Optional evaluation using GitHub Models API
6. **summary** - Aggregated results and next steps

### 4. Main README Update

Added new section: "Prompt Engineering & Testing" that:
- Introduces GitHub Models integration
- Provides quick start commands
- Links to all documentation
- Highlights 4 prompt variants

## How It Works

### For Developers

1. **Manual Testing (GitHub UI)**
   - Navigate to Models tab → Prompts
   - Select a `.prompt.yml` file
   - Click Run to test against different models
   - Compare outputs side-by-side

2. **Automated Testing (CLI)**
   ```bash
   # Single prompt
   gh models eval prompts/base-command-generation.prompt.yml

   # All prompts
   gh models eval prompts/*.prompt.yml

   # Specific model
   gh models eval prompts/base-command-generation.prompt.yml --model gpt-4
   ```

3. **CI/CD Pipeline**
   - Automatically runs on PRs touching prompt files
   - Validates syntax, security, and rendering
   - Generates metrics and analysis reports

### For the Project

Current cmdai backends (Ollama, vLLM) have `create_system_prompt()` methods that use embedded prompts. Future enhancement:

```rust
// Proposed: Load prompts dynamically
fn create_system_prompt(&self, request: &CommandRequest) -> String {
    let template = load_prompt_template("base-command-generation")?;
    template.render(request)
}
```

This allows:
- A/B testing different prompts in production
- Runtime prompt switching
- User-selectable prompt variants
- Continuous prompt optimization based on GitHub Models data

## Test Coverage

Each prompt includes test cases covering:

- ✅ **Basic functionality** - "list all files" → `ls`
- ✅ **Safety boundaries** - "delete everything" → blocked/warning
- ✅ **Edge cases** - Ambiguous requests, empty input, special characters
- ✅ **Path quoting** - Spaces in filenames → properly quoted
- ✅ **POSIX compliance** - Only portable utilities used

## Evaluation Metrics

We track these metrics for each prompt variant:

| Metric | Target | Description |
|--------|--------|-------------|
| Accuracy | >90% | Commands match user intent |
| Safety Score | >95% | Dangerous requests blocked |
| JSON Compliance | >98% | Valid JSON responses |
| Latency P50 | <1s | Median response time |
| Latency P95 | <2s | 95th percentile response time |
| Token Efficiency | Minimize | Average tokens per prompt |
| POSIX Compliance | >95% | Portable commands |

## Benefits

### 1. Quality Assurance
- Catch prompt regressions before production
- Ensure consistent behavior across models
- Validate safety constraints systematically

### 2. Optimization
- Compare prompt variants objectively
- Identify performance bottlenecks
- Reduce token costs

### 3. Safety
- Automated security vulnerability scanning
- Dangerous command blocking verification
- Prompt injection attack detection

### 4. Collaboration
- Version-controlled prompts in Git
- Clear prompt evolution history
- Team can contribute improvements via PRs

### 5. Transparency
- All prompts visible to users
- Test cases document expected behavior
- Benchmarks show real performance data

## Next Steps

### Immediate (Can do now)

1. **Test existing prompts**
   ```bash
   gh models eval prompts/base-command-generation.prompt.yml
   ```

2. **Review benchmark results**
   - Update `prompts/README.md` with actual metrics
   - Compare across GPT-4, Claude, Llama models

3. **Create additional test cases**
   - Add more edge cases
   - Test dangerous command variations
   - Validate POSIX compliance

### Short-term (Next sprint)

1. **Dynamic prompt loading**
   - Refactor backends to load `.prompt.yml` files
   - Add prompt selection CLI flag: `--prompt safety-focused`
   - Implement runtime prompt switching

2. **Expand prompt variants**
   - Create domain-specific prompts (DevOps, data science, etc.)
   - Add language-specific variants
   - Test multi-step command generation

3. **Enhanced metrics**
   - Implement automated accuracy scoring
   - Track prompt performance over time
   - Create benchmarking dashboard

### Long-term (Future releases)

1. **User-customizable prompts**
   - Allow users to create custom `.prompt.yml` files
   - Prompt marketplace/sharing
   - Community-contributed variants

2. **Adaptive prompting**
   - Learn from user corrections
   - Context-aware prompt selection
   - Personalized prompt tuning

3. **Advanced evaluation**
   - Integration with promptfoo or similar tools
   - Adversarial testing framework
   - Red team prompt security testing

## Files Changed

```
New files:
├── .github/workflows/prompt-evaluation.yml  (CI/CD automation)
├── docs/PROMPT_TESTING_STRATEGY.md         (Comprehensive guide)
├── prompts/
│   ├── README.md                            (Prompts overview)
│   ├── QUICKSTART.md                        (5-min getting started)
│   ├── base-command-generation.prompt.yml   (Production prompt)
│   ├── safety-focused.prompt.yml            (Safety variant)
│   ├── concise-generation.prompt.yml        (Performance variant)
│   └── verbose-explanation.prompt.yml       (Educational variant)
└── GITHUB_MODELS_IMPLEMENTATION.md          (This file)

Modified files:
└── README.md                                 (Added testing section)
```

## Resources

- [GitHub Models Documentation](https://docs.github.com/en/github-models)
- [Storing Prompts in Repositories](https://docs.github.com/en/github-models/use-github-models/storing-prompts-in-github-repositories)
- [Evaluating AI Models](https://docs.github.com/en/github-models/use-github-models/evaluating-ai-models)

## Questions?

- Check [PROMPT_TESTING_STRATEGY.md](docs/PROMPT_TESTING_STRATEGY.md) for detailed answers
- See [QUICKSTART.md](prompts/QUICKSTART.md) for hands-on tutorials
- Open an issue for bugs or feature requests

---

**Implementation Date**: 2025-11-19
**Status**: Ready for review and testing
**Next Action**: Run `gh models eval` to start testing!
