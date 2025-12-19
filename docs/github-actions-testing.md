# Running Terminal-Bench 2.0 on GitHub Actions

This guide explains how to run Terminal-Bench 2.0 evaluation using GitHub Actions, which provides Docker infrastructure automatically.

## Quick Start

1. **Set up API keys** (one-time setup)
2. **Trigger the workflow** from GitHub UI
3. **Download results** from the workflow run

## Step 1: Configure API Keys

### Add GitHub Secrets

Go to your repository settings: `Settings > Secrets and variables > Actions`

Add the following secrets:

#### For Anthropic models (Claude)
```
Name: ANTHROPIC_API_KEY
Value: sk-ant-xxxxx...
```

#### For OpenAI models (GPT-4)
```
Name: OPENAI_API_KEY
Value: sk-xxxxx...
```

⚠️ **Important**: At least one API key is required to run the benchmark.

## Step 2: Trigger the Workflow

### Via GitHub Web UI

1. Go to the **Actions** tab in your repository
2. Select **"Terminal-Bench 2.0 Evaluation"** from the workflows list
3. Click **"Run workflow"** button
4. Configure the run:

| Parameter | Description | Default | Options |
|-----------|-------------|---------|---------|
| **Model** | LLM to use | `anthropic/claude-sonnet-4-5` | Claude Sonnet 4.5, Claude Sonnet 4, Claude Opus 4, GPT-4 |
| **Concurrent trials** | Parallel tests | `4` | `1`-`100` (higher = faster but more $) |
| **Dataset** | Benchmark to run | `terminal-bench@2.0` | `terminal-bench@2.0`, `hello-world@head` |
| **Number of runs** | Complete iterations | `1` | `1`-`5` (leaderboard needs 5) |

5. Click **"Run workflow"**

### Recommended Configurations

#### Quick Test (Development)
```yaml
Model: anthropic/claude-sonnet-4-5
Concurrent trials: 4
Dataset: hello-world@head
Number of runs: 1
```
**Time**: ~5 minutes | **Cost**: ~$0.50

#### Full Evaluation (Single Run)
```yaml
Model: anthropic/claude-sonnet-4-5
Concurrent trials: 8
Dataset: terminal-bench@2.0
Number of runs: 1
```
**Time**: ~2-3 hours | **Cost**: ~$50-100

#### Leaderboard Submission (5 Runs)
```yaml
Model: anthropic/claude-sonnet-4-5
Concurrent trials: 8
Dataset: terminal-bench@2.0
Number of runs: 5
```
**Time**: ~10-15 hours | **Cost**: ~$250-500

## Step 3: Monitor Progress

### Real-time Monitoring

1. Click on the running workflow in the Actions tab
2. Click on the "Run Terminal-Bench 2.0" job
3. Expand the steps to see live logs:
   - **Build cmdai**: Compilation progress
   - **Run Terminal-Bench evaluation**: Task execution
   - **Aggregate results**: Summary statistics

### Progress Indicators

- ✅ Tasks completed successfully
- ❌ Tasks failed
- ⏸️ Tasks in progress

## Step 4: Review Results

### Download Results Artifact

1. Wait for the workflow to complete
2. Scroll to the bottom of the workflow run page
3. Find the **Artifacts** section
4. Download: `terminal-bench-results-[dataset]-[model]-[run-number].zip`

### Results Structure

```
results/
├── SUMMARY.md                    # Overview and statistics
├── run-1/                        # First benchmark run
│   ├── result.json              # Aggregated metrics
│   └── trials/                  # Individual task results
│       ├── task-1/
│       │   ├── trajectory.json  # Agent actions
│       │   ├── stdout.txt       # Command outputs
│       │   └── verification.txt # Test results
│       └── task-2/
│           └── ...
├── run-2/                        # Second run (if requested)
└── ...
```

### Key Metrics in SUMMARY.md

- **Success rate**: Percentage of tasks completed successfully
- **Total trials**: Number of tasks attempted
- **Errors**: Number of failures
- **Per-task breakdown**: Individual task results

## Step 5: Analyze Results

### Understanding the Output

#### Success Rate
- **77%+**: Excellent! Exceeds current SOTA (49.6%)
- **50-77%**: Competitive with best agents
- **30-50%**: Good baseline, needs optimization
- **< 30%**: Needs significant improvements

#### Common Failure Patterns

Check `results/run-X/trials/*/verification.txt` for:

1. **Command generation errors**: cmdai produced invalid commands
2. **Execution failures**: Commands ran but produced wrong results
3. **Timeout errors**: Tasks took too long to complete
4. **Safety validation blocks**: Commands deemed too dangerous

### Optimization Workflow

Based on results:

1. **Identify failure patterns** in task logs
2. **Improve cmdai prompts** for common scenarios
3. **Adjust safety rules** if too restrictive
4. **Re-run benchmark** with improvements
5. **Compare results** across runs

## Leaderboard Submission

### Requirements

- ✅ 5 complete benchmark runs
- ✅ All runs on same model/configuration
- ✅ Results artifacts downloaded
- ✅ Agent metadata documented

### Submission Process

1. **Download all 5 run artifacts** from GitHub Actions
2. **Extract the results** directories
3. **Email to Terminal-Bench team**:
   - Attach job directories (or provide download link)
   - Include agent info:
     ```
     Agent: cmdai
     Version: 0.1.0
     Model: anthropic/claude-sonnet-4-5
     Date: 2024-12-05
     Average success rate: X%
     ```

4. **Wait for verification** and leaderboard update

### Submission Checklist

- [ ] 5 complete runs completed successfully
- [ ] Same model used for all runs
- [ ] Results downloaded and extracted
- [ ] Agent metadata prepared
- [ ] Email sent to Terminal-Bench team
- [ ] Follow up if no response within 1 week

## Cost Estimation

### API Usage Costs

**Per task** (rough estimates):
- Claude Sonnet 4.5: ~$0.50 - $2.00
- Claude Opus 4: ~$1.00 - $4.00
- GPT-4: ~$0.75 - $3.00

**Full benchmark** (89 tasks):
- Single run: $50-200
- 5 runs (leaderboard): $250-1000

### Reducing Costs

1. **Test with hello-world** first (free/cheap)
2. **Use cheaper models** for development (Sonnet vs Opus)
3. **Lower concurrency** (slower but same total cost)
4. **Fix obvious bugs** before full runs
5. **Sample subset** of tasks during development

## Troubleshooting

### Workflow Fails to Start

**Issue**: "Missing required secret"
**Solution**: Add API keys to GitHub Secrets (Step 1)

### Docker Errors

**Issue**: "Cannot connect to Docker daemon"
**Solution**: GitHub Actions has Docker pre-installed. This shouldn't happen. Try re-running the workflow.

### Timeout Errors

**Issue**: Workflow times out after 6 hours
**Solution**:
- Reduce `n_concurrent` (paradoxically can help with rate limits)
- Split into multiple runs with fewer tasks each
- Use faster model for initial testing

### Import Errors

**Issue**: "Cannot import CmdaiAgent"
**Solution**: Check that `harbor_agents/cmdai/cmdai_agent.py` exists in the repository

### Low Success Rate

**Issue**: Success rate < 30%
**Solution**:
1. Review task failure logs
2. Test cmdai locally on failed tasks
3. Improve command generation prompts
4. Adjust safety validation rules
5. Consider multi-turn planning enhancements

## Advanced Usage

### Testing Specific Tasks

Modify the workflow to test specific tasks:

```yaml
# In .github/workflows/terminal-bench.yml
# Add to harbor run command:
--task-name "task-name-pattern*"
```

### Custom Agent Parameters

Pass additional kwargs to CmdaiAgent:

```yaml
# Add to harbor run command:
--ak max_turns=100
--ak timeout_sec=300
```

### Using Different Branches

Test development branches:

1. Push changes to your branch
2. Workflow will use code from that branch automatically
3. No need to merge to main first

## Continuous Evaluation

### Automated Testing

Set up automated runs on:
- **Pull requests**: Test before merge
- **Schedule**: Nightly benchmark runs
- **Release tags**: Full 5-run evaluation

### Monitoring Improvements

Track success rate across commits:

1. Run benchmark on each major change
2. Record results in a spreadsheet
3. Plot improvement over time
4. Identify regressions quickly

## Resources

- **Workflow file**: `.github/workflows/terminal-bench.yml`
- **Agent implementation**: `harbor_agents/cmdai/cmdai_agent.py`
- **Results guide**: `docs/terminal-bench-testing.md`
- **Leaderboard**: https://www.tbench.ai/leaderboard/terminal-bench/2.0

---

**Next Steps**:
1. ⚙️ Configure API keys in GitHub Secrets
2. 🏃 Run your first evaluation
3. 📊 Review results and iterate
4. 🏆 Submit to leaderboard when ready!
