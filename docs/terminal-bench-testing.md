# Terminal-Bench 2.0 Testing Guide for cmdai

## Overview

Terminal-Bench 2.0 is a benchmark with 89 meticulously crafted tasks for evaluating AI coding agents in terminal environments. This document outlines how to test cmdai against this benchmark.

## Current Leaderboard Status

As of December 2024:
- **Top performer**: OpenAI Codex CLI (GPT-5) with **49.6%** success rate
- **Target for cmdai**: 77% accuracy (significantly higher than current leaders!)
- **Benchmark difficulty**: Only ~50% solved by best agents
- **Tasks**: 89 verified tasks with 300+ hours of validation each

## Prerequisites

### Required Infrastructure

1. **Docker** (for local testing) OR cloud provider API keys:
   - Daytona
   - E2B
   - Modal
   - Runloop

2. **Harbor Framework** (installed ✅)
```bash
uv tool install harbor
```

3. **cmdai binary** (built ✅)
```bash
cargo build --release
# Binary at: ./target/release/cmdai
```

4. **API Keys** (for LLM backends):
   - ANTHROPIC_API_KEY (for Claude models)
   - OPENAI_API_KEY (for OpenAI models)
   - Or configure cmdai to use local models

## Testing Approaches

### Option 1: Docker-based Local Testing (Recommended)

**Requirements**: Docker installed and running

```bash
# Set API key
export ANTHROPIC_API_KEY="your-key-here"

# Run Terminal-Bench 2.0 with custom cmdai agent
harbor run \
  --dataset terminal-bench@2.0 \
  --agent-import-path /home/user/cmdai/harbor_agents/cmdai/cmdai_agent.py:CmdaiAgent \
  --model anthropic/claude-sonnet-4-5 \
  --n-concurrent 4 \
  --env docker
```

### Option 2: Cloud-based Testing

**Requirements**: Cloud provider API key (e.g., Daytona)

```bash
# Set API keys
export ANTHROPIC_API_KEY="your-anthropic-key"
export DAYTONA_API_KEY="your-daytona-key"

# Run on cloud with high concurrency
harbor run \
  --dataset terminal-bench@2.0 \
  --agent-import-path /home/user/cmdai/harbor_agents/cmdai/cmdai_agent.py:CmdaiAgent \
  --model anthropic/claude-sonnet-4-5 \
  --n-concurrent 100 \
  --env daytona
```

### Option 3: Terminus-2 Comparison Baseline

Test with the standard terminus-2 agent first to establish a baseline:

```bash
export ANTHROPIC_API_KEY="your-key-here"

# Run terminus-2 agent
harbor run \
  --dataset terminal-bench@2.0 \
  --agent terminus-2 \
  --model anthropic/claude-sonnet-4-5 \
  --n-concurrent 4
```

## Custom cmdai Agent Implementation

Located at: `./harbor_agents/cmdai/cmdai_agent.py`

### Agent Architecture

The cmdai agent wrapper implements a multi-turn loop:

1. **Turn N**: Generate command using cmdai
   ```bash
   cmdai --output json "task instruction"
   ```

2. **Execute**: Run the generated command in the terminal

3. **Observe**: Capture stdout, stderr, and return code

4. **Iterate**: Pass results back to cmdai for next command

5. **Complete**: Continue until task completion or max turns reached

### Key Features

- **Single-shot command generation**: cmdai generates one command at a time
- **Multi-turn orchestration**: Agent wrapper handles iteration logic
- **Error recovery**: Failed commands prompt cmdai to generate fixes
- **Context preservation**: Previous commands and outputs inform next generation

## Scoring and Evaluation

### Success Metrics

Terminal-Bench evaluates tasks based on:
- **Functional correctness**: Does the solution work?
- **Test verification**: Do all test cases pass?
- **Time efficiency**: Completed within timeout limits

### Leaderboard Submission

For official leaderboard submission:

1. Run **5 complete benchmark runs**
2. Submit results via:
   ```bash
   # Results are saved in jobs/ directory
   # Email job directories to Terminal-Bench team
   ```

3. Include metadata:
   - Agent name: cmdai
   - Version: 0.1.0
   - Model backend configuration
   - Success rate across 89 tasks

## Current Implementation Status

### ✅ Completed

- Harbor framework installation
- cmdai binary build (release mode)
- Custom Harbor agent wrapper created
- Integration architecture designed
- Documentation and testing guide

### ⏸️ Blocked

- **Local testing**: Requires Docker (not available in current environment)
- **Cloud testing**: Requires cloud provider API keys

### 📋 Next Steps

1. **Set up Docker** OR obtain cloud provider credentials
2. **Configure API keys** for LLM backends
3. **Run initial test** with hello-world dataset
4. **Execute full benchmark** on terminal-bench@2.0
5. **Analyze results** and iterate on cmdai improvements
6. **Submit to leaderboard** after achieving target accuracy

## Optimization Strategies for 77% Target

To reach 77% accuracy (vs. current 49.6% leader):

### 1. Enhanced Command Generation

- **Better prompts**: Include more context about task requirements
- **Multi-step planning**: Break complex tasks into subtasks
- **Error analysis**: Learn from failed command patterns

### 2. Safety Validation Integration

- **Pre-execution checks**: Use cmdai's safety validator
- **Command refinement**: Iterate on dangerous commands
- **Risk assessment**: Avoid critical failures

### 3. Context Window Optimization

- **Relevant history**: Keep last N commands and outputs
- **Terminal state summary**: Compress long outputs
- **Task progress tracking**: Maintain completion checklist

### 4. Fallback Strategies

- **Alternative approaches**: Try different command strategies
- **Tool discovery**: Check available commands before use
- **Error recovery**: Implement robust retry logic

## Comparison: cmdai vs. Terminus-2

| Aspect | cmdai | Terminus-2 |
|--------|-------|------------|
| **Architecture** | Single-shot generator | Multi-turn agent |
| **Command generation** | One command per call | Batch commands |
| **Planning** | External orchestration | Built-in reasoning |
| **Safety** | Comprehensive validator | Basic checks |
| **Model backend** | Local + fallback | API-based |
| **Startup time** | < 100ms | API latency |
| **Cost** | Local inference | API costs |

## Resources

- **Terminal-Bench leaderboard**: https://www.tbench.ai/leaderboard/terminal-bench/2.0
- **Harbor documentation**: https://harborframework.com/docs
- **cmdai repository**: https://github.com/wildcard/cmdai

## Contact and Support

For Terminal-Bench submission and questions:
- Email results and job directories to Terminal-Bench team
- Check leaderboard for latest standings
- Review task definitions in Harbor registry

---

**Status**: Infrastructure setup complete, awaiting Docker/cloud resources for testing
**Next milestone**: First benchmark run on terminal-bench@2.0
**Target**: 77% accuracy on 89 tasks
