# Tutorial: Your First Command

Welcome! This tutorial will guide you step-by-step through generating your first command with cmdai.

## What You'll Learn

In this tutorial, you'll learn to:
- ✅ Generate a simple command from natural language
- ✅ Understand the safety review process
- ✅ Execute generated commands
- ✅ Handle errors gracefully

**Time to complete:** ~5 minutes

---

## Step 1: Installation

First, build cmdai from source:

```bash
# Clone the repository
git clone https://github.com/wildcard/cmdai.git
cd cmdai

# Build the release binary
cargo build --release
```

**Expected output:**
```
   Compiling cmdai v0.1.0 (/path/to/cmdai)
    Finished release [optimized] target(s) in 2m 34s
```

<div class="info">
<strong>💡 Tip:</strong> The first build takes a few minutes. Subsequent builds are much faster!
</div>

---

## Step 2: Verify Installation

Check that cmdai is working:

```bash
./target/release/cmdai --version
```

**Expected output:**
```
cmdai 0.1.0
```

**What's happening?** You're running the compiled cmdai binary and asking it to show its version.

---

## Step 3: Generate Your First Command

Let's generate a simple command to list files:

```bash
./target/release/cmdai "show me all files in the current directory"
```

**What you'll see:**

```
🤖 Generating command...
✨ Generated command:
  ls -la

Safety Check: ✅ Safe
  • No dangerous operations detected
  • Risk level: Safe

Execute this command? (y/N)
```

**What's happening?**
1. cmdai sends your request to the LLM backend
2. The backend generates a shell command
3. cmdai validates the command for safety
4. You're asked to confirm before execution

---

## Step 4: Review the Command

Before typing `y`, let's understand what `ls -la` does:

- `ls` - Lists files and directories
- `-l` - Uses long listing format (shows permissions, owner, size, date)
- `-a` - Shows all files, including hidden ones (starting with `.`)

**Is it safe?** Yes! This command only reads information, it doesn't modify anything.

---

## Step 5: Execute the Command

Type `y` and press Enter:

```
Execute this command? (y/N) y

Executing...
total 64
drwxr-xr-x  12 user  staff    384 Nov 19 08:00 .
drwxr-xr-x   8 user  staff    256 Nov 18 10:15 ..
-rw-r--r--   1 user  staff   1234 Nov 19 07:30 README.md
drwxr-xr-x   5 user  staff    160 Nov 18 12:00 src
-rw-r--r--   1 user  staff    567 Nov 19 08:00 Cargo.toml
...

✅ Command executed successfully
```

**Congratulations!** 🎉 You've just generated and executed your first command with cmdai!

---

## Step 6: Try Different Prompts

Now try these variations to see how cmdai adapts:

### Example A: Different Phrasing

```bash
./target/release/cmdai "list everything here"
```

**Generated:**
```bash
ls -la
```

<div class="info">
<strong>💡 Insight:</strong> cmdai understands natural language! Different phrasings produce similar commands.
</div>

### Example B: More Specific Request

```bash
./target/release/cmdai "show only PDF files in this directory"
```

**Generated:**
```bash
find . -maxdepth 1 -name "*.pdf" -type f
```

**What changed?**
- More specific request → more specific command
- Uses `find` instead of `ls` for pattern matching
- `-maxdepth 1` limits search to current directory only

### Example C: Human-Readable Output

```bash
./target/release/cmdai "show disk usage in human readable format"
```

**Generated:**
```bash
df -h
```

**What's new?**
- `df` shows disk filesystem usage
- `-h` flag provides human-readable sizes (e.g., "10G" instead of "10485760")

---

## Step 7: Understanding Safety

Let's see what happens with a potentially dangerous command:

```bash
./target/release/cmdai "delete all files"
```

**What you'll see:**

```
🤖 Generating command...

🛑 SAFETY WARNING: Critical Risk Detected!

Generated command:
  rm -rf *

Risk Level: 🔴 CRITICAL
Reason: Recursive deletion detected

This command will:
  • Delete ALL files in the current directory
  • Cannot be undone
  • May cause data loss

❌ Command blocked by safety validator in strict mode.

To allow this command:
  1. Use --safety permissive flag (use with extreme caution)
  2. Manually type the command yourself
```

**What's happening?**
- cmdai detected a dangerous pattern
- The safety validator blocked execution
- You're protected from accidental data loss!

<div class="warning">
<strong>⚠️ Important:</strong> cmdai's safety features are your first line of defense, but always review commands before executing!
</div>

---

## What You Learned

Congratulations! You now know how to:

✅ Generate commands from natural language
✅ Review generated commands for safety
✅ Execute commands with confirmation
✅ Understand safety warnings
✅ Use different phrasings for similar results

---

## Next Steps

Now that you've mastered the basics, try these tutorials:

- **[Working with Files](./working-with-files.md)** - File operations and management
- **[System Operations](./system-operations.md)** - System information and monitoring
- **[Try It Online](./playground.md)** - Interactive examples in your browser

Or explore:
- [Quick Start Guide](../user-guide/quick-start.md) - More examples and patterns
- [Safety & Security](../user-guide/safety.md) - Deep dive into safety features
- [Configuration](../user-guide/configuration.md) - Customize cmdai for your workflow

---

## Troubleshooting

### Issue: "command not found"

**Solution:** Use the full path to the binary:
```bash
./target/release/cmdai "your command"
```

Or add to PATH:
```bash
export PATH="$PWD/target/release:$PATH"
cmdai "your command"
```

### Issue: Backend unavailable

**Solution:** cmdai will automatically use the embedded backend. Check logs:
```bash
RUST_LOG=debug ./target/release/cmdai "your command"
```

### Issue: Build fails

**Solution:** Ensure you have Rust 1.75+:
```bash
rustc --version  # Should be 1.75.0 or higher
rustup update   # Update if needed
```

---

## Practice Challenge

Try to generate commands for these tasks without looking at the answer:

1. "show the 5 largest files in the current directory"
2. "count how many files are here"
3. "show the current date and time"

<details>
<summary>Click to see expected commands</summary>

1. `du -sh * | sort -rh | head -5` or `ls -lhS | head -6`
2. `ls -1 | wc -l` or `find . -maxdepth 1 -type f | wc -l`
3. `date`

Your commands might be slightly different - that's okay! Multiple solutions can work.

</details>

---

**Ready for more?** Continue to [Tutorial: Working with Files](./working-with-files.md) →
