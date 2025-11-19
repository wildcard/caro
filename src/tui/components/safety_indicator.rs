//! Safety indicator component for showing command risk levels

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct SafetyIndicatorComponent;

/// Safety risk levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyLevel {
    Safe,
    Moderate,
    High,
    Critical,
}

impl SafetyLevel {
    fn color(&self) -> Color {
        match self {
            SafetyLevel::Safe => Color::Green,
            SafetyLevel::Moderate => Color::Yellow,
            SafetyLevel::High => Color::LightRed,
            SafetyLevel::Critical => Color::Red,
        }
    }

    fn icon(&self) -> &str {
        match self {
            SafetyLevel::Safe => "✓",
            SafetyLevel::Moderate => "⚠",
            SafetyLevel::High => "⚠",
            SafetyLevel::Critical => "✗",
        }
    }

    fn label(&self) -> &str {
        match self {
            SafetyLevel::Safe => "SAFE",
            SafetyLevel::Moderate => "MODERATE",
            SafetyLevel::High => "HIGH RISK",
            SafetyLevel::Critical => "CRITICAL",
        }
    }

    fn description(&self) -> &str {
        match self {
            SafetyLevel::Safe => "This command is safe to execute",
            SafetyLevel::Moderate => "This command requires caution",
            SafetyLevel::High => "This command may cause unintended changes",
            SafetyLevel::Critical => "This command is dangerous and should not be executed",
        }
    }
}

fn render_safety_indicator(frame: &mut Frame, area: Rect, level: SafetyLevel, command: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    // Safety level indicator
    let level_text = vec![Line::from(vec![
        Span::styled(
            format!(" {} ", level.icon()),
            Style::default()
                .fg(level.color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            level.label(),
            Style::default()
                .fg(level.color())
                .add_modifier(Modifier::BOLD),
        ),
    ])];

    let level_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(level.color()))
        .title("Safety Level");

    let level_paragraph = Paragraph::new(level_text).block(level_block);
    frame.render_widget(level_paragraph, chunks[0]);

    // Command display
    let cmd_text = vec![Line::from(vec![
        Span::styled("$ ", Style::default().fg(Color::White)),
        Span::styled(command, Style::default().fg(Color::White)),
    ])];

    let cmd_block = Block::default().borders(Borders::ALL).title("Command");

    let cmd_paragraph = Paragraph::new(cmd_text).block(cmd_block);
    frame.render_widget(cmd_paragraph, chunks[1]);

    // Description
    let desc_text = vec![
        Line::from(""),
        Line::from(level.description()),
    ];

    let desc_block = Block::default()
        .borders(Borders::ALL)
        .title("Description");

    let desc_paragraph = Paragraph::new(desc_text).block(desc_block);
    frame.render_widget(desc_paragraph, chunks[2]);
}

impl ShowcaseComponent for SafetyIndicatorComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "SafetyIndicator",
            "Visual indicator for command safety levels",
        )
        .with_category("Feedback")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Safe Command",
                "Green indicator for safe commands",
                |frame, area| {
                    render_safety_indicator(frame, area, SafetyLevel::Safe, "ls -la");
                },
            ),
            ShowcaseStory::new(
                "Moderate Risk",
                "Yellow indicator for moderate risk commands",
                |frame, area| {
                    render_safety_indicator(
                        frame,
                        area,
                        SafetyLevel::Moderate,
                        "chmod 644 file.txt",
                    );
                },
            ),
            ShowcaseStory::new(
                "High Risk",
                "Orange/Light Red indicator for high risk commands",
                |frame, area| {
                    render_safety_indicator(
                        frame,
                        area,
                        SafetyLevel::High,
                        "rm -rf ./target",
                    );
                },
            ),
            ShowcaseStory::new(
                "Critical Risk",
                "Red indicator for critical/dangerous commands",
                |frame, area| {
                    render_safety_indicator(
                        frame,
                        area,
                        SafetyLevel::Critical,
                        "sudo rm -rf /",
                    );
                },
            ),
        ]
    }
}
