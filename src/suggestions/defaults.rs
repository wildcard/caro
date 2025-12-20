//! Default suggestions for new users and common workflows

use super::{QueryCategory, SuggestedQuery, SuggestionReason};
use super::environment::GitState;

/// Get beginner-friendly suggestions for new terminal users
pub fn get_beginner_suggestions() -> Vec<SuggestedQuery> {
    vec![
        // File Operations - The Basics
        SuggestedQuery::new(
            "list all files in this directory",
            "See what files and folders are here",
            SuggestionReason::NewUserOnboarding,
            QueryCategory::FileOperations,
        ).with_relevance(0.95),

        SuggestedQuery::new(
            "show hidden files too",
            "List all files including hidden ones (starting with .)",
            SuggestionReason::NewUserOnboarding,
            QueryCategory::FileOperations,
        ).with_relevance(0.90),

        SuggestedQuery::new(
            "find files by name",
            "Search for files matching a pattern",
            SuggestionReason::NewUserOnboarding,
            QueryCategory::FileOperations,
        ).with_relevance(0.85),

        SuggestedQuery::new(
            "search for text in files",
            "Find files containing specific text (grep)",
            SuggestionReason::NewUserOnboarding,
            QueryCategory::FileOperations,
        ).with_relevance(0.85),

        // System Info
        SuggestedQuery::new(
            "show disk usage",
            "See how much disk space is used",
            SuggestionReason::NewUserOnboarding,
            QueryCategory::System,
        ).with_relevance(0.75),

        SuggestedQuery::new(
            "show running processes",
            "See what programs are currently running",
            SuggestionReason::NewUserOnboarding,
            QueryCategory::System,
        ).with_relevance(0.70),

        // Learning Essential Tools
        SuggestedQuery::new(
            "show current directory path",
            "Print working directory (pwd)",
            SuggestionReason::NewUserOnboarding,
            QueryCategory::Learning,
        ).with_relevance(0.80),

        SuggestedQuery::new(
            "count lines in a file",
            "Use wc to count lines, words, or characters",
            SuggestionReason::NewUserOnboarding,
            QueryCategory::Learning,
        ).with_relevance(0.65),
    ]
}

/// Get Git-specific suggestions based on repository state
/// Caro loves Git!
pub fn get_git_suggestions(state: &GitState) -> Vec<SuggestedQuery> {
    let mut suggestions = Vec::new();

    // Always useful
    suggestions.push(
        SuggestedQuery::new(
            "show git status",
            "See current branch and changes",
            SuggestionReason::GitWorkflow {
                state: format!("on branch {}", state.branch),
            },
            QueryCategory::Git,
        ).with_relevance(0.95)
    );

    // Handle conflicts first (highest priority)
    if state.has_conflicts {
        suggestions.push(
            SuggestedQuery::new(
                "show files with merge conflicts",
                "List all conflicting files to resolve",
                SuggestionReason::GitWorkflow {
                    state: "merge conflicts detected".to_string(),
                },
                QueryCategory::Git,
            ).with_relevance(1.0)
        );

        suggestions.push(
            SuggestedQuery::new(
                "abort the current merge or rebase",
                "Cancel the in-progress operation",
                SuggestionReason::GitWorkflow {
                    state: "can abort if needed".to_string(),
                },
                QueryCategory::Git,
            ).with_relevance(0.85)
        );

        return suggestions;
    }

    // Handle uncommitted changes
    if !state.is_clean {
        if state.modified_count > 0 || state.untracked_count > 0 {
            suggestions.push(
                SuggestedQuery::new(
                    "show what changed in files",
                    "View diff of modified files",
                    SuggestionReason::GitWorkflow {
                        state: format!("{} modified, {} untracked", state.modified_count, state.untracked_count),
                    },
                    QueryCategory::Git,
                ).with_relevance(0.92)
            );

            suggestions.push(
                SuggestedQuery::new(
                    "stage all changes",
                    "Add all modified and new files to staging",
                    SuggestionReason::GitWorkflow {
                        state: "ready to stage".to_string(),
                    },
                    QueryCategory::Git,
                ).with_relevance(0.88)
            );
        }

        if state.staged_count > 0 {
            suggestions.push(
                SuggestedQuery::new(
                    "show staged changes",
                    "View diff of what will be committed",
                    SuggestionReason::GitWorkflow {
                        state: format!("{} files staged", state.staged_count),
                    },
                    QueryCategory::Git,
                ).with_relevance(0.90)
            );

            suggestions.push(
                SuggestedQuery::new(
                    "commit staged changes",
                    "Create a commit with staged files",
                    SuggestionReason::GitWorkflow {
                        state: "ready to commit".to_string(),
                    },
                    QueryCategory::Git,
                ).with_relevance(0.93)
            );
        }
    }

    // Handle push/pull
    if state.ahead > 0 {
        suggestions.push(
            SuggestedQuery::new(
                "push commits to remote",
                format!("Push {} commits to origin", state.ahead),
                SuggestionReason::GitWorkflow {
                    state: format!("{} commits ahead", state.ahead),
                },
                QueryCategory::Git,
            ).with_relevance(0.95)
        );
    }

    if state.behind > 0 {
        suggestions.push(
            SuggestedQuery::new(
                "pull changes from remote",
                format!("Fetch and merge {} new commits", state.behind),
                SuggestionReason::GitWorkflow {
                    state: format!("{} commits behind", state.behind),
                },
                QueryCategory::Git,
            ).with_relevance(0.95)
        );
    }

    // Clean repo suggestions
    if state.is_clean && state.ahead == 0 && state.behind == 0 {
        suggestions.push(
            SuggestedQuery::new(
                "show recent commit history",
                "View the last few commits",
                SuggestionReason::GitWorkflow {
                    state: "up to date".to_string(),
                },
                QueryCategory::Git,
            ).with_relevance(0.80)
        );

        suggestions.push(
            SuggestedQuery::new(
                "create a new branch",
                "Start work on a new feature",
                SuggestionReason::GitWorkflow {
                    state: "clean working tree".to_string(),
                },
                QueryCategory::Git,
            ).with_relevance(0.75)
        );
    }

    suggestions
}

/// Get Docker suggestions when docker is available
pub fn get_docker_suggestions() -> Vec<SuggestedQuery> {
    vec![
        SuggestedQuery::new(
            "list running containers",
            "Show all active Docker containers",
            SuggestionReason::DetectedTool { tool: "docker".to_string() },
            QueryCategory::Docker,
        ).with_relevance(0.90),

        SuggestedQuery::new(
            "list all containers including stopped",
            "Show all containers, running and stopped",
            SuggestionReason::DetectedTool { tool: "docker".to_string() },
            QueryCategory::Docker,
        ).with_relevance(0.85),

        SuggestedQuery::new(
            "list docker images",
            "Show all downloaded images",
            SuggestionReason::DetectedTool { tool: "docker".to_string() },
            QueryCategory::Docker,
        ).with_relevance(0.80),

        SuggestedQuery::new(
            "show container logs",
            "View logs from a running container",
            SuggestionReason::DetectedTool { tool: "docker".to_string() },
            QueryCategory::Docker,
        ).with_relevance(0.75),

        SuggestedQuery::new(
            "clean up unused docker resources",
            "Remove stopped containers and unused images",
            SuggestionReason::DetectedTool { tool: "docker".to_string() },
            QueryCategory::Docker,
        ).with_relevance(0.70),
    ]
}

/// Get suggestions based on detected project type
pub fn get_project_suggestions(project_type: &super::environment::ProjectType) -> Vec<SuggestedQuery> {
    match project_type {
        super::environment::ProjectType::Rust => vec![
            SuggestedQuery::new(
                "build the project",
                "Compile the Rust project",
                SuggestionReason::DirectoryContext { context: "Rust project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.90),

            SuggestedQuery::new(
                "run tests",
                "Execute cargo test",
                SuggestionReason::DirectoryContext { context: "Rust project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.88),

            SuggestedQuery::new(
                "check for errors without building",
                "Fast compilation check with cargo check",
                SuggestionReason::DirectoryContext { context: "Rust project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.85),

            SuggestedQuery::new(
                "format the code",
                "Run cargo fmt to format all Rust files",
                SuggestionReason::DirectoryContext { context: "Rust project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.80),
        ],

        super::environment::ProjectType::Node => vec![
            SuggestedQuery::new(
                "install dependencies",
                "Run npm/yarn install",
                SuggestionReason::DirectoryContext { context: "Node.js project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.90),

            SuggestedQuery::new(
                "run the development server",
                "Start the dev server",
                SuggestionReason::DirectoryContext { context: "Node.js project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.88),

            SuggestedQuery::new(
                "run tests",
                "Execute npm test",
                SuggestionReason::DirectoryContext { context: "Node.js project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.85),

            SuggestedQuery::new(
                "build for production",
                "Create production build",
                SuggestionReason::DirectoryContext { context: "Node.js project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.80),
        ],

        super::environment::ProjectType::Python => vec![
            SuggestedQuery::new(
                "install dependencies",
                "Install packages from requirements",
                SuggestionReason::DirectoryContext { context: "Python project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.90),

            SuggestedQuery::new(
                "run the main script",
                "Execute the Python application",
                SuggestionReason::DirectoryContext { context: "Python project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.85),

            SuggestedQuery::new(
                "run tests",
                "Execute pytest",
                SuggestionReason::DirectoryContext { context: "Python project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.85),

            SuggestedQuery::new(
                "create virtual environment",
                "Set up isolated Python environment",
                SuggestionReason::DirectoryContext { context: "Python project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.75),
        ],

        super::environment::ProjectType::Go => vec![
            SuggestedQuery::new(
                "build the project",
                "Compile with go build",
                SuggestionReason::DirectoryContext { context: "Go project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.90),

            SuggestedQuery::new(
                "run tests",
                "Execute go test",
                SuggestionReason::DirectoryContext { context: "Go project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.88),

            SuggestedQuery::new(
                "download dependencies",
                "Run go mod download",
                SuggestionReason::DirectoryContext { context: "Go project".to_string() },
                QueryCategory::Development,
            ).with_relevance(0.80),
        ],

        super::environment::ProjectType::Docker => vec![
            SuggestedQuery::new(
                "build the docker image",
                "Build from Dockerfile",
                SuggestionReason::DirectoryContext { context: "Docker project".to_string() },
                QueryCategory::Docker,
            ).with_relevance(0.90),

            SuggestedQuery::new(
                "start docker compose services",
                "Run docker-compose up",
                SuggestionReason::DirectoryContext { context: "Docker project".to_string() },
                QueryCategory::Docker,
            ).with_relevance(0.88),
        ],

        _ => vec![],
    }
}

/// Get suggestions based on frequently used commands
pub fn get_history_based_suggestions(top_commands: &[(String, u32)]) -> Vec<SuggestedQuery> {
    let mut suggestions = Vec::new();

    for (cmd, count) in top_commands.iter().take(5) {
        let suggestion = match cmd.as_str() {
            "git" => Some(SuggestedQuery::new(
                "show git status and recent changes",
                "Quick overview of repository state",
                SuggestionReason::HistoryPattern { command: cmd.clone() },
                QueryCategory::Git,
            )),
            "docker" => Some(SuggestedQuery::new(
                "show running docker containers",
                "List active containers",
                SuggestionReason::HistoryPattern { command: cmd.clone() },
                QueryCategory::Docker,
            )),
            "npm" | "yarn" | "pnpm" => Some(SuggestedQuery::new(
                "run npm scripts",
                "Execute project scripts",
                SuggestionReason::HistoryPattern { command: cmd.clone() },
                QueryCategory::Development,
            )),
            "cargo" => Some(SuggestedQuery::new(
                "build and test the project",
                "Compile and run tests",
                SuggestionReason::HistoryPattern { command: cmd.clone() },
                QueryCategory::Development,
            )),
            "python" | "python3" => Some(SuggestedQuery::new(
                "run python script",
                "Execute Python code",
                SuggestionReason::HistoryPattern { command: cmd.clone() },
                QueryCategory::Development,
            )),
            "find" => Some(SuggestedQuery::new(
                "find files modified recently",
                "Search for recently changed files",
                SuggestionReason::HistoryPattern { command: cmd.clone() },
                QueryCategory::FileOperations,
            )),
            "grep" | "rg" => Some(SuggestedQuery::new(
                "search for pattern in files",
                "Find text across files",
                SuggestionReason::HistoryPattern { command: cmd.clone() },
                QueryCategory::FileOperations,
            )),
            "kubectl" => Some(SuggestedQuery::new(
                "list kubernetes pods",
                "Show running pods in cluster",
                SuggestionReason::HistoryPattern { command: cmd.clone() },
                QueryCategory::System,
            )),
            "ssh" => Some(SuggestedQuery::new(
                "connect to remote server",
                "SSH into a machine",
                SuggestionReason::HistoryPattern { command: cmd.clone() },
                QueryCategory::Network,
            )),
            _ => None,
        };

        if let Some(mut s) = suggestion {
            // Higher relevance for more frequently used commands
            let relevance = 0.6 + (0.3 * (*count as f32 / 100.0).min(1.0));
            s = s.with_relevance(relevance);
            suggestions.push(s);
        }
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beginner_suggestions() {
        let suggestions = get_beginner_suggestions();
        assert!(!suggestions.is_empty());

        // Should include file listing
        assert!(suggestions.iter().any(|s| s.query.contains("list")));
    }

    #[test]
    fn test_git_suggestions_clean_repo() {
        let state = GitState {
            branch: "main".to_string(),
            is_clean: true,
            staged_count: 0,
            modified_count: 0,
            untracked_count: 0,
            ahead: 0,
            behind: 0,
            has_conflicts: false,
        };

        let suggestions = get_git_suggestions(&state);
        assert!(!suggestions.is_empty());

        // Should suggest viewing history for clean repo
        assert!(suggestions.iter().any(|s| s.query.contains("history")));
    }

    #[test]
    fn test_git_suggestions_with_changes() {
        let state = GitState {
            branch: "feature".to_string(),
            is_clean: false,
            staged_count: 0,
            modified_count: 3,
            untracked_count: 1,
            ahead: 0,
            behind: 0,
            has_conflicts: false,
        };

        let suggestions = get_git_suggestions(&state);

        // Should suggest staging changes
        assert!(suggestions.iter().any(|s| s.query.contains("stage")));
    }

    #[test]
    fn test_git_suggestions_with_conflicts() {
        let state = GitState {
            branch: "main".to_string(),
            is_clean: false,
            staged_count: 0,
            modified_count: 2,
            untracked_count: 0,
            ahead: 0,
            behind: 0,
            has_conflicts: true,
        };

        let suggestions = get_git_suggestions(&state);

        // Conflicts should be highest priority
        assert!(suggestions.iter().any(|s| s.query.contains("conflict")));
    }

    #[test]
    fn test_docker_suggestions() {
        let suggestions = get_docker_suggestions();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.query.contains("container")));
    }

    #[test]
    fn test_project_suggestions_rust() {
        let suggestions = get_project_suggestions(&super::super::environment::ProjectType::Rust);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.query.contains("build")));
    }

    #[test]
    fn test_history_based_suggestions() {
        let top_commands = vec![
            ("git".to_string(), 50),
            ("docker".to_string(), 30),
            ("ls".to_string(), 100),
        ];

        let suggestions = get_history_based_suggestions(&top_commands);

        // Should have git and docker suggestions
        assert!(suggestions.iter().any(|s| matches!(
            &s.reason,
            SuggestionReason::HistoryPattern { command } if command == "git"
        )));
    }
}
