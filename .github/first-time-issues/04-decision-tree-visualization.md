# 🌳 Add Decision Tree Visualization for Command Generation

**Labels**: `good-first-issue`, `first-time-contributor`, `agent`, `visualization`, `decision-tree`, `ux`
**Difficulty**: Medium ⭐⭐⭐
**Skills**: Rust, tree data structures, terminal graphics, algorithmic thinking
**Perfect for**: Decision tree enthusiasts, data structure lovers, visual thinkers, agent builders

## The Vision

cmdai uses a **static decision tree** to map natural language to terminal commands. Right now, this process is invisible to users. Let's create a visualization that shows:

1. How the natural language was parsed
2. What decision nodes were traversed
3. Why a particular command was generated
4. Alternative paths that were considered

Think of it as "explain mode" for AI decision-making!

## What You'll Build

A new flag `cmdai --show-tree "your prompt"` that visualizes the decision process:

```
cmdai --show-tree "list all pdf files"

Decision Tree Visualization:
============================

Intent Recognition:
└─ "list" detected → LIST_FILES operation
   └─ "all" detected → RECURSIVE_SEARCH
      └─ "pdf" detected → FILE_EXTENSION_FILTER
         └─ "files" detected → CONFIRM_FILE_OPERATION

Command Construction:
└─ Tool: find
   ├─ Path: current directory (.)
   ├─ Filter: -name "*.pdf"
   └─ Action: -type f

Generated Command: find . -type f -name "*.pdf"

Alternative Paths Considered:
├─ ls + grep (rejected: not recursive)
└─ locate (rejected: requires updatedb)

Safety Check: ✓ SAFE
```

## Implementation Guide

### Step 1: Design the Decision Tree Structure

Create `src/agent/decision_tree.rs`:

```rust
use serde::{Deserialize, Serialize};

/// Represents a node in the command generation decision tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionNode {
    /// Intent recognition node
    Intent {
        detected: String,
        confidence: f32,
        operation: Operation,
        next: Box<DecisionNode>,
    },
    /// Parameter extraction node
    Parameter {
        name: String,
        value: String,
        next: Option<Box<DecisionNode>>,
    },
    /// Command construction node
    CommandBuilder {
        tool: String,
        arguments: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    ListFiles,
    SearchText,
    ProcessFiles,
    SystemInfo,
    NetworkOperation,
}

/// Visualizes a decision tree in terminal-friendly format
pub struct TreeVisualizer {
    tree: DecisionNode,
}

impl TreeVisualizer {
    pub fn new(tree: DecisionNode) -> Self {
        Self { tree }
    }

    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str("Decision Tree Visualization:\n");
        output.push_str("============================\n\n");
        self.render_node(&self.tree, &mut output, "", true);
        output
    }

    fn render_node(&self, node: &DecisionNode, output: &mut String, prefix: &str, is_last: bool) {
        // Implement tree rendering with box-drawing characters
        let connector = if is_last { "└─" } else { "├─" };

        match node {
            DecisionNode::Intent { detected, operation, next, .. } => {
                output.push_str(&format!("{}{} \"{}\" detected → {:?}\n",
                    prefix, connector, detected, operation));

                let new_prefix = format!("{}   ", prefix);
                self.render_node(next, output, &new_prefix, true);
            }
            // Handle other node types...
            _ => {}
        }
    }
}
```

### Step 2: Capture Decision Trail

Update the command generation to track decisions:

```rust
pub struct DecisionTrail {
    nodes: Vec<DecisionNode>,
    alternatives_considered: Vec<AlternativePath>,
}

pub struct AlternativePath {
    tool: String,
    reason_rejected: String,
}

impl DecisionTrail {
    pub fn record_intent(&mut self, detected: &str, operation: Operation) {
        // Record each decision as it's made
    }

    pub fn record_alternative(&mut self, tool: &str, reason: &str) {
        // Track alternatives that were considered but rejected
    }
}
```

### Step 3: Add CLI Flag

In `src/cli/mod.rs`:

```rust
#[derive(Parser, Debug)]
pub struct Cli {
    // ... existing fields

    /// Show decision tree visualization
    #[arg(long)]
    pub show_tree: bool,
}
```

### Step 4: Render the Tree

When `--show-tree` is active:

```rust
if cli.show_tree {
    let decision_trail = generate_command_with_trail(&prompt)?;
    let visualizer = TreeVisualizer::new(decision_trail.root_node);
    println!("{}", visualizer.render());
}
```

### Step 5: Add Tests

```rust
#[test]
fn test_tree_visualization_renders() {
    let tree = DecisionNode::Intent {
        detected: "list".to_string(),
        confidence: 0.95,
        operation: Operation::ListFiles,
        next: Box::new(DecisionNode::Parameter {
            name: "extension".to_string(),
            value: "pdf".to_string(),
            next: None,
        }),
    };

    let visualizer = TreeVisualizer::new(tree);
    let output = visualizer.render();

    assert!(output.contains("list"));
    assert!(output.contains("pdf"));
    assert!(output.contains("└─") || output.contains("├─"));
}
```

## Acceptance Criteria

- [ ] `--show-tree` flag displays decision tree visualization
- [ ] Tree shows intent recognition steps
- [ ] Tree shows parameter extraction
- [ ] Tree shows command construction logic
- [ ] Alternative paths are listed with rejection reasons
- [ ] Output is terminal-friendly (ASCII art, box-drawing chars)
- [ ] Works for at least 5 different prompt types
- [ ] Safety validation is shown in the tree
- [ ] Tests cover tree construction and rendering
- [ ] Code passes `cargo fmt` and `cargo clippy`

## Visualization Examples

### Example 1: File Listing
```
Intent Recognition:
└─ "list" → LIST_FILES
   └─ "pdf" → FILTER(extension=pdf)
      └─ "larger than 10MB" → FILTER(size>10M)

Tool Selection:
└─ find (confidence: 0.95)
   ├─ ls + grep (rejected: size filter unavailable)
   └─ locate (rejected: requires index)

Safety: ✓ SAFE
```

### Example 2: Text Search
```
Intent Recognition:
└─ "search" → SEARCH_TEXT
   └─ "TODO" → PATTERN(literal="TODO")
      └─ "in code files" → FILTER(extension=[.js,.py,.rs])

Tool Selection:
└─ grep (confidence: 0.90)
   └─ ack (considered: not standard)

Safety: ✓ SAFE
```

## Why This Matters

1. **Transparency**: Users understand AI decision-making
2. **Education**: Learn how NLP maps to commands
3. **Trust**: See why cmdai chose this command
4. **Debugging**: Helps us improve the decision tree
5. **Differentiation**: No other tool shows this!

## Technical Challenges

1. **Decision Capture**: Retrofit existing generation logic
2. **Tree Rendering**: Clean ASCII art in terminals
3. **Performance**: Don't slow down normal usage
4. **Complexity**: Keep visualization simple but informative

## Resources

- [Tree Visualization in Rust](https://docs.rs/ptree/latest/ptree/)
- [Box-Drawing Characters](https://en.wikipedia.org/wiki/Box-drawing_character)
- [Decision Trees](https://en.wikipedia.org/wiki/Decision_tree)
- Our `src/agent/mod.rs` for existing command generation logic

## Optional Enhancements

- **Color coding**: Different colors for different node types
- **Confidence scores**: Show probability for each decision
- **Export to JSON**: `--show-tree --format json`
- **Web view**: Generate HTML visualization
- **Interactive mode**: Click to explore decision paths

## Questions?

We'll help you with:
- Understanding the current command generation flow
- Designing the tree data structure
- Rendering ASCII trees in terminals
- Capturing decision trails without major refactoring

**Ready to make AI decision-making transparent? Let's build the most educational agent system ever! 🌳**
