# cmdai Harbor Agent

Custom Harbor agent wrapper for testing cmdai against Terminal-Bench 2.0.

## Overview

This directory contains the Harbor agent implementation that allows cmdai to be evaluated on Terminal-Bench 2.0 benchmark tasks.

## Architecture

### Single-shot vs Multi-turn

**cmdai** is designed as a single-shot command generator:
- Input: Natural language description
- Output: One shell command
- Execution: External (user runs the command)

**Terminal-Bench** requires a multi-turn agent:
- Input: Task instruction
- Process: Iteratively generate and execute commands
- Output: Complete task solution

### Bridge Implementation

`CmdaiAgent` (in `cmdai/cmdai_agent.py`) bridges this gap:

```python
for turn in range(max_turns):
    # 1. Generate command using cmdai
    command = cmdai.generate(prompt)

    # 2. Execute in terminal
    result = execute(command)

    # 3. Update context with results
    prompt = update_prompt(instruction, result)

    # 4. Check if task complete
    if task_complete(result):
        break
```

## Files

- `cmdai/__init__.py` - Package initialization
- `cmdai/cmdai_agent.py` - Main agent implementation
- `README.md` - This file

## Agent Features

### Setup Phase
- Uploads cmdai binary to test container
- Verifies installation and version
- Prepares execution environment

### Execution Loop
- **Turn-based generation**: Calls cmdai for each command
- **JSON parsing**: Extracts commands from cmdai output
- **Error recovery**: Handles failed commands gracefully
- **Context preservation**: Maintains task history
- **Completion detection**: Heuristics for task success

### Logging
Each turn creates a directory with:
- `prompt.txt` - Input to cmdai
- `generated_command.txt` - Command from cmdai
- `cmdai_stdout.txt` - cmdai output
- `exec_stdout.txt` - Command execution output
- Return codes and error messages

## Usage

### Local Testing (with Docker)

```bash
harbor run \
  --dataset terminal-bench@2.0 \
  --agent-import-path ./harbor_agents/cmdai/cmdai_agent.py:CmdaiAgent \
  --model anthropic/claude-sonnet-4-5 \
  --n-concurrent 4
```

### GitHub Actions

The workflow is pre-configured:
1. Go to Actions tab
2. Select "Terminal-Bench 2.0 Evaluation"
3. Click "Run workflow"
4. Configure parameters and run

## Configuration

### Agent Parameters

Can be passed via `--ak` (agent kwarg):

```bash
--ak max_turns=100      # Maximum command iterations
--ak timeout_sec=60     # Execution timeout per command
```

### cmdai Binary Location

Default: `/usr/local/bin/cmdai` in container

Modify in `cmdai_agent.py` if needed:
```python
self._cmdai_binary = "/custom/path/cmdai"
```

## Limitations

### Current Constraints

1. **No streaming**: cmdai generates one command at a time
2. **Limited planning**: Each command is independent
3. **Context window**: Long task histories may exceed limits
4. **No backtracking**: Can't undo previous commands

### Potential Improvements

1. **Multi-step planning**: Generate command sequences
2. **Error analysis**: Learn from failed patterns
3. **State tracking**: Maintain task completion checklist
4. **Adaptive prompting**: Adjust context based on progress

## Performance Expectations

Based on Terminal-Bench 2.0 leaderboard:

| Success Rate | Status |
|--------------|--------|
| 77%+ | 🎯 Target (ambitious) |
| 50-77% | 🥇 Competitive with SOTA |
| 30-50% | 🥈 Good baseline |
| < 30% | 🔧 Needs optimization |

Current leader: 49.6% (OpenAI Codex CLI)

## Development Workflow

### Testing Locally

1. Build cmdai: `cargo build --release`
2. Test on sample task:
   ```bash
   harbor run \
     --dataset hello-world@head \
     --agent-import-path ./harbor_agents/cmdai/cmdai_agent.py:CmdaiAgent \
     --model anthropic/claude-sonnet-4-5
   ```
3. Review logs in `jobs/` directory

### Iterating on Agent Logic

1. Modify `cmdai_agent.py`
2. Test with simple task
3. Review turn-by-turn logs
4. Adjust prompt construction
5. Re-test until satisfied

### Full Benchmark Run

1. Test with hello-world first
2. Run single full benchmark
3. Analyze failure patterns
4. Optimize cmdai/agent
5. Run 5 times for leaderboard

## Troubleshooting

### Agent Not Found

**Error**: `Cannot import CmdaiAgent`

**Solution**: Check import path is correct:
```bash
--agent-import-path /full/path/to/harbor_agents/cmdai/cmdai_agent.py:CmdaiAgent
```

### Binary Not Found

**Error**: `cmdai binary not found at /home/user/cmdai/target/release/cmdai`

**Solution**:
1. Build cmdai: `cargo build --release`
2. Verify path in `cmdai_agent.py` matches actual location

### Import Errors

**Error**: `ModuleNotFoundError: No module named 'harbor'`

**Solution**: Harbor must be installed in the environment running the test. In GitHub Actions this is handled automatically.

### Low Success Rate

**Issue**: Tasks failing frequently

**Debug steps**:
1. Check turn logs for patterns
2. Test cmdai locally on failed tasks
3. Review generated commands
4. Adjust safety validation if too strict
5. Improve prompt templates

## Contributing

To improve the agent:

1. Identify failure patterns in benchmark results
2. Modify `cmdai_agent.py` logic
3. Test changes locally
4. Submit PR with performance comparison
5. Update this README with learnings

## Resources

- **Terminal-Bench**: https://www.tbench.ai/
- **Harbor docs**: https://harborframework.com/docs
- **cmdai repo**: https://github.com/wildcard/cmdai
- **Testing guide**: ../docs/github-actions-testing.md
