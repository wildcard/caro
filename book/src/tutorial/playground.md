# Try It Online

> 🚧 **Coming Soon:** Interactive cmdai playground powered by WebAssembly

Welcome to the cmdai interactive playground! This page will allow you to try cmdai directly in your browser without installing anything.

---

## 🎯 What's the Playground?

The cmdai playground will be a browser-based environment where you can:

- ✨ **Try cmdai instantly** - No installation required
- 🔒 **Safe experimentation** - Sandboxed environment
- 📚 **Learn interactively** - Step-by-step tutorials with live examples
- 🎨 **See results in real-time** - Immediate feedback
- 💾 **Share examples** - URL-based code sharing

---

## 🚀 Quick Demo (Concept)

Here's what the playground will look like:

```
┌─────────────────────────────────────────────────────────────┐
│ 🎮 cmdai Playground                         [Share] [Reset] │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Enter your natural language command:                        │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ list all files in the current directory                 │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                           [Generate Command] │
│                                                               │
│  Generated Command:                                          │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ ls -la                                                   │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  Safety Check: ✅ Safe                                       │
│  Risk Level: Safe - No dangerous operations detected        │
│                                                               │
│  Output (Simulated):                                         │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │ total 24                                                 │ │
│  │ drwxr-xr-x  5 user  staff   160 Nov 19 10:00 .          │ │
│  │ drwxr-xr-x  8 user  staff   256 Nov 18 09:00 ..         │ │
│  │ -rw-r--r--  1 user  staff  1234 Nov 19 08:30 README.md  │ │
│  │ drwxr-xr-x  3 user  staff    96 Nov 18 12:00 src        │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 📖 Interactive Tutorials

The playground will include guided tutorials with live editing:

### Tutorial 1: Your First Command

```
Step 1/5: Basic Command Generation

Try this prompt: "show all files"

[Try it] [Next Step] [Skip Tutorial]
```

### Tutorial 2: Working with Files

```
Step 1/3: Finding Files

Challenge: Generate a command to find all PDF files

Your prompt: [________________]

[Hint] [Check Answer] [Next]
```

### Tutorial 3: Safety in Action

```
Step 1/4: Understanding Safety

Try a dangerous command: "delete everything"

See how cmdai protects you!

[Try it] [Learn More] [Next]
```

---

## 🛠️ Interactive Examples

Click any example to load it in the playground:

### Example: File Operations

<div class="playground-example">
<strong>Find large files</strong>
<pre><code>find files larger than 100MB</code></pre>
<button>[Try This Example]</button>
</div>

<div class="playground-example">
<strong>Count files by type</strong>
<pre><code>count how many files of each type</code></pre>
<button>[Try This Example]</button>
</div>

<div class="playground-example">
<strong>Find recent changes</strong>
<pre><code>show files modified in last 24 hours</code></pre>
<button>[Try This Example]</button>
</div>

### Example: System Monitoring

<div class="playground-example">
<strong>Check disk space</strong>
<pre><code>show disk usage in human readable format</code></pre>
<button>[Try This Example]</button>
</div>

<div class="playground-example">
<strong>Find memory hogs</strong>
<pre><code>show processes using most memory</code></pre>
<button>[Try This Example]</button>
</div>

### Example: Safety Validation

<div class="playground-example">
<strong>Safe deletion</strong>
<pre><code>delete temporary files older than 30 days</code></pre>
<em>See safety warnings in action!</em>
<button>[Try This Example]</button>
</div>

<div class="playground-example">
<strong>Dangerous command (blocked)</strong>
<pre><code>delete all files</code></pre>
<em>Watch cmdai protect you!</em>
<button>[Try This Example]</button>
</div>

---

## 🎓 Learning Modes

### Beginner Mode
- **Guided prompts** with suggestions
- **Detailed explanations** of each command
- **Safety tips** highlighted
- **Step-by-step** tutorials

### Advanced Mode
- **Free-form** command generation
- **Multiple backends** to try
- **Configuration options** to experiment
- **Performance metrics** displayed

### Challenge Mode
- **Puzzles** to solve using cmdai
- **Timed challenges** for speed
- **Leaderboard** for community
- **Achievement badges** for milestones

---

## 🔧 Technical Implementation

### Architecture

The playground will use **WebAssembly (WASM)** to run cmdai directly in your browser:

```
┌──────────────┐
│   Browser    │
├──────────────┤
│  React UI    │
│      ↓       │
│  WASM cmdai  │  ← Rust compiled to WebAssembly
│      ↓       │
│  LLM Backend │  ← Embedded model or API
└──────────────┘
```

### Key Technologies

1. **Rust + WASM** - cmdai compiled to WebAssembly
2. **wasm-bindgen** - JavaScript/Rust interop
3. **Web Workers** - Non-blocking inference
4. **IndexedDB** - Model caching
5. **React/TypeScript** - Interactive UI

### Model Options

**Option A: Embedded Model (Recommended)**
- Small quantized model (~100MB)
- Runs entirely in browser
- No server calls needed
- Privacy-preserving

**Option B: API Backend**
- Connect to OpenAI/Anthropic
- Faster, more powerful
- Requires API key
- Network dependent

**Option C: Hybrid**
- Start with API for speed
- Download embedded model in background
- Switch to local when ready

---

## 🎨 Feature Preview

### Shareable Links

Create a link to share your example:

```
https://cmdai.github.io/playground?example=find-large-files
```

### URL Parameters

```
?prompt=show%20all%20files          # Pre-filled prompt
&backend=embedded                   # Backend selection
&safety=strict                      # Safety level
&tutorial=first-command             # Load tutorial
```

### Export Options

```
[Download as Shell Script] [Copy Command] [Share Link]
```

---

## 🗺️ Roadmap

### Phase 1: Basic Playground (MVP)
- [x] Design specification
- [ ] WASM compilation setup
- [ ] Basic UI with prompt input
- [ ] Command generation display
- [ ] Safety validation display

### Phase 2: Interactive Features
- [ ] Simulated command execution
- [ ] Real-time syntax highlighting
- [ ] Example library
- [ ] Share functionality

### Phase 3: Learning Features
- [ ] Step-by-step tutorials
- [ ] Interactive challenges
- [ ] Progress tracking
- [ ] Achievement system

### Phase 4: Advanced Features
- [ ] Multiple backend support
- [ ] Configuration playground
- [ ] Performance metrics
- [ ] Community examples

---

## 💡 Try It Now (Manual)

While the playground is in development, try cmdai locally:

### Quick Start

```bash
# Install cmdai
git clone https://github.com/wildcard/cmdai.git
cd cmdai
cargo build --release

# Try the examples from this page!
./target/release/cmdai "show all files"
./target/release/cmdai "find files larger than 100MB"
./target/release/cmdai "show disk usage"
```

### Example Session

Follow along with our tutorials:

1. **[Your First Command](./first-command.md)** - 5 minute intro
2. **[Working with Files](./working-with-files.md)** - 15 minute deep dive
3. **[System Operations](./system-operations.md)** - 15 minute guide

---

## 🤝 Help Build the Playground!

The playground is an open-source effort. We'd love your help!

### Ways to Contribute

1. **Design** - UI/UX for the playground
2. **Development** - WASM implementation
3. **Content** - Tutorial examples
4. **Testing** - Try early versions

### Get Involved

- **GitHub Issue**: [Playground Implementation #TODO]
- **Discord**: [Join our community]
- **Discussions**: [Share ideas]

---

## 📚 Related Resources

- **[Getting Started](../user-guide/getting-started.md)** - Install cmdai locally
- **[Quick Start](../user-guide/quick-start.md)** - Common patterns
- **[Architecture](../dev-guide/architecture.md)** - How cmdai works
- **[Contributing](../community/contributing.md)** - Join the project

---

## 🎉 Coming Soon!

We're actively working on the playground. Follow our progress:

- **GitHub Milestones**: Track development
- **Twitter/X**: [@cmdai_dev] - Updates
- **Blog**: Weekly progress reports

**Star the repo** to get notified when the playground launches! ⭐

---

<div class="info">
<strong>📢 Want early access?</strong>
<p>Sign up for the playground beta program to be among the first to try it!</p>
<p><a href="https://github.com/wildcard/cmdai/discussions/playground-beta">[Join Beta Program]</a></p>
</div>

---

## 💬 Feedback

Have ideas for the playground? We want to hear them!

- What features would you love to see?
- What tutorials would help you learn?
- What examples would be most useful?

**Share your thoughts**: [GitHub Discussions - Playground Ideas]

---

**Meanwhile, dive into our comprehensive tutorials:**

- ← [Tutorial: System Operations](./system-operations.md)
- → [User Guide](../user-guide/getting-started.md)

