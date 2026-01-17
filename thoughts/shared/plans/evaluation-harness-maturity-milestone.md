# Milestone: LLM Evaluation Harness Maturity & Quality Confidence

## Vision

Transform the evaluation harness from a basic testing framework into a comprehensive quality assurance platform that:
- Prevents regressions in command generation quality
- Provides actionable insights for model selection and tuning
- Establishes best practices for LLM evaluation in CLI tools
- Creates feedback loops from testing → prompting → fine-tuning → product

## Current State Assessment

### ✅ What We Have (v1.0 - Completed)
- **WP01-WP05**: Core infrastructure (models, dataset, evaluators, baseline, 100 test cases)
- **WP06**: cargo test integration with CLI
- **WP07**: GitHub Actions CI/CD automation
- **Baseline**: 31% pass rate with static_matcher backend
- **Categories**: Correctness, Safety, POSIX, MultiBackend

### ❌ What's Missing for Maturity

#### 1. **Backend Coverage** (Critical Gap)
- Current: Only static_matcher tested in CI
- Missing: MLX, Embedded (SmolLM, Qwen), Ollama backends
- Impact: No regression detection for actual LLM backends

#### 2. **Prompt Engineering Framework** (Strategic Gap)
- Current: Hard-coded prompts in backend implementations
- Missing: Systematic prompt versioning, A/B testing, optimization
- Impact: Can't measure prompt quality improvements

#### 3. **Model-Specific Insights** (Knowledge Gap)
- Current: Generic pass/fail metrics
- Missing: Per-model strengths/weaknesses analysis
- Impact: Can't make informed model selection decisions

#### 4. **Feedback Loop** (Product Integration Gap)
- Current: Results stored but not actionable
- Missing: Automated issue creation, pattern extraction
- Impact: Test failures don't drive product improvements

#### 5. **Fine-Tuning Integration** (Future Gap)
- Current: No connection to training pipeline
- Missing: Dataset export, training metrics correlation
- Impact: Can't measure fine-tuning effectiveness

#### 6. **Historical Trend Analysis** (Decision-Making Gap)
- Current: Single-run snapshots
- Missing: Time-series analysis, regression visualization
- Impact: Can't see quality trends over releases

#### 7. **Test Case Quality** (Coverage Gap)
- Current: 100 manually curated test cases
- Missing: Automated test generation, edge case discovery
- Impact: Limited coverage of real-world scenarios

#### 8. **Performance Benchmarking** (Efficiency Gap)
- Current: Basic execution time tracking
- Missing: Token efficiency, cost analysis, latency profiling
- Impact: Can't optimize for cost/performance trade-offs

## Milestone Structure

### Phase 1: Multi-Backend Validation (Weeks 1-2)
**Goal**: Establish regression detection for all production backends

#### WP09: Multi-Backend CI Matrix
- Add MLX backend to CI evaluation matrix
- Add embedded backends (SmolLM, Qwen) to matrix
- Add Ollama backend to matrix (optional, requires external service)
- Configure platform-specific runners (macOS for MLX, Linux for embedded)
- Implement graceful backend unavailability handling

**Deliverables**:
- [ ] `.github/workflows/evaluation.yml` updated with full matrix
- [ ] Backend availability detection in test harness
- [ ] Per-backend baseline files (`baselines/{backend}-main-latest.json`)
- [ ] CI fails on regression in any backend

**Success Criteria**:
- All 4 backends evaluated on every PR
- Baseline comparison for each backend
- <5 minute total evaluation time

---

### Phase 2: Prompt Engineering Framework (Weeks 3-4)
**Goal**: Systematic prompt optimization and versioning

#### WP10: Prompt Versioning System
- Create `prompts/` directory with versioned prompt files
- Implement prompt loader with version selection
- Add prompt metadata (author, date, target model, parameters)
- Support prompt templates with variable substitution

**Deliverables**:
- [ ] `src/prompts/registry.rs` - Prompt version management
- [ ] `prompts/v1.0/*.md` - Initial prompt versions
- [ ] CLI flag: `--prompt-version v1.0`
- [ ] Evaluation report includes prompt version used

#### WP11: Prompt A/B Testing
- Run evaluation with multiple prompt versions in parallel
- Generate comparison reports (which prompt performs better)
- Statistical significance testing (chi-square, p-value)
- Automated "winner" selection based on metrics

**Deliverables**:
- [ ] `src/evaluation/prompt_comparison.rs`
- [ ] CLI: `cargo test --test evaluation -- --compare-prompts v1.0,v1.1`
- [ ] Report: Side-by-side prompt performance comparison
- [ ] Recommendation: "Prompt v1.1 improves correctness by 12%"

**Success Criteria**:
- Can compare up to 5 prompt versions in single run
- Statistical confidence scores for differences
- Automated rollback if new prompt regresses

---

### Phase 3: Model-Specific Intelligence (Weeks 5-6)
**Goal**: Build knowledge base of model strengths/weaknesses

#### WP12: Per-Model Analysis
- Track which test cases each model fails
- Identify model-specific failure patterns
- Generate "model profile" reports
- Store profiles in knowledge base

**Deliverables**:
- [ ] `src/evaluation/model_profiling.rs`
- [ ] Per-model reports: `reports/model-profiles/{model}.md`
- [ ] Failure pattern extraction (e.g., "SmolLM fails on complex regex")
- [ ] Model recommendation system: "Use Qwen for POSIX commands"

#### WP13: Capability Matrix
- Create capability matrix (model × category → pass rate)
- Identify sweet spots for each model
- Generate routing recommendations
- Track capability improvements over versions

**Deliverables**:
- [ ] Capability matrix visualization
- [ ] Model selection guide for users
- [ ] Automated model routing based on query type
- [ ] Historical capability tracking

**Success Criteria**:
- Clear understanding of which model excels at what
- Can recommend optimal model for any query
- Capability matrix included in release notes

---

### Phase 4: Product Feedback Loops (Weeks 7-8)
**Goal**: Automated issue creation and pattern extraction

#### WP14: Automated Issue Creation
- Detect new failure patterns
- Automatically create GitHub issues for regressions
- Link issues to specific test cases and commits
- Assign priority based on failure severity

**Deliverables**:
- [ ] `src/evaluation/issue_automation.rs`
- [ ] GitHub API integration for issue creation
- [ ] Issue template: "Regression: {test_id} failing on {backend}"
- [ ] Automatic labeling (regression, model-specific, critical)

#### WP15: Pattern Extraction
- Extract common failure modes from test results
- Identify missing safety patterns from blocked commands
- Suggest new test cases based on failures
- Generate "lessons learned" reports

**Deliverables**:
- [ ] Failure pattern analyzer
- [ ] Safety pattern gap detector
- [ ] Test case suggestion system
- [ ] Weekly "insights" report

**Success Criteria**:
- Regressions automatically filed as GitHub issues
- Test suite grows organically from real failures
- Safety patterns updated based on evaluation results

---

### Phase 5: Fine-Tuning & LoRA Integration (Weeks 9-10)
**Goal**: Close the loop from evaluation → training → evaluation

#### WP16: Training Dataset Export
- Export evaluation dataset in fine-tuning formats
- Generate preference pairs from pass/fail results
- Create RLHF feedback dataset
- Support multiple formats (JSONL, Parquet, etc.)

**Deliverables**:
- [ ] `src/evaluation/export.rs` - Dataset exporters
- [ ] CLI: `cargo test --test evaluation -- --export-training-data`
- [ ] Formats: OpenAI fine-tuning, Anthropic Claude, local LoRA
- [ ] Preference pairs: (query, good_command, bad_command)

#### WP17: Training Metrics Correlation
- Track evaluation metrics before/after fine-tuning
- Correlate training loss with evaluation pass rate
- Identify overfitting vs generalization
- Generate training effectiveness reports

**Deliverables**:
- [ ] Fine-tuning experiment tracking
- [ ] Correlation analysis: loss vs pass rate
- [ ] Overfitting detection
- [ ] Training recommendations

**Success Criteria**:
- Can export evaluation data for fine-tuning
- Can measure fine-tuning effectiveness
- Clear correlation between training and evaluation metrics

---

### Phase 6: Advanced Analytics & Visualization (Weeks 11-12)
**Goal**: Decision-making dashboards and historical insights

#### WP18: Time-Series Analysis
- Store evaluation history in time-series database
- Track pass rate trends over releases
- Identify regression patterns
- Alert on anomalous drops

**Deliverables**:
- [ ] Time-series storage (SQLite or PostgreSQL)
- [ ] Historical trend charts
- [ ] Regression anomaly detection
- [ ] Email/Slack alerts on regressions

#### WP19: Interactive Dashboard (WP08 Enhanced)
- HTML dashboard with Chart.js visualizations
- Historical trend lines
- Model comparison heatmaps
- Drill-down to individual test failures
- Export to PDF for stakeholders

**Deliverables**:
- [ ] `src/evaluation/dashboard.rs`
- [ ] HTML templates with embedded charts
- [ ] Interactive filters (date range, backend, category)
- [ ] PDF export for release reports

**Success Criteria**:
- Dashboard viewable without internet
- All evaluation data explorable interactively
- Stakeholders can understand quality at a glance

---

### Phase 7: Test Coverage & Quality (Weeks 13-14)
**Goal**: Maximize test case coverage and quality

#### WP20: Automated Test Generation
- Generate test cases from real user queries (telemetry)
- Create edge cases using fuzzing techniques
- Discover failure modes with property-based testing
- Augment dataset automatically

**Deliverables**:
- [ ] Telemetry-to-test-case pipeline
- [ ] Property-based test generator
- [ ] Edge case fuzzer
- [ ] Automated dataset expansion

#### WP21: Test Case Quality Metrics
- Measure test case difficulty
- Identify redundant test cases
- Find coverage gaps
- Prioritize high-value tests

**Deliverables**:
- [ ] Test difficulty scoring
- [ ] Redundancy detector
- [ ] Coverage gap analysis
- [ ] Test case prioritization

**Success Criteria**:
- Dataset grows from 100 → 300+ test cases
- Coverage of long-tail scenarios
- High-quality, non-redundant test suite

---

### Phase 8: Performance & Cost Optimization (Weeks 15-16)
**Goal**: Optimize for token efficiency and cost

#### WP22: Token Efficiency Analysis
- Track tokens per evaluation (input + output)
- Calculate cost per backend
- Identify token-heavy test cases
- Optimize prompts for efficiency

**Deliverables**:
- [ ] Token usage tracking per backend
- [ ] Cost analysis dashboard
- [ ] Token efficiency recommendations
- [ ] Prompt optimization suggestions

#### WP23: Parallel Execution Optimization
- Batch evaluation requests
- Implement caching for repeated queries
- Optimize backend startup time
- Reduce CI evaluation time

**Deliverables**:
- [ ] Request batching
- [ ] Result caching
- [ ] Startup optimization
- [ ] Target: <2 minute total CI time

**Success Criteria**:
- Token usage reduced by 30%
- CI evaluation time <2 minutes
- Cost per evaluation <$0.10

---

## Best Practices & Standards

### 1. Evaluation Methodology
- **Baseline Management**: Update baselines only on main branch merges
- **Regression Threshold**: 5% drop in pass rate = regression
- **Statistical Significance**: p<0.05 for prompt comparisons
- **Minimum Sample Size**: 100 test cases per category

### 2. Prompt Engineering
- **Version Control**: All prompts in git with semantic versioning
- **A/B Testing**: Always compare against current champion
- **Documentation**: Every prompt version has rationale and changelog
- **Rollback Policy**: Revert if regression >10% in any category

### 3. Test Case Design
- **Categories**: Maintain balanced distribution (25% each)
- **Difficulty**: 40% easy, 40% medium, 20% hard
- **Real-World**: 80% from real usage, 20% synthetic edge cases
- **Coverage**: Each command type has ≥3 test cases

### 4. Model Evaluation
- **Multi-Backend**: All PRs evaluated on all backends
- **Platform-Specific**: MLX on macOS, others on Linux
- **Timeout**: 30s per test, 5min total evaluation
- **Graceful Degradation**: Skip unavailable backends

### 5. Reporting
- **Format**: JSON for machines, Markdown for humans
- **Artifacts**: 30-day retention in CI
- **Dashboards**: Auto-generated on every run
- **Notifications**: Slack on regression, email on release

---

## Success Metrics

### Short-Term (3 months)
- [ ] All 4 backends evaluated in CI
- [ ] Prompt A/B testing operational
- [ ] Model capability matrix published
- [ ] Automated issue creation working

### Mid-Term (6 months)
- [ ] Fine-tuning pipeline integrated
- [ ] Historical trend analysis available
- [ ] 300+ test cases in dataset
- [ ] <2 minute CI evaluation time

### Long-Term (12 months)
- [ ] Evaluation harness is industry-leading OSS example
- [ ] Published best practices guide
- [ ] Speaking engagements/blog posts on methodology
- [ ] Community contributions to test dataset

---

## Dependencies & Prerequisites

### Technical
- [ ] PostgreSQL for time-series storage
- [ ] GitHub Actions minutes budget
- [ ] MLX-capable macOS runner
- [ ] S3 or artifact storage for reports

### Organizational
- [ ] Telemetry opt-in for test generation
- [ ] Budget for LLM API costs
- [ ] Stakeholder buy-in for dashboard usage
- [ ] Engineering time allocation

---

## Risk Mitigation

### Risk 1: CI Cost Explosion
- **Mitigation**: Implement result caching, optimize token usage
- **Fallback**: Run full evaluation weekly, quick smoke tests on PR

### Risk 2: Backend Unavailability
- **Mitigation**: Graceful degradation, clear skip messages
- **Fallback**: Run offline backends in manual workflow

### Risk 3: Test Case Quality Decay
- **Mitigation**: Automated quality metrics, regular review
- **Fallback**: Quarterly manual test case audit

### Risk 4: Dashboard Complexity
- **Mitigation**: Start with MVP, add features incrementally
- **Fallback**: Keep simple terminal output as primary interface

---

## Timeline

```
┌─────────────────────────────────────────────────────────────┐
│ Evaluation Harness Maturity Roadmap (16 weeks)             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Weeks 1-2:   WP09 Multi-Backend CI Matrix                  │
│ Weeks 3-4:   WP10-11 Prompt Engineering Framework          │
│ Weeks 5-6:   WP12-13 Model-Specific Intelligence           │
│ Weeks 7-8:   WP14-15 Product Feedback Loops                │
│ Weeks 9-10:  WP16-17 Fine-Tuning Integration               │
│ Weeks 11-12: WP18-19 Advanced Analytics & Dashboard        │
│ Weeks 13-14: WP20-21 Test Coverage & Quality               │
│ Weeks 15-16: WP22-23 Performance & Cost Optimization       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Budget Estimate

### Development Time
- 16 weeks × 20 hours/week = 320 engineering hours
- At blended rate: ~$80-120k total investment

### Infrastructure Costs
- GitHub Actions: ~$100/month (extra minutes)
- LLM API calls: ~$200/month (evaluation runs)
- Storage (S3/artifacts): ~$20/month
- Total: ~$320/month ongoing

### ROI Justification
- **Prevented Regressions**: Saved customer trust + support costs
- **Faster Debugging**: 50% reduction in time to root cause
- **Model Selection**: 20% improvement in quality through optimal routing
- **Fine-Tuning**: 30% reduction in training iterations through better data

---

## Open Questions

1. **Telemetry Integration**: How to safely collect real queries for test generation?
2. **Community Involvement**: Should test dataset be crowdsourced?
3. **Model Hosting**: Run local models vs API calls for cost?
4. **Dashboard Hosting**: Self-hosted vs GitHub Pages?
5. **Fine-Tuning Platform**: Local LoRA vs cloud fine-tuning service?

---

## Next Steps

1. **Week 0**: Review and approve this milestone plan
2. **Week 1**: Spike on multi-backend CI matrix feasibility
3. **Week 2**: Create WP09 detailed implementation spec
4. **Week 3**: Begin implementation of Phase 1

