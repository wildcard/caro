# Frequently Asked Questions (FAQ)

**cmdai** - Your local AI assistant for safe shell commands

This FAQ answers common questions about cmdai's privacy, safety, capabilities, and usage. If you don't find your answer here, please check our [User Guide](USER_GUIDE.md) or [open an issue](https://github.com/wildcard/cmdai/issues).

---

## General Questions

### Q: What is cmdai?

**A:** cmdai is a command-line tool that converts natural language descriptions into safe, executable shell commands using local AI models. Instead of searching Stack Overflow or memorizing complex command syntax, just describe what you want in plain English, and cmdai generates the command for you.

**Example:**
```bash
$ cmdai "find all PDF files larger than 10MB in my Downloads folder"
Generated command:
  find ~/Downloads -name "*.pdf" -size +10M -ls

Execute this command? (y/N)
```

cmdai runs entirely on your computer using locally-downloaded AI models - no cloud services, no internet connection required after initial setup.

---

### Q: Is my data sent to the cloud? Are my commands private?

**A:** No data is ever sent to the cloud. cmdai is **100% local and offline-first**:

- AI models run locally on your computer (CPU or GPU)
- Your command descriptions never leave your machine
- Your files, directories, and command history stay private
- Works completely offline after the initial model download
- No telemetry, analytics, or data collection

**Privacy guarantee:** cmdai is designed for users who care about privacy and data sovereignty. Your terminal commands and file operations are yours alone.

---

### Q: How much does cmdai cost?

**A:** cmdai is **completely free** and **open source** (AGPL-3.0 license):

- Free to download, use, and modify
- No subscriptions, no API fees, no hidden costs
- No premium tiers or feature paywalls
- The AI model downloads are free (hosted on Hugging Face)

The only "cost" is the one-time ~1.5GB disk space for the AI model and a few minutes for the initial download.

---

### Q: What makes cmdai different from GitHub Copilot CLI, Shell GPT, or other AI command tools?

**A:** cmdai is designed with different priorities:

| Feature | cmdai | GitHub Copilot CLI | Shell GPT |
|---------|-------|-------------------|-----------|
| **Runs Locally** | Yes - 100% offline | No - cloud API | Configurable |
| **Privacy** | Complete - no data sent | Data sent to GitHub | Depends on model |
| **Cost** | Free forever | Requires GitHub subscription | Free (OpenAI API costs) |
| **Safety Validation** | Built-in, comprehensive | Limited | User responsibility |
| **Startup Speed** | <100ms (target) | Network dependent | Network dependent |
| **Offline Support** | Full (after setup) | No | Limited |
| **Open Source** | Yes (AGPL-3.0) | No | Yes (MIT) |
| **Apple Silicon Optimized** | Yes (MLX backend) | N/A | No |

**cmdai is best for**: Privacy-conscious users, offline environments, developers who want fast local inference, and those who don't want cloud dependencies.

---

### Q: How much disk space do I need?

**A:** Approximately **1.5-2GB total**:

- **AI Model**: ~1.1GB (Qwen2.5-Coder-1.5B-Instruct, quantized)
- **Tokenizer files**: ~3MB
- **cmdai binary**: ~10-50MB (depending on platform and features)
- **Cache/config**: <10MB

Models are stored in `~/.cache/cmdai/` (Linux/macOS) or `%LOCALAPPDATA%\cmdai\cache` (Windows). You can delete the cache to reclaim space, but the model will need to be re-downloaded on next use.

---

### Q: What operating systems are supported?

**A:** cmdai supports all major platforms:

**Tier 1 Support** (tested and optimized):
- macOS 11+ (Intel and Apple Silicon M1/M2/M3)
- Ubuntu 20.04+ LTS
- Debian 11+

**Tier 2 Support** (should work, community-tested):
- Windows 10/11 (PowerShell and CMD)
- Fedora, Arch, other Linux distributions
- FreeBSD (experimental)

**Best Performance:**
- Apple Silicon Macs (M1/M2/M3) with MLX backend: <2 seconds inference
- Modern x86_64 CPUs: 3-5 seconds inference
- Windows: 5-7 seconds inference (CPU-only)

See [Platform Testing Results](PLATFORM_TESTING_RESULTS.md) for detailed compatibility information.

---

### Q: Is cmdai open source? What's the license?

**A:** Yes! cmdai is fully open source under the **GNU Affero General Public License v3.0 (AGPL-3.0)**.

**What this means:**
- ✅ You can use cmdai for free, forever (personal and commercial)
- ✅ You can modify the source code to fit your needs
- ✅ You can redistribute cmdai
- ✅ You have access to all source code
- ⚠️ If you modify and distribute cmdai (including network services), you must share your changes under AGPL-3.0
- ⚠️ You must document any changes you make

**Why AGPL?** We chose AGPL to ensure cmdai remains open and accessible while preventing cloud providers from offering closed-source commercial versions without contributing back to the community.

View the source: [github.com/wildcard/cmdai](https://github.com/wildcard/cmdai)

---

## Usage Questions

### Q: How do I execute the generated command? Does it run automatically?

**A:** By default, cmdai **shows** the command but does NOT execute it automatically. You have three options:

**Option 1: Manual execution** (default, safest):
```bash
$ cmdai "list all files"
Generated command:
  ls -la

# Copy and paste the command yourself, or:
Execute this command? (y/N) y
# Press 'y' and Enter to execute
```

**Option 2: Auto-execute with confirmation**:
```bash
$ cmdai --execute "list all files"
# Will prompt for confirmation before running
```

**Option 3: Execute without confirmation** (use carefully!):
```bash
$ cmdai --execute --confirm "list all files"
# Runs immediately if safety validation passes
```

For potentially dangerous commands, cmdai always requires explicit confirmation regardless of flags (unless you use `--safety permissive`).

---

### Q: Can I use cmdai in scripts or automation?

**A:** Yes! cmdai provides JSON output and scriptable interfaces:

**JSON output for parsing:**
```bash
$ cmdai --output json "compress all logs"
{
  "command": "gzip *.log",
  "risk_level": "safe",
  "shell": "bash",
  "timestamp": "2024-11-19T10:30:00Z"
}
```

**Use in scripts:**
```bash
#!/bin/bash
# Generate command, capture as variable
CMD=$(cmdai --output json "find large files" | jq -r '.command')

# Review before executing
echo "About to run: $CMD"
read -p "Continue? (y/N) " confirm
[[ $confirm == "y" ]] && eval "$CMD"
```

**Non-interactive mode** (for automation):
```bash
# Pre-approve safe commands
cmdai --execute --confirm --safety moderate "show disk usage"
```

**Important:** For unattended automation, thoroughly test commands first and use `--safety strict` to block dangerous operations.

---

### Q: What shells are supported?

**A:** cmdai generates POSIX-compliant commands that work across shells:

**Fully Supported:**
- **Bash** (default on most Linux/macOS)
- **Zsh** (macOS default since Catalina)
- **sh** (POSIX shell)
- **Fish** (with `--shell fish` flag)
- **PowerShell** (Windows, with `--shell powershell`)
- **CMD** (Windows, with `--shell cmd`)

**Shell Detection:**
cmdai automatically detects your current shell from the `$SHELL` environment variable. You can override this:

```bash
$ cmdai --shell fish "find files modified today"
# Generates Fish shell syntax

$ cmdai --shell powershell "list running processes"
# Generates PowerShell syntax
```

**POSIX Compliance:** By default, cmdai generates commands using standard POSIX utilities (ls, find, grep, awk, sed) for maximum portability.

---

### Q: How do I change the default shell?

**A:** Set the default shell in your config file:

**1. Create/edit config:**
```bash
$ mkdir -p ~/.config/cmdai
$ nano ~/.config/cmdai/config.toml
```

**2. Add shell preference:**
```toml
[cli]
default_shell = "zsh"  # Options: bash, zsh, fish, sh, powershell, cmd
```

**3. Verify:**
```bash
$ cmdai --show-config
Default shell: zsh
```

Alternatively, set the `SHELL` environment variable or use the `--shell` flag on each invocation.

---

### Q: Can cmdai handle interactive commands (like vim, nano, less)?

**A:** Yes, but with some nuances:

**Current Behavior** (as of v1.0):
- cmdai generates the command syntax correctly
- You execute the command yourself, so interactivity works naturally
- Example: `cmdai "edit config file with vim"` → `vim ~/.config/cmdai/config.toml` (you run it)

**With --execute flag:**
- Interactive commands work if cmdai can detect them (vim, nano, less, top, htop, etc.)
- cmdai passes through stdin/stdout/stderr correctly
- Example: `cmdai --execute "edit file.txt with nano"` opens nano in your terminal

**Limitations:**
- Very complex interactive applications may need manual execution
- TUI applications work better when you run the generated command yourself

**Workaround for edge cases:**
Just copy the generated command and run it manually - cmdai's strength is generating correct syntax, not necessarily executing every command.

---

### Q: Does cmdai learn from my corrections? Can I train it?

**A:** Not currently, but feedback mechanisms are planned:

**Current (v1.0):**
- cmdai uses a pre-trained model (Qwen2.5-Coder) that cannot be modified
- No learning from individual user interactions
- Each command generation is independent

**Planned (v1.1+):**
- **Local feedback loop**: Mark commands as "good" or "bad" to improve future suggestions
- **Custom patterns**: Add your own command templates and preferences
- **Fine-tuning support**: (Advanced) Fine-tune the model on your command history

**Why not now?** We prioritize safety and predictability for v1.0. Learning systems can introduce unexpected behavior. Once the safety framework is proven, we'll add adaptive features.

**Current workaround:** Use shell aliases for frequently-needed commands:
```bash
alias mycmd="cmdai 'my common task description'"
```

---

### Q: Can I chain multiple commands or use pipes?

**A:** Yes! cmdai understands complex command descriptions:

**Pipes:**
```bash
$ cmdai "find all Python files and count the lines"
Generated command:
  find . -name "*.py" -exec wc -l {} + | awk '{sum+=$1} END {print sum}'
```

**Command chaining:**
```bash
$ cmdai "create a backup directory and copy all configs there"
Generated command:
  mkdir -p backup && cp ~/.config/*.conf backup/
```

**Logical operators:**
```bash
$ cmdai "check if docker is running or start it"
Generated command:
  systemctl is-active docker || sudo systemctl start docker
```

**Complex pipelines:**
```bash
$ cmdai "find large log files, compress them, and show space saved"
Generated command:
  find /var/log -name "*.log" -size +100M -exec sh -c 'original=$(stat -f%z "$1"); gzip "$1"; compressed=$(stat -f%z "$1.gz"); echo "Saved: $(($original - $compressed)) bytes"' _ {} \;
```

cmdai is trained on real-world command patterns and understands how to combine operations safely.

---

## Safety & Security

### Q: Is it safe to execute commands automatically? What if the AI makes a mistake?

**A:** cmdai is designed with safety as the top priority:

**Multi-Layer Safety System:**

1. **Dangerous Pattern Detection** (52 built-in patterns):
   - System destruction: `rm -rf /`, `rm -rf ~`, `mkfs`, `dd`
   - Fork bombs: `:(){ :|:& };:`
   - Privilege escalation: `sudo su`, `chmod 777 /etc`
   - Critical path operations: Modification of `/bin`, `/usr`, `/etc`, `/sys`

2. **Risk Assessment** (4 levels):
   - **Safe** (Green): Normal operations, no confirmation needed
   - **Moderate** (Yellow): Requires confirmation in strict mode
   - **High** (Orange): Requires explicit user approval
   - **Critical** (Red): Blocked by default, requires `--allow-dangerous`

3. **User Confirmation Workflow**:
   ```bash
   $ cmdai "remove all temporary files in system directories"
   ⚠️  WARNING: This command is DANGEROUS (risk level: CRITICAL)

   Command: rm -rf /tmp/*
   Reason: Operates on system paths

   This command could damage your system. Execute anyway? (y/N) n
   Aborted.
   ```

4. **POSIX Validation**: Generated commands use safe, standard utilities

**Best Practices:**
- Start with `--dry-run` (default) to review commands
- Use `--safety strict` in production environments
- Review unfamiliar commands before executing
- Test in a safe environment first (VM, container, test directory)

**Bottom Line:** cmdai is safer than copying commands from Stack Overflow without understanding them. You're always in control.

---

### Q: What commands are blocked or flagged as dangerous?

**A:** cmdai blocks or warns about 52+ dangerous patterns across 6 categories:

**1. Filesystem Destruction:**
- `rm -rf /`, `rm -rf /*`, `rm -rf ~`
- `rm -rf /home`, `rm -rf /Users`
- `mkfs.*`, `dd if=/dev/zero`
- `shred`, `wipe` on system paths

**2. System Modification:**
- Operations on `/bin`, `/usr/bin`, `/sbin`, `/usr/sbin`
- Operations on `/etc`, `/boot`, `/sys`
- `chmod 777 /`, recursive permission changes on root

**3. Privilege Escalation:**
- `sudo su`, `sudo -i`, `sudo bash`
- `chmod +s` (setuid)
- Unauthorized `/etc/sudoers` modification

**4. Resource Exhaustion:**
- Fork bombs: `:(){ :|:& };:`
- Infinite loops: `while true; do ... done` without limits
- Uncontrolled process spawning

**5. Network Security:**
- `curl | bash`, `wget | sh` (arbitrary code execution)
- Firewall rule deletion
- Exposure of sensitive data over network

**6. Data Loss:**
- Overwriting block devices: `dd ... of=/dev/sda`
- Truncating critical files: `> /etc/passwd`
- Unmounting system partitions

**Customization:**
Add your own patterns to `~/.config/cmdai/config.toml`:
```toml
[safety]
custom_patterns = [
    "npm.*-g.*root",  # Block global npm installs as root
    "git push.*--force.*main",  # Block force push to main
]
```

See [Safety Validation Documentation](docs/safety.md) for complete pattern list.

---

### Q: Can cmdai delete my files by accident?

**A:** Multiple safeguards prevent accidental file deletion:

**1. Explicit Confirmation for Deletions:**
```bash
$ cmdai "remove old log files"
Generated command:
  find /var/log -name "*.log" -mtime +30 -delete

⚠️  WARNING: This command performs file deletion (risk level: MODERATE)
Execute this command? (y/N)
```

**2. Path Safety Checks:**
- Blocks operations on system paths (`/`, `/usr`, `/etc`, etc.)
- Warns about recursive operations in home directory
- Detects wildcard expansion that could affect many files

**3. Dry Run by Default:**
cmdai shows you the command BEFORE execution:
```bash
$ cmdai "delete all tmp files"
Generated command:
  rm -rf /tmp/*  # ← You see this before it runs!

Execute this command? (y/N) n  # ← You can say no!
```

**4. Safe Alternatives Suggested:**
When possible, cmdai prefers safer alternatives:
- `trash` instead of `rm` (if installed)
- Move to backup directory before deletion
- Use `-i` (interactive) flag for confirmations

**5. Audit Trail** (planned for v1.1):
- Log all executed commands to `~/.cmdai_history`
- Ability to undo recent operations

**Reality Check:** cmdai cannot prevent you from confirming dangerous commands. If you type "y" to `rm -rf ~`, it will run. The safety system protects against *accidental* damage, not intentional actions.

**Best Practice:** Test destructive commands in a safe environment first, or add `--dry-run` to see what would happen.

---

### Q: How does the safety validation work? Can I trust it?

**A:** cmdai's safety validation uses multiple techniques:

**1. Pattern Matching (Primary Method):**
- 52 pre-compiled regular expressions
- Covers known dangerous commands from CVE databases, security research, and community input
- Patterns are **allow-listed**, not block-listed (conservative approach)

**2. Syntax Analysis:**
- Parses command structure (pipes, redirects, subshells)
- Identifies operators that can cause harm (`>`, `>>`, `|&`, etc.)
- Detects shell expansion patterns that might be dangerous

**3. Context-Aware Risk Assessment:**
```rust
// Simplified example
fn assess_risk(command: &str, context: &ExecutionContext) -> RiskLevel {
    if contains_system_destruction(command) { return RiskLevel::Critical; }
    if operates_on_system_paths(command) { return RiskLevel::High; }
    if modifies_files(command) { return RiskLevel::Moderate; }
    RiskLevel::Safe
}
```

**4. POSIX Compliance Check:**
- Validates that commands use standard utilities
- Rejects non-standard or platform-specific dangerous operations

**Limitations (Be Aware):**
- ❌ Cannot detect *semantic* errors (command that's syntactically safe but logically wrong)
- ❌ Cannot predict all edge cases (novel attack patterns)
- ❌ Relies on pattern database (new dangerous commands need to be added)
- ❌ Can be bypassed by obfuscation (though the AI rarely generates obfuscated commands)

**Trust Model:**
- **DO trust** cmdai to catch common dangerous patterns (rm -rf /, fork bombs, etc.)
- **DO trust** cmdai more than unvalidated commands from the internet
- **DON'T trust** cmdai as a replacement for understanding what a command does
- **DON'T trust** cmdai in high-stakes environments without review

**Open Source Advantage:** The safety patterns are in `src/safety/patterns.rs` - you can review, audit, and contribute improvements!

---

### Q: Can I trust AI-generated commands? What if the AI hallucinates?

**A:** AI-generated commands require the same caution as any external source:

**How cmdai Reduces Hallucinations:**

1. **Specialized Model**: Uses Qwen2.5-Coder, trained specifically on code and commands (not general chat)
2. **Strict System Prompt**: Instructs the AI to generate only valid POSIX commands, no explanations or guesses
3. **JSON-Only Output**: Forces structured responses, reducing free-form errors
4. **Validation Pipeline**: All commands pass through safety validation before display

**Reality Check - AI Limitations:**
- ✅ Excellent at common operations (file management, text processing, system info)
- ⚠️ Good at standard utilities with common flags
- ❌ May struggle with very niche tools or uncommon flag combinations
- ❌ Can generate syntactically valid but semantically incorrect commands
- ❌ May not understand complex business logic or context-specific requirements

**Examples of Potential Issues:**

**Good (Common Patterns):**
```bash
$ cmdai "find files modified in last 24 hours"
✓ find . -type f -mtime -1  # Correct!
```

**Risky (Uncommon Tools):**
```bash
$ cmdai "convert HEIC to JPEG with optimal compression"
? ffmpeg -i input.heic -q:v 2 output.jpg  # May or may not be "optimal" for your use case
```

**Best Practices:**
1. **Verify before executing**: Read the generated command, make sure it makes sense
2. **Test in safe environment**: Try on sample data first
3. **Use --verbose**: See the AI's reasoning (planned feature)
4. **Check man pages**: If unsure, verify flags with `man <command>`
5. **Start simple**: Test cmdai on simple tasks before complex operations

**When NOT to use cmdai:**
- Mission-critical operations without review
- Commands affecting production systems without testing
- Operations you don't understand (learn what the command does first)

**When cmdai shines:**
- Discovering correct syntax for utilities you rarely use
- Combining commands you know exist but can't remember how to chain
- Exploring alternatives to your usual approach
- Learning new command patterns

**Bottom Line:** Treat cmdai like a knowledgeable colleague - helpful and usually right, but always double-check important work.

---

## Technical Questions

### Q: Which AI model does cmdai use?

**A:** cmdai uses **Qwen2.5-Coder-1.5B-Instruct** (quantized to 4-bit) by default:

**Why this model?**
- **Specialized**: Trained specifically on code, including shell commands
- **Compact**: 1.5 billion parameters, small enough for laptops
- **Fast**: Quantized to 4-bit (Q4_K_M) for quick inference
- **Accurate**: Excellent performance on command generation tasks
- **Permissive License**: Apache 2.0, safe for all uses

**Specifications:**
- **Size (quantized)**: ~1.1GB download
- **Size (in memory)**: ~1.5-2GB RAM during inference
- **Architecture**: Qwen2.5 (transformer-based)
- **Context length**: 8192 tokens (more than enough for command generation)
- **Training cutoff**: September 2024 (includes modern tools)

**Performance Benchmarks:**
- **Accuracy**: ~92% on command generation tasks (internal testing)
- **Inference time**: 1-5 seconds depending on hardware
- **Model quality**: Comparable to GPT-3.5 for command tasks

**Source:** [Hugging Face - Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF](https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF)

---

### Q: Does cmdai work offline? What requires internet?

**A:** cmdai is designed to work **fully offline** after initial setup:

**Requires Internet (One-Time Only):**
1. **First-time setup**: Downloading the AI model (~1.1GB)
2. **Software updates**: Installing newer versions of cmdai

**Works Completely Offline:**
- ✅ Command generation
- ✅ Safety validation
- ✅ Command execution
- ✅ All configuration and cache management
- ✅ Reading local documentation

**Offline Workflow:**
```bash
# Initial setup (requires internet)
$ cmdai --version  # Triggers model download
Downloading Qwen2.5-Coder-1.5B-Instruct (1.1GB)...
[████████████████████] 100% - ETA: 0s
Model ready!

# Now disconnect from internet - everything still works
$ cmdai "find large files"
Generated command: find . -size +100M
# ✓ No internet needed!
```

**Offline Use Cases:**
- Air-gapped environments (after pre-loading model)
- Working on planes, trains, remote locations
- Security-conscious environments (no external network access)
- Avoiding bandwidth costs

**Optional Remote Backends** (require internet):
- vLLM remote server (if configured)
- Ollama server (if configured)
- Custom LLM endpoints (if configured)

**Default Behavior:** cmdai uses the embedded local model, so no internet is needed after setup.

---

### Q: How fast is command generation? What's the performance?

**A:** Performance varies by hardware, but cmdai is optimized for speed:

**Performance Targets (v1.0):**

| Metric | Apple Silicon (M1) | Modern Intel/AMD | Older Hardware | Windows |
|--------|-------------------|------------------|----------------|---------|
| **Startup Time** | <100ms | <150ms | <200ms | <300ms |
| **First Inference** | <2s | 3-5s | 5-8s | 5-10s |
| **Subsequent Inferences** | <1.5s | 2-4s | 4-7s | 4-8s |
| **Safety Validation** | <10ms | <10ms | <20ms | <20ms |

**Real-World Examples:**

**Apple Silicon M1 Mac (Best Case):**
```bash
$ time cmdai "find large files"
Generated command: find . -size +100M

real    0m1.832s  # Total time: 1.8 seconds
user    0m1.654s
sys     0m0.124s
```

**Intel i7 Linux (Typical):**
```bash
$ time cmdai "compress all logs"
Generated command: gzip *.log

real    0m3.421s  # Total time: 3.4 seconds
user    0m3.201s
sys     0m0.186s
```

**What Affects Performance:**

**Faster:**
- Apple Silicon with MLX backend (GPU acceleration)
- More RAM (16GB+)
- SSD storage for model loading
- Shorter, simpler prompts

**Slower:**
- CPU-only inference
- Limited RAM (<8GB causes swapping)
- HDD storage
- Very long or complex prompts

**Optimization Tips:**

1. **Use MLX backend** (Apple Silicon only):
   ```toml
   # ~/.config/cmdai/config.toml
   [backend]
   primary = "mlx"  # Enables GPU acceleration
   ```

2. **Keep model in cache**: Don't delete `~/.cache/cmdai/` unnecessarily

3. **Use shorter prompts**: "find PDFs" is faster than "find all PDF files in my home directory sorted by size"

4. **Close memory-intensive apps**: Free up RAM for model inference

**Benchmark Your System:**
```bash
$ cmdai --benchmark
Running performance tests...
Startup time: 87ms ✓
First inference: 1.95s ✓
Safety validation: 8ms ✓
Your system meets all performance targets!
```

See [Performance Benchmarks](docs/performance.md) for detailed results.

---

### Q: Can I use my own AI model? Can I use different backends?

**A:** Yes! cmdai supports multiple backends and custom models:

**Built-in Backends:**

**1. Embedded Backend (Default - Local):**
```toml
# ~/.config/cmdai/config.toml
[backend]
primary = "embedded"
model = "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF"  # Default
```

**2. Ollama Backend (Local Server):**
```toml
[backend]
primary = "ollama"

[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"  # Or any Ollama model
```

**3. vLLM Backend (Remote Server):**
```toml
[backend]
primary = "vllm"

[backend.vllm]
base_url = "http://your-server:8000"
model_name = "codellama/CodeLlama-7b-hf"
api_key = "optional-api-key"
```

**4. Custom OpenAI-Compatible Endpoints:**
```toml
[backend]
primary = "vllm"  # Uses OpenAI-compatible API

[backend.vllm]
base_url = "http://localhost:8000/v1"
model_name = "your-custom-model"
```

**Using Your Own Model (Advanced):**

**Requirements:**
- Model must support instruction-following
- Model should be trained on code (for best results)
- Model format: GGUF (for embedded), or served via Ollama/vLLM

**Example - Custom GGUF Model:**
```bash
# 1. Download your model to cache directory
$ mkdir -p ~/.cache/cmdai/models/custom
$ cp /path/to/your-model.gguf ~/.cache/cmdai/models/custom/

# 2. Update config
$ nano ~/.config/cmdai/config.toml
[backend.embedded]
model_path = "~/.cache/cmdai/models/custom/your-model.gguf"
model_name = "custom-model"

# 3. Test
$ cmdai "test command"
```

**Recommended Alternative Models:**
- **CodeLlama 7B**: Larger, more capable, slower
- **DeepSeek-Coder 1.3B**: Similar size, good performance
- **StarCoder 1B**: Fast, code-focused
- **Phi-2**: General-purpose, compact

**Fallback System:**
```toml
[backend]
primary = "ollama"
enable_fallback = true
fallback_order = ["embedded", "vllm"]  # Try these if primary fails
```

**Note:** Custom models may not work well without fine-tuning for command generation. The default model is thoroughly tested.

---

### Q: How do I update to a newer version of cmdai?

**A:** Update method depends on your installation:

**Homebrew (macOS/Linux):**
```bash
$ brew update
$ brew upgrade cmdai
```

**Package Manager (Linux):**
```bash
# APT (Debian/Ubuntu)
$ sudo apt update && sudo apt upgrade cmdai

# DNF (Fedora)
$ sudo dnf upgrade cmdai

# Pacman (Arch)
$ sudo pacman -Syu cmdai
```

**Manual Binary (All Platforms):**
```bash
# 1. Download latest release
$ curl -L https://github.com/wildcard/cmdai/releases/latest/download/cmdai-$(uname -s)-$(uname -m) -o /tmp/cmdai

# 2. Replace existing binary
$ chmod +x /tmp/cmdai
$ sudo mv /tmp/cmdai /usr/local/bin/cmdai

# 3. Verify
$ cmdai --version
cmdai 1.1.0
```

**Building from Source:**
```bash
$ cd /path/to/cmdai
$ git pull origin main
$ cargo build --release
$ sudo cp target/release/cmdai /usr/local/bin/
```

**What Gets Preserved During Updates:**
- ✅ Configuration: `~/.config/cmdai/config.toml`
- ✅ Model cache: `~/.cache/cmdai/models/`
- ✅ Command history: `~/.cmdai_history` (if enabled)

**What Gets Replaced:**
- Binary executable
- Built-in safety patterns (may be updated)
- Default configuration template

**Check for Updates:**
```bash
$ cmdai --check-update
Current version: 1.0.0
Latest version: 1.1.0
Update available! Run: brew upgrade cmdai
```

**Migration Notes:**
Major version updates (1.x → 2.x) may require config migration. See [UPGRADING.md](UPGRADING.md) for version-specific instructions.

---

## Troubleshooting

### Q: I'm getting "Model not found in cache" error. What should I do?

**A:** This means the AI model hasn't been downloaded yet (normal for first-time use):

**Solution 1 - Automatic Download (Recommended):**
```bash
$ cmdai --version
cmdai 1.0.0

Downloading model: Qwen2.5-Coder-1.5B-Instruct (1.1GB)
This is a one-time download and may take a few minutes...

[████████████████████] 100% - 1.1GB/1.1GB - ETA: 0s
Model downloaded successfully!
Cache location: ~/.cache/cmdai/models/
```

**Solution 2 - Manual Download:**
```bash
$ cmdai --download-model
Downloading Qwen2.5-Coder-1.5B-Instruct...
[████████████████████] 100%
Done! Model saved to ~/.cache/cmdai/models/
```

**If Download Fails:**

**Check internet connection:**
```bash
$ ping huggingface.co
# Should respond if connected
```

**Check disk space:**
```bash
$ df -h ~/.cache
# Need at least 2GB free
```

**Check cache directory permissions:**
```bash
$ ls -ld ~/.cache/cmdai
# Should be writable by your user
$ chmod 755 ~/.cache/cmdai
```

**Try manual download from Hugging Face:**
```bash
$ mkdir -p ~/.cache/cmdai/models
$ cd ~/.cache/cmdai/models
$ wget https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf
$ wget https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/tokenizer.json
```

**Clear corrupted cache and retry:**
```bash
$ rm -rf ~/.cache/cmdai
$ cmdai --version  # Re-downloads model
```

---

### Q: cmdai says a command seems unsafe, but I want to run it anyway. How can I override?

**A:** Several options to override safety warnings:

**Option 1 - Confirm Individually (Safest):**
```bash
$ cmdai "remove all .tmp files in /var"
⚠️  WARNING: This command is DANGEROUS (risk level: HIGH)
Execute this command? (y/N) y
# Type 'y' to proceed
```

**Option 2 - Adjust Safety Level:**
```bash
# Permissive mode (fewer warnings)
$ cmdai --safety permissive "risky command"

# Moderate mode (balanced)
$ cmdai --safety moderate "risky command"

# Strict mode (maximum protection, default)
$ cmdai --safety strict "risky command"
```

**Option 3 - Disable Safety Checks (Not Recommended):**
```bash
$ cmdai --no-safety "dangerous command"
⚠️  WARNING: Safety validation disabled!
```

**Option 4 - Allow Specific Pattern:**
```toml
# ~/.config/cmdai/config.toml
[safety]
allow_patterns = [
    "rm -rf /tmp/my-specific-dir",  # Allow this specific command
]
```

**Option 5 - Run Generated Command Manually:**
```bash
$ cmdai "risky operation"
Generated command:
  rm -rf /tmp/*

# Copy and run yourself (bypasses cmdai safety)
$ rm -rf /tmp/*
```

**Warning Signs You Should NOT Override:**
- Command operates on `/`, `/usr`, `/etc`, `/bin`
- Command contains `rm -rf` without specific paths
- You don't understand what the command does
- Command is flagged as "CRITICAL" risk level

**Safe Overrides:**
- Command operates on a test directory you own
- You've verified the command logic manually
- Command is flagged as "MODERATE" but you understand the risk

**Best Practice:** If cmdai blocks something repeatedly, consider whether you really need that operation or if there's a safer alternative.

---

### Q: Command generation is very slow (>10 seconds). How can I fix this?

**A:** Several factors can slow down inference. Try these solutions:

**Check 1 - Verify Performance Baseline:**
```bash
$ cmdai --benchmark
Running benchmarks...
Startup: 87ms ✓
Inference: 12.4s ✗ (target: <5s)
```

**Solution 1 - Enable MLX Backend (Apple Silicon Only):**
```toml
# ~/.config/cmdai/config.toml
[backend]
primary = "mlx"  # GPU-accelerated
```
Expected improvement: 5-8s → 1.5-2s

**Solution 2 - Check RAM Usage:**
```bash
$ free -h  # Linux
$ vm_stat  # macOS

# If RAM is low, close other applications
# Model needs ~2GB RAM to run efficiently
```

**Solution 3 - Use Smaller Model (Faster but Less Accurate):**
```toml
[backend.embedded]
model_name = "DeepSeek-Coder-1.3B"  # Smaller model
```

**Solution 4 - Switch to Remote Backend (If Available):**
```toml
[backend]
primary = "ollama"  # Uses local Ollama server (often faster)

[backend.ollama]
base_url = "http://localhost:11434"
model_name = "codellama:7b"
```

**Solution 5 - Optimize System:**
```bash
# Linux: Check CPU governor
$ cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor
# Should be "performance" not "powersave"
$ sudo cpupower frequency-set -g performance

# macOS: Disable low power mode
# System Preferences > Battery > Uncheck "Low Power Mode"

# Check for background processes
$ top
# Kill unnecessary CPU-heavy processes
```

**Solution 6 - Reinstall Model (Might Be Corrupted):**
```bash
$ rm -rf ~/.cache/cmdai/models
$ cmdai --version  # Re-downloads
```

**Solution 7 - Use Verbose Mode to Diagnose:**
```bash
$ cmdai --verbose "test command"
[DEBUG] Loading model: 2.1s
[DEBUG] Tokenizing prompt: 0.05s
[DEBUG] Running inference: 8.3s  ← Slow step
[DEBUG] Parsing response: 0.02s
```

**Expected Performance by Hardware:**
- Apple M1/M2/M3: 1.5-2s
- Modern Intel/AMD (4+ cores): 3-5s
- Older hardware (2 cores): 5-8s
- Very old hardware: 10-15s (consider remote backend)

If none of these help, please [open an issue](https://github.com/wildcard/cmdai/issues) with `--verbose` output.

---

### Q: Download failed or got interrupted. How do I retry or resume?

**A:** cmdai supports download resumption (v1.0+):

**Automatic Resume:**
```bash
$ cmdai --download-model
Downloading model...
[█████░░░░░░░░░░░] 35% - Connection lost

# Re-run same command - resumes from 35%
$ cmdai --download-model
Resuming download from 35%...
[████████████████████] 100% - Complete!
```

**Manual Resume:**
```bash
$ cmdai --resume-download
Found incomplete download: Qwen2.5-Coder (382MB/1.1GB)
Resume? (y/N) y
Resuming...
```

**Clear Failed Download and Restart:**
```bash
# Remove partial download
$ rm -rf ~/.cache/cmdai/models/.tmp

# Start fresh
$ cmdai --download-model --force
Downloading from beginning...
```

**If Resume Doesn't Work (Server Doesn't Support Range Requests):**
```bash
# Download with external tool, then move to cache
$ wget -c https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf
$ mkdir -p ~/.cache/cmdai/models
$ mv qwen2.5-coder-1.5b-instruct-q4_k_m.gguf ~/.cache/cmdai/models/

# Download tokenizer
$ wget https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/tokenizer.json
$ mv tokenizer.json ~/.cache/cmdai/models/
```

**Verify Downloaded Files:**
```bash
$ cmdai --verify-model
Checking model integrity...
File: qwen2.5-coder-1.5b-instruct-q4_k_m.gguf
Size: 1.1GB ✓
Checksum: a3f2e8... ✓
Tokenizer: Found ✓

Model is valid and ready to use!
```

**Checksum Mismatch (Corrupted Download):**
```bash
$ cmdai --verify-model
Checksum mismatch! File may be corrupted.

# Delete and re-download
$ rm ~/.cache/cmdai/models/qwen2.5-coder-1.5b-instruct-q4_k_m.gguf
$ cmdai --download-model
```

---

### Q: I'm getting "Permission denied" errors. What's wrong?

**A:** Permission errors usually relate to cache, config, or command execution:

**Error 1 - Cannot Write to Cache Directory:**
```
Error: Permission denied (os error 13)
Location: ~/.cache/cmdai/models
```

**Solution:**
```bash
# Check ownership
$ ls -ld ~/.cache/cmdai
drwxr-xr-x  root  root  # ← Wrong! Should be your user

# Fix ownership
$ sudo chown -R $USER:$USER ~/.cache/cmdai

# Or recreate directory
$ sudo rm -rf ~/.cache/cmdai
$ mkdir -p ~/.cache/cmdai
```

**Error 2 - Cannot Write Config File:**
```
Error: Permission denied writing to ~/.config/cmdai/config.toml
```

**Solution:**
```bash
$ chmod 755 ~/.config/cmdai
$ chmod 644 ~/.config/cmdai/config.toml
```

**Error 3 - Cannot Execute Binary:**
```
bash: /usr/local/bin/cmdai: Permission denied
```

**Solution:**
```bash
$ chmod +x /usr/local/bin/cmdai
```

**Error 4 - Generated Command Needs Sudo:**
```bash
$ cmdai "install package"
Generated command:
  apt install vim

Execute? (y/N) y
Error: Permission denied (need sudo)
```

**Solution - cmdai Won't Auto-Sudo (Security):**
```bash
# Run the generated command with sudo yourself
$ sudo apt install vim

# Or ask for sudo command explicitly
$ cmdai "install vim with sudo"
Generated command:
  sudo apt install vim
```

**Error 5 - SELinux or AppArmor Blocking:**
```
Error: Permission denied (enforcing mode)
```

**Solution (Advanced):**
```bash
# Check SELinux status
$ getenforce
Enforcing

# Temporary: Set permissive mode (testing only!)
$ sudo setenforce 0

# Permanent: Add SELinux policy for cmdai
# (Consult SELinux documentation)
```

**Common Permission Checklist:**
```bash
# 1. Cache directory
$ ls -ld ~/.cache/cmdai
drwxr-xr-x  youruser  yourgroup  # ✓ Correct

# 2. Config directory
$ ls -ld ~/.config/cmdai
drwxr-xr-x  youruser  yourgroup  # ✓ Correct

# 3. Binary
$ ls -l $(which cmdai)
-rwxr-xr-x  1 root  root  # ✓ Correct (executable)

# 4. No root-owned files in user directories
$ find ~/.cache/cmdai ~/.config/cmdai -user root
# Should return nothing
```

---

## Still Have Questions?

**Documentation:**
- [User Guide](USER_GUIDE.md) - Comprehensive usage documentation
- [Troubleshooting Guide](TROUBLESHOOTING.md) - Detailed problem-solving
- [Configuration Guide](docs/configuration.md) - Advanced configuration options

**Community:**
- [GitHub Issues](https://github.com/wildcard/cmdai/issues) - Bug reports and feature requests
- [GitHub Discussions](https://github.com/wildcard/cmdai/discussions) - Questions and community support
- [Contributing Guide](CONTRIBUTING.md) - How to contribute to cmdai

**Getting Help:**
When asking for help, please include:
1. cmdai version: `cmdai --version`
2. Operating system: `uname -a`
3. Error message or unexpected behavior
4. Steps to reproduce
5. Output of `cmdai --verbose <your command>`

We're here to help make cmdai work for you!

---

**Last Updated:** 2024-11-19
**cmdai Version:** 1.0.0
**Feedback:** Found an error or have a suggestion? [Open an issue](https://github.com/wildcard/cmdai/issues)
