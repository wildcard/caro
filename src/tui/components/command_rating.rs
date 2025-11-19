//! Command rating and voting component
//!
//! This component addresses the community's request for voting and ranking
//! past commands and AI-generated alternatives, enabling collective intelligence
//! and command quality improvement.

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct CommandRatingComponent;

#[derive(Debug, Clone)]
struct RatedCommand {
    query: &'static str,
    command: &'static str,
    upvotes: usize,
    downvotes: usize,
    user_vote: Option<Vote>,
    comments: usize,
    alternatives: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Vote {
    Up,
    Down,
}

impl RatedCommand {
    fn score(&self) -> i32 {
        self.upvotes as i32 - self.downvotes as i32
    }

    fn percentage(&self) -> f32 {
        let total = self.upvotes + self.downvotes;
        if total == 0 {
            0.0
        } else {
            (self.upvotes as f32 / total as f32) * 100.0
        }
    }
}

const RATED_COMMANDS: &[RatedCommand] = &[
    RatedCommand {
        query: "find large files over 100MB",
        command: "find . -type f -size +100M -exec ls -lh {} \\;",
        upvotes: 47,
        downvotes: 3,
        user_vote: None,
        comments: 5,
        alternatives: 3,
    },
    RatedCommand {
        query: "count lines in all Rust files",
        command: "find . -name '*.rs' | xargs wc -l | tail -1",
        upvotes: 32,
        downvotes: 8,
        user_vote: Some(Vote::Up),
        comments: 12,
        alternatives: 5,
    },
    RatedCommand {
        query: "show disk usage sorted by size",
        command: "du -ah | sort -hr | head -20",
        upvotes: 28,
        downvotes: 2,
        user_vote: None,
        comments: 3,
        alternatives: 2,
    },
    RatedCommand {
        query: "find and remove node_modules",
        command: "find . -name 'node_modules' -type d -prune -exec rm -rf '{}' +",
        upvotes: 15,
        downvotes: 25,
        user_vote: Some(Vote::Down),
        comments: 18,
        alternatives: 7,
    },
];

fn render_rating(
    frame: &mut Frame,
    area: Rect,
    view_type: &str,
    selected_idx: Option<usize>,
    show_details: bool,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(5),    // Command list
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Header
    let header = Paragraph::new("Community-Rated Commands")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Command list
    let mut lines = Vec::new();

    let sorted_commands: Vec<&RatedCommand> = if view_type == "top" {
        let mut cmds: Vec<&RatedCommand> = RATED_COMMANDS.iter().collect();
        cmds.sort_by(|a, b| b.score().cmp(&a.score()));
        cmds
    } else if view_type == "controversial" {
        let mut cmds: Vec<&RatedCommand> = RATED_COMMANDS.iter().collect();
        cmds.sort_by(|a, b| {
            let a_controversy = a.upvotes.min(a.downvotes);
            let b_controversy = b.upvotes.min(b.downvotes);
            b_controversy.cmp(&a_controversy)
        });
        cmds
    } else {
        RATED_COMMANDS.iter().collect()
    };

    for (idx, cmd) in sorted_commands.iter().enumerate() {
        let is_selected = selected_idx == Some(idx);
        let score = cmd.score();
        let score_color = if score > 20 {
            Color::Green
        } else if score > 0 {
            Color::Yellow
        } else {
            Color::Red
        };

        // Score and voting arrows
        let up_style = if cmd.user_vote == Some(Vote::Up) {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let down_style = if cmd.user_vote == Some(Vote::Down) {
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::from(vec![
            Span::styled(" ▲ ", up_style),
            Span::styled(
                format!("{:4}", score),
                Style::default()
                    .fg(score_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ▼", down_style),
            Span::raw("  │  "),
            Span::styled(
                cmd.query,
                if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                },
            ),
        ]));

        if show_details || is_selected {
            // Command line
            lines.push(Line::from(vec![
                Span::raw("        │  "),
                Span::styled("$ ", Style::default().fg(Color::Green)),
                Span::styled(cmd.command, Style::default().fg(Color::White)),
            ]));

            // Stats line
            lines.push(Line::from(vec![
                Span::raw("        │  "),
                Span::styled(
                    format!("{}% upvoted", cmd.percentage() as i32),
                    Style::default().fg(Color::Cyan),
                ),
                Span::raw("  │  "),
                Span::styled(
                    format!("{} comments", cmd.comments),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw("  │  "),
                Span::styled(
                    format!("{} alternatives", cmd.alternatives),
                    Style::default().fg(Color::Magenta),
                ),
            ]));

            // Vote breakdown
            lines.push(Line::from(vec![
                Span::raw("        │  "),
                Span::styled("▲ ", Style::default().fg(Color::Green)),
                Span::styled(cmd.upvotes.to_string(), Style::default().fg(Color::Green)),
                Span::raw("  "),
                Span::styled("▼ ", Style::default().fg(Color::Red)),
                Span::styled(cmd.downvotes.to_string(), Style::default().fg(Color::Red)),
            ]));

            lines.push(Line::from(""));
        }
    }

    let list = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(
        match view_type {
            "top" => "Top Rated",
            "controversial" => "Most Controversial",
            _ => "All Commands",
        },
    ));
    frame.render_widget(list, chunks[1]);

    // Footer
    let footer_text = "↑↓: Navigate │ Space: Vote │ C: Comments │ A: Alternatives │ S: Sort";

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(footer, chunks[2]);
}

fn render_voting_detail(frame: &mut Frame, area: Rect, cmd_idx: usize) {
    let cmd = &RATED_COMMANDS[cmd_idx];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Query and command
            Constraint::Length(8), // Voting stats
            Constraint::Min(5),    // Comments preview
            Constraint::Length(3), // Actions
        ])
        .split(area);

    // Query and command
    let query_lines = vec![
        Line::from(vec![
            Span::styled(
                "Query: ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(cmd.query),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "$ ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(cmd.command, Style::default().fg(Color::White)),
        ]),
    ];

    let query_block =
        Paragraph::new(query_lines).block(Block::default().borders(Borders::ALL).title("Command"));
    frame.render_widget(query_block, chunks[0]);

    // Voting stats
    let score = cmd.score();
    let percentage = cmd.percentage();
    let total_votes = cmd.upvotes + cmd.downvotes;

    let stats_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Score: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{:+}", score),
                Style::default()
                    .fg(if score > 0 { Color::Green } else { Color::Red })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  ({:.0}% upvoted)", percentage),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ▲ Upvotes:   ", Style::default().fg(Color::Green)),
            Span::styled(
                cmd.upvotes.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  ▼ Downvotes: ", Style::default().fg(Color::Red)),
            Span::styled(
                cmd.downvotes.to_string(),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Total Votes: ", Style::default().fg(Color::White)),
            Span::styled(
                total_votes.to_string(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let stats_block = Paragraph::new(stats_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Voting Statistics"),
    );
    frame.render_widget(stats_block, chunks[1]);

    // Comments preview
    let comments_lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "@rustdev42 ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("2h ago", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from("  This is a great command! Much better than using du."),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "@shellmaster ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("5h ago", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from("  Consider adding -prune to avoid traversing excluded dirs."),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {} more comments...", cmd.comments - 2),
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let comments_block = Paragraph::new(comments_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Comments ({})", cmd.comments)),
    );
    frame.render_widget(comments_block, chunks[2]);

    // Actions
    let actions = Paragraph::new(
        "↑: Upvote │ ↓: Downvote │ C: View All Comments │ A: View Alternatives │ Esc: Back",
    )
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL))
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(actions, chunks[3]);
}

impl ShowcaseComponent for CommandRatingComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "CommandRating",
            "Community voting and rating system for commands and alternatives",
        )
        .with_category("Input")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Command List",
                "List of rated commands with scores and voting arrows",
                |frame, area| {
                    render_rating(frame, area, "default", None, false);
                },
            ),
            ShowcaseStory::new(
                "With Selection",
                "Selected command showing detailed information",
                |frame, area| {
                    render_rating(frame, area, "default", Some(1), false);
                },
            ),
            ShowcaseStory::new(
                "Top Rated",
                "Commands sorted by highest score (most upvotes)",
                |frame, area| {
                    render_rating(frame, area, "top", None, true);
                },
            ),
            ShowcaseStory::new(
                "Controversial",
                "Commands with most mixed voting (high up and downvotes)",
                |frame, area| {
                    render_rating(frame, area, "controversial", None, true);
                },
            ),
            ShowcaseStory::new(
                "Voting Detail View",
                "Detailed view of a single command with voting stats and comments",
                |frame, area| {
                    render_voting_detail(frame, area, 0);
                },
            ),
            ShowcaseStory::new(
                "User Voted Up",
                "Command that the user has upvoted (green arrow)",
                |frame, area| {
                    render_rating(frame, area, "default", Some(1), true);
                },
            ),
            ShowcaseStory::new(
                "User Voted Down",
                "Command that the user has downvoted (red arrow)",
                |frame, area| {
                    render_rating(frame, area, "default", Some(3), true);
                },
            ),
        ]
    }
}
