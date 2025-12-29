/**
 * Code-Heavy Article Demo
 * ========================
 * Example article with lots of syntax highlighted code blocks.
 */

import React from 'react';
import { ArticlePage } from '../ArticlePage';
import { CodeBlock } from '../../components/CodeBlock';

export function CodeHeavyArticle() {
  return (
    <ArticlePage
      title="Building a CLI with Rust and Clap"
      subtitle="A step-by-step guide to creating powerful command-line applications with Rust's clap library."
      category="Rust"
      categorySlug="rust"
      author={{
        name: 'Alex Chen',
        avatar: '/images/authors/alex.jpg',
        bio: 'Rust enthusiast and CLI developer. Building tools that developers love.',
      }}
      publishedAt="2025-12-15"
      readingTime="12 min read"
      coverImage="/images/articles/rust-cli-cover.jpg"
      coverImageAlt="Terminal window with Rust code"
      variant="code-heavy"
    >
      <p>
        Command-line interfaces remain one of the most powerful ways to interact
        with software. In this guide, we'll build a complete CLI application
        using Rust and the clap library.
      </p>

      <h2>Setting Up the Project</h2>

      <p>
        First, let's create a new Rust project and add our dependencies:
      </p>

      <CodeBlock
        language="bash"
        filename="Terminal"
        code={`cargo new my-cli
cd my-cli
cargo add clap --features derive
cargo add anyhow`}
      />

      <p>
        The <code>derive</code> feature enables clap's derive macro, which
        provides a declarative way to define CLI arguments.
      </p>

      <h2>Defining CLI Arguments</h2>

      <p>
        Let's define our CLI structure using clap's derive macros:
      </p>

      <CodeBlock
        language="rust"
        filename="src/main.rs"
        showLineNumbers
        highlightLines={[4, 5, 6, 7, 8]}
        code={`use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "my-cli")]
#[command(author = "Your Name")]
#[command(version = "1.0")]
#[command(about = "A powerful CLI tool")]
#[command(long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// The command to execute
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        /// The project name
        #[arg(short, long)]
        name: String,
    },
    /// Build the project
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },
}`}
      />

      <h2>Implementing Command Handlers</h2>

      <p>
        Now let's implement handlers for each subcommand:
      </p>

      <CodeBlock
        language="rust"
        filename="src/main.rs"
        showLineNumbers
        code={`fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("Verbose mode enabled");
    }

    match cli.command {
        Commands::Init { name } => {
            println!("Initializing project: {}", name);
            init_project(&name)?;
        }
        Commands::Build { release } => {
            let mode = if release { "release" } else { "debug" };
            println!("Building in {} mode", mode);
            build_project(release)?;
        }
    }

    Ok(())
}

fn init_project(name: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(name)?;
    std::fs::write(
        format!("{}/config.toml", name),
        "[project]\\nname = \\"{}\\""
    )?;
    Ok(())
}

fn build_project(release: bool) -> anyhow::Result<()> {
    let status = std::process::Command::new("cargo")
        .arg("build")
        .args(if release { vec!["--release"] } else { vec![] })
        .status()?;

    if !status.success() {
        anyhow::bail!("Build failed");
    }
    Ok(())
}`}
      />

      <h2>Adding Colors and Progress</h2>

      <p>
        Let's make our CLI more user-friendly with colored output:
      </p>

      <CodeBlock
        language="rust"
        filename="src/output.rs"
        code={`use colored::Colorize;

pub fn success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

pub fn info(message: &str) {
    println!("{} {}", "→".blue().bold(), message);
}

pub fn warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
}`}
      />

      <h2>Testing Your CLI</h2>

      <p>
        Finally, let's add some integration tests:
      </p>

      <CodeBlock
        language="rust"
        filename="tests/integration.rs"
        showLineNumbers
        code={`use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("my-cli").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("A powerful CLI tool"));
}

#[test]
fn test_init_command() {
    let mut cmd = Command::cargo_bin("my-cli").unwrap();
    cmd.args(["init", "--name", "test-project"])
        .assert()
        .success()
        .stdout(contains("Initializing project"));
}`}
      />

      <h2>Conclusion</h2>

      <p>
        You now have a fully functional CLI application with subcommands,
        colored output, and tests. The clap library makes it easy to build
        professional-grade command-line tools in Rust.
      </p>

      <p>
        Check out the{' '}
        <a href="https://docs.rs/clap">clap documentation</a> for more
        advanced features like custom validators and shell completions.
      </p>
    </ArticlePage>
  );
}

export default CodeHeavyArticle;
