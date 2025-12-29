## Description

<!-- Provide a clear and concise description of what this PR does -->

### Motivation

<!-- Why are these changes needed? What problem do they solve? -->

### Changes Made

<!-- List the key changes in this PR -->

-
-
-

---

## Type of Change

<!-- Mark the relevant option(s) with an 'x' -->

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Refactoring (code restructuring without changing behavior)
- [ ] Documentation update (changes to docs, comments, or guides)
- [ ] Performance improvement (makes code faster or more efficient)
- [ ] Test coverage (adds or improves tests)
- [ ] CI/CD or tooling (changes to build, release, or development tools)

---

## Checklist

<!-- Ensure you have completed these steps before requesting review -->

### Code Quality

- [ ] I have run `cargo fmt --all` (code is properly formatted)
- [ ] I have run `cargo clippy -- -D warnings` (no clippy warnings)
- [ ] I have run `cargo test` (all tests pass)
- [ ] I have run `cargo audit` (no security vulnerabilities in dependencies)

### Developer Certificate of Origin

- [ ] All commits are signed off (`git commit -s`) per the [DCO](https://developercertificate.org/)

### Testing

- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] I have added contract tests for new public APIs (if applicable)
- [ ] I have added integration tests for cross-module workflows (if applicable)
- [ ] I have verified performance requirements are met (if applicable)

### Documentation

- [ ] I have added rustdoc comments for new public APIs
- [ ] I have updated relevant documentation (README, specs, guides)
- [ ] I have added examples to demonstrate new functionality (if applicable)
- [ ] I have updated the CHANGELOG.md with my changes

### TDD Workflow

- [ ] I followed the Red-Green-Refactor cycle (if implementing new features)
- [ ] I wrote failing tests before implementing the solution
- [ ] I verified tests with `cargo watch -x test` during development

---

## Breaking Changes

<!-- If this PR includes breaking changes, describe them here -->

### API Changes

<!-- List any changes to public APIs, function signatures, or module structure -->

### Migration Guide

<!-- Provide step-by-step instructions for users to migrate from the old behavior to the new behavior -->

### Deprecation Plan

<!-- If applicable, describe how old APIs will be deprecated and when they will be removed -->

---

## Related Issues and Specs

<!-- Link to related issues, feature requests, or specifications -->

- Closes #<!-- issue number -->
- Related to #<!-- issue number -->
- Implements spec: `specs/<!-- feature-id -->/spec.md`
- Addresses contract: `specs/<!-- feature-id -->/contracts/<!-- contract-name -->.md`

---

## Performance Impact

<!-- Describe any performance implications of this PR -->

### Benchmarks

<!-- If applicable, include benchmark results comparing before/after -->

```
cargo bench

Before:
- Startup time: 95ms
- Validation time: 45ms

After:
- Startup time: 85ms (-10.5%)
- Validation time: 38ms (-15.6%)
```

### Binary Size

<!-- If applicable, report changes to release binary size -->

```
Before: 47.2 MB
After:  48.1 MB (+0.9 MB)
```

### Memory Usage

<!-- If applicable, report changes to memory consumption -->

---

## Screenshots / Examples

<!-- For CLI changes, include before/after screenshots or terminal output -->

### Before

```bash
$ caro "list files"
Error: command generation failed
```

### After

```bash
$ caro "list files"
Generated command:
  find . -type f -name "*"

Execute this command? (y/N)
```

---

## Testing Evidence

<!-- Provide evidence that you've tested your changes -->

### Test Output

<!-- Paste relevant test output showing all tests pass -->

```
$ cargo test
    Finished test [unoptimized + debuginfo] target(s) in 2.34s
     Running unittests src/lib.rs (target/debug/deps/caro-...)

running 42 tests
test cache::tests::test_cache_manager_retrieves_model ... ok
test safety::tests::test_dangerous_pattern_detection ... ok
...
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Manual Testing

<!-- Describe manual testing you performed -->

- [ ] Tested on macOS (Apple Silicon / Intel)
- [ ] Tested on Linux (Ubuntu / Fedora / Arch)
- [ ] Tested on Windows (10 / 11)
- [ ] Tested with different backends (MLX / vLLM / Ollama / Mock)
- [ ] Tested edge cases and error conditions

---

## Additional Context

<!-- Any other information that reviewers should know -->

### Technical Decisions

<!-- Explain any non-obvious technical decisions or trade-offs -->

### Future Work

<!-- List any follow-up work that should be done in future PRs -->

### Questions for Reviewers

<!-- Any specific feedback you're looking for -->

---

## Reviewer Checklist

<!-- For maintainers reviewing this PR -->

- [ ] Code follows Rust best practices and project conventions
- [ ] Tests are comprehensive and follow TDD principles
- [ ] Documentation is clear and complete
- [ ] Changes align with project specifications
- [ ] Performance impact is acceptable
- [ ] Breaking changes are justified and documented
- [ ] Security implications have been considered

---

**By submitting this PR, I confirm that:**

- [ ] My code follows the style guidelines of this project (see [AGENTS.md](https://github.com/wildcard/caro/blob/main/docs/development/AGENTS.md))
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] My changes generate no new warnings or errors
- [ ] I have read and followed the [contributing guidelines](https://github.com/wildcard/caro/blob/main/CONTRIBUTING.md)
- [ ] I agree to the [Code of Conduct](https://github.com/wildcard/caro/blob/main/CODE_OF_CONDUCT.md)
- [ ] All my commits are signed off under the [Developer Certificate of Origin](https://developercertificate.org/)

---

<!-- Thank you for contributing to caro! -->
