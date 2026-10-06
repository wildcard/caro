//! Contract: every agent in `.claude/agents/` declares its tools.
//!
//! An agent with no `tools:` line inherits every tool in the session,
//! including connected MCP servers (mail, chat, deploys). Two agents that
//! called themselves read-only could write files that way. Least privilege per
//! role is google/ax's "minimally privileged policies"; see
//! `docs/research/2026-09-24-google-ax-lessons.md`.

use std::fs;
use std::path::Path;

/// Agents allowed to inherit every tool, with the reason.
const INHERITS_ALL: &[(&str, &str)] = &[(
    "claude-design-frontend-engineer",
    "drives browser MCP servers (Claude_in_Chrome, Playwright, Claude_Preview) \
     whose tool names depend on what is installed",
)];

/// Built-in tool names an agent may list. A typo here silently drops a tool.
const KNOWN_TOOLS: &[&str] = &[
    "Read",
    "Write",
    "Edit",
    "Bash",
    "Grep",
    "Glob",
    "WebFetch",
    "WebSearch",
    "NotebookEdit",
    "TodoWrite",
];

/// Tools that change files or run commands.
const WRITE_TOOLS: &[&str] = &["Write", "Edit", "NotebookEdit", "Bash"];

struct Agent {
    name: String,
    description: String,
    tools: Option<Vec<String>>,
}

fn frontmatter_field<'a>(frontmatter: &'a str, key: &str) -> Option<&'a str> {
    frontmatter
        .lines()
        .find_map(|line| line.strip_prefix(key)?.strip_prefix(':'))
        .map(str::trim)
}

fn agents() -> Vec<Agent> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(".claude/agents");
    let mut agents: Vec<Agent> = fs::read_dir(&dir)
        .expect("read .claude/agents")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .map(|path| {
            let text = fs::read_to_string(&path).expect("read agent file");
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            let frontmatter = text
                .strip_prefix("---\n")
                .and_then(|rest| rest.split_once("\n---\n"))
                .map(|(fm, _)| fm)
                .unwrap_or_else(|| panic!("{name}: missing frontmatter"));
            Agent {
                description: frontmatter_field(frontmatter, "description")
                    .unwrap_or_default()
                    .to_string(),
                tools: frontmatter_field(frontmatter, "tools")
                    .map(|list| list.split(',').map(|t| t.trim().to_string()).collect()),
                name,
            }
        })
        .collect();
    agents.sort_by(|a, b| a.name.cmp(&b.name));
    assert!(!agents.is_empty(), "no agents found in {}", dir.display());
    agents
}

fn inherits_all(name: &str) -> bool {
    INHERITS_ALL.iter().any(|(agent, _)| *agent == name)
}

#[test]
fn every_agent_declares_its_tools() {
    let missing: Vec<_> = agents()
        .into_iter()
        .filter(|a| a.tools.is_none() && !inherits_all(&a.name))
        .map(|a| a.name)
        .collect();
    assert!(
        missing.is_empty(),
        "agents without a `tools:` line inherit every tool, MCP servers included: {missing:?}. \
         Add a `tools:` line, or add the agent to INHERITS_ALL with the reason."
    );
}

#[test]
fn declared_tools_are_known() {
    for agent in agents() {
        for tool in agent.tools.iter().flatten() {
            assert!(
                KNOWN_TOOLS.contains(&tool.as_str()),
                "{}: unknown tool {tool:?} (typo? known: {KNOWN_TOOLS:?})",
                agent.name
            );
        }
    }
}

#[test]
fn read_only_agents_cannot_write() {
    for agent in agents() {
        if !agent.description.to_lowercase().contains("read-only") {
            continue;
        }
        let tools = agent
            .tools
            .as_ref()
            .unwrap_or_else(|| panic!("{}: read-only agent must list its tools", agent.name));
        for tool in tools {
            assert!(
                !WRITE_TOOLS.contains(&tool.as_str()),
                "{}: describes itself as read-only but has {tool}",
                agent.name
            );
        }
    }
}

#[test]
fn inherit_exemptions_are_not_stale() {
    let agents = agents();
    for (name, _reason) in INHERITS_ALL {
        let agent = agents
            .iter()
            .find(|a| a.name == *name)
            .unwrap_or_else(|| panic!("INHERITS_ALL names missing agent {name}"));
        assert!(
            agent.tools.is_none(),
            "{name} now declares tools; remove it from INHERITS_ALL"
        );
    }
}
