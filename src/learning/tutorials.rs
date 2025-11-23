//! Interactive tutorial system for learning shell commands
//!
//! Provides step-by-step lessons with hands-on exercises, progress tracking,
//! and spaced repetition.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Tutorial difficulty level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

/// Complete tutorial with multiple lessons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    pub id: String,
    pub title: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub lessons: Vec<Lesson>,
}

/// Individual lesson within a tutorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub title: String,
    pub explanation: String,
    pub example_command: String,
    pub expected_output: String,
    pub hints: Vec<String>,
    #[serde(default)]
    pub quiz: Option<Quiz>,
}

/// Quiz question for a lesson
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quiz {
    pub question: String,
    pub answer: String,
    #[serde(default)]
    pub hints: Vec<String>,
}

/// Result of completing a tutorial
#[derive(Debug, Clone)]
pub struct TutorialResult {
    pub tutorial_id: String,
    pub lessons_completed: usize,
    pub total_lessons: usize,
    pub score: f32,
}

impl Tutorial {
    /// Load tutorial from YAML file
    pub fn load(tutorial_id: &str) -> Result<Self> {
        // Try to load from tutorials directory
        let tutorial_path = PathBuf::from("tutorials").join(format!("{}.yaml", tutorial_id));

        if tutorial_path.exists() {
            let content = std::fs::read_to_string(&tutorial_path)
                .context("Failed to read tutorial file")?;
            let tutorial: Tutorial =
                serde_yaml::from_str(&content).context("Failed to parse tutorial YAML")?;
            Ok(tutorial)
        } else {
            // Load embedded tutorial
            Self::load_embedded(tutorial_id)
        }
    }

    /// Load embedded tutorial (built-in tutorials)
    fn load_embedded(tutorial_id: &str) -> Result<Self> {
        match tutorial_id {
            "find-basics" => Ok(Self::create_find_basics_tutorial()),
            "grep-basics" => Ok(Self::create_grep_basics_tutorial()),
            _ => anyhow::bail!("Tutorial '{}' not found", tutorial_id),
        }
    }

    /// Create find basics tutorial
    fn create_find_basics_tutorial() -> Self {
        Self {
            id: "find-basics".to_string(),
            title: "Mastering the find Command".to_string(),
            description: "Learn how to search for files and directories effectively".to_string(),
            difficulty: Difficulty::Beginner,
            lessons: vec![
                Lesson {
                    title: "Finding Files by Name".to_string(),
                    explanation: "The find command searches for files matching patterns. Use -name for case-sensitive search.".to_string(),
                    example_command: "find . -name '*.txt'".to_string(),
                    expected_output: "Lists all .txt files in current directory and subdirectories".to_string(),
                    hints: vec![
                        "The . means current directory".to_string(),
                        "Use quotes around patterns with wildcards".to_string(),
                    ],
                    quiz: Some(Quiz {
                        question: "How would you find all .log files?".to_string(),
                        answer: "find . -name '*.log'".to_string(),
                        hints: vec!["Use the same pattern as .txt but with .log extension".to_string()],
                    }),
                },
                Lesson {
                    title: "Finding by Type".to_string(),
                    explanation: "Use -type to filter by file type: f=file, d=directory, l=symlink".to_string(),
                    example_command: "find . -type f -name '*.sh'".to_string(),
                    expected_output: "Lists only .sh files (not directories)".to_string(),
                    hints: vec![
                        "The -type f filters for regular files only".to_string(),
                        "Combine multiple options for more specific searches".to_string(),
                    ],
                    quiz: Some(Quiz {
                        question: "How would you find only directories named 'test'?".to_string(),
                        answer: "find . -type d -name 'test'".to_string(),
                        hints: vec!["Use -type d for directories".to_string()],
                    }),
                },
                Lesson {
                    title: "Finding by Modification Time".to_string(),
                    explanation: "Use -mtime to find files by age. -mtime -7 means modified in last 7 days.".to_string(),
                    example_command: "find . -type f -mtime -7".to_string(),
                    expected_output: "Lists files modified in the last 7 days".to_string(),
                    hints: vec![
                        "-mtime -N means less than N days ago".to_string(),
                        "-mtime +N means more than N days ago".to_string(),
                    ],
                    quiz: Some(Quiz {
                        question: "How would you find files older than 30 days?".to_string(),
                        answer: "find . -type f -mtime +30".to_string(),
                        hints: vec!["Use + for 'more than'".to_string()],
                    }),
                },
            ],
        }
    }

    /// Create grep basics tutorial
    fn create_grep_basics_tutorial() -> Self {
        Self {
            id: "grep-basics".to_string(),
            title: "Mastering the grep Command".to_string(),
            description: "Learn how to search text patterns effectively".to_string(),
            difficulty: Difficulty::Beginner,
            lessons: vec![
                Lesson {
                    title: "Basic Pattern Search".to_string(),
                    explanation: "grep searches for patterns in files and outputs matching lines.".to_string(),
                    example_command: "grep 'error' logfile.txt".to_string(),
                    expected_output: "Shows all lines containing 'error'".to_string(),
                    hints: vec![
                        "Pattern is case-sensitive by default".to_string(),
                        "Use quotes around patterns with spaces".to_string(),
                    ],
                    quiz: Some(Quiz {
                        question: "How would you search for 'warning' in a file?".to_string(),
                        answer: "grep 'warning' filename".to_string(),
                        hints: vec!["Same syntax, just change the pattern".to_string()],
                    }),
                },
                Lesson {
                    title: "Case-Insensitive Search".to_string(),
                    explanation: "Use -i flag to ignore case differences.".to_string(),
                    example_command: "grep -i 'error' logfile.txt".to_string(),
                    expected_output: "Finds 'error', 'Error', 'ERROR', etc.".to_string(),
                    hints: vec![
                        "The -i flag makes search case-insensitive".to_string(),
                    ],
                    quiz: Some(Quiz {
                        question: "How would you search for 'todo' regardless of case?".to_string(),
                        answer: "grep -i 'todo' filename".to_string(),
                        hints: vec!["Add the -i flag".to_string()],
                    }),
                },
                Lesson {
                    title: "Recursive Search".to_string(),
                    explanation: "Use -r to search all files in a directory tree.".to_string(),
                    example_command: "grep -r 'TODO' src/".to_string(),
                    expected_output: "Searches all files under src/ directory".to_string(),
                    hints: vec![
                        "The -r flag searches recursively through directories".to_string(),
                        "Combine -r with -i for case-insensitive recursive search".to_string(),
                    ],
                    quiz: Some(Quiz {
                        question: "How would you recursively search for 'function' in current directory?".to_string(),
                        answer: "grep -r 'function' .".to_string(),
                        hints: vec!["Use . for current directory".to_string()],
                    }),
                },
            ],
        }
    }

    /// Get list of available tutorials
    pub fn list_available() -> Vec<TutorialInfo> {
        vec![
            TutorialInfo {
                id: "find-basics".to_string(),
                title: "Mastering the find Command".to_string(),
                difficulty: Difficulty::Beginner,
                lessons_count: 3,
            },
            TutorialInfo {
                id: "grep-basics".to_string(),
                title: "Mastering the grep Command".to_string(),
                difficulty: Difficulty::Beginner,
                lessons_count: 3,
            },
        ]
    }
}

/// Tutorial information (metadata only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialInfo {
    pub id: String,
    pub title: String,
    pub difficulty: Difficulty,
    pub lessons_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_embedded_find_tutorial() {
        let tutorial = Tutorial::load("find-basics").unwrap();

        assert_eq!(tutorial.id, "find-basics");
        assert_eq!(tutorial.difficulty, Difficulty::Beginner);
        assert_eq!(tutorial.lessons.len(), 3);
    }

    #[test]
    fn test_load_embedded_grep_tutorial() {
        let tutorial = Tutorial::load("grep-basics").unwrap();

        assert_eq!(tutorial.id, "grep-basics");
        assert_eq!(tutorial.lessons.len(), 3);
    }

    #[test]
    fn test_list_available() {
        let tutorials = Tutorial::list_available();

        assert_eq!(tutorials.len(), 2);
        assert!(tutorials.iter().any(|t| t.id == "find-basics"));
        assert!(tutorials.iter().any(|t| t.id == "grep-basics"));
    }

    #[test]
    fn test_tutorial_has_quizzes() {
        let tutorial = Tutorial::load("find-basics").unwrap();

        for lesson in &tutorial.lessons {
            assert!(lesson.quiz.is_some());
        }
    }
}
