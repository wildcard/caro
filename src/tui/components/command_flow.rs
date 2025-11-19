//! Complete command generation flow component
//!
//! This component demonstrates the full cmdai workflow from input to execution,
//! showing how multiple components can work together in a cohesive user experience.

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct CommandFlowComponent;

/// Workflow steps in the command generation process
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlowStep {
    Input,
    Generating,
    SafetyCheck,
    Confirmation,
    Executing,
    Complete,
}

impl FlowStep {
    fn label(&self) -> &str {
        match self {
            FlowStep::Input => "1. Input",
            FlowStep::Generating => "2. Generating",
            FlowStep::SafetyCheck => "3. Safety Check",
            FlowStep::Confirmation => "4. Confirmation",
            FlowStep::Executing => "5. Executing",
            FlowStep::Complete => "6. Complete",
        }
    }

    fn color(&self, current: FlowStep) -> Color {
        if *self as u8 == current as u8 {
            Color::Green
        } else if (*self as u8) < (current as u8) {
            Color::Cyan
        } else {
            Color::DarkGray
        }
    }

    fn icon(&self, current: FlowStep) -> &str {
        if *self as u8 == current as u8 {
            "▶"
        } else if (*self as u8) < (current as u8) {
            "✓"
        } else {
            "○"
        }
    }
}

fn render_flow(frame: &mut Frame, area: Rect, current_step: FlowStep, query: &str, command: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Title
            Constraint::Length(10), // Flow visualization
            Constraint::Min(5),     // Content
        ])
        .split(area);

    // Title
    let title = Paragraph::new("Command Generation Workflow")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Flow visualization
    let steps = [
        FlowStep::Input,
        FlowStep::Generating,
        FlowStep::SafetyCheck,
        FlowStep::Confirmation,
        FlowStep::Executing,
        FlowStep::Complete,
    ];

    let mut flow_lines = vec![Line::from("")];

    for step in &steps {
        let icon = step.icon(current_step);
        let label = step.label();
        let color = step.color(current_step);

        flow_lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("{} ", icon),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(label, Style::default().fg(color)),
        ]));
    }

    let flow_viz = Paragraph::new(flow_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Progress"),
    );
    frame.render_widget(flow_viz, chunks[1]);

    // Content based on current step
    let content = match current_step {
        FlowStep::Input => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Query: ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(format!("  {}", query)),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Press Enter to generate command", Style::default().fg(Color::DarkGray)),
                ]),
            ]
        }
        FlowStep::Generating => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("⠋ ", Style::default().fg(Color::Cyan)),
                    Span::raw("Generating command from natural language..."),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Backend: ", Style::default().fg(Color::Yellow)),
                    Span::raw("Embedded MLX (Apple Silicon)"),
                ]),
                Line::from(vec![
                    Span::styled("Model: ", Style::default().fg(Color::Yellow)),
                    Span::raw("mlx-community/Qwen2.5-Coder-1.5B-Instruct"),
                ]),
            ]
        }
        FlowStep::SafetyCheck => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Generated Command:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("$ ", Style::default().fg(Color::Green)),
                    Span::raw(command),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("✓ ", Style::default().fg(Color::Green)),
                    Span::styled("SAFE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::raw(" - This command is safe to execute"),
                ]),
            ]
        }
        FlowStep::Confirmation => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("Command: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("$ ", Style::default().fg(Color::Green)),
                    Span::raw(command),
                ]),
                Line::from(""),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Execute this command? ", Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(" Yes ", Style::default().fg(Color::Black).bg(Color::Green)),
                    Span::raw("   "),
                    Span::styled(" No ", Style::default().fg(Color::Red)),
                ]),
            ]
        }
        FlowStep::Executing => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("⠋ ", Style::default().fg(Color::Green)),
                    Span::raw("Executing command..."),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("$ ", Style::default().fg(Color::Green)),
                    Span::raw(command),
                ]),
            ]
        }
        FlowStep::Complete => {
            vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("✓ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled("Command executed successfully!", Style::default().fg(Color::Green)),
                ]),
                Line::from(""),
                Line::from("Output:"),
                Line::from(""),
                Line::from("  file1.txt"),
                Line::from("  file2.txt"),
                Line::from("  file3.txt"),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Completed in 1.2s", Style::default().fg(Color::DarkGray)),
                ]),
            ]
        }
    };

    let content_widget = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Current Step"),
    );
    frame.render_widget(content_widget, chunks[2]);
}

impl ShowcaseComponent for CommandFlowComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "CommandFlow",
            "Complete command generation workflow from input to execution",
        )
        .with_category("Workflow")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            ShowcaseStory::new(
                "Step 1: Input",
                "User enters natural language query",
                |frame, area| {
                    render_flow(
                        frame,
                        area,
                        FlowStep::Input,
                        "list all PDF files larger than 10MB",
                        "",
                    );
                },
            ),
            ShowcaseStory::new(
                "Step 2: Generating",
                "AI model generates shell command from query",
                |frame, area| {
                    render_flow(
                        frame,
                        area,
                        FlowStep::Generating,
                        "list all PDF files larger than 10MB",
                        "",
                    );
                },
            ),
            ShowcaseStory::new(
                "Step 3: Safety Check",
                "Command passes through safety validation",
                |frame, area| {
                    render_flow(
                        frame,
                        area,
                        FlowStep::SafetyCheck,
                        "list all PDF files larger than 10MB",
                        "find . -name '*.pdf' -size +10M -ls",
                    );
                },
            ),
            ShowcaseStory::new(
                "Step 4: Confirmation",
                "User confirms before execution",
                |frame, area| {
                    render_flow(
                        frame,
                        area,
                        FlowStep::Confirmation,
                        "list all PDF files larger than 10MB",
                        "find . -name '*.pdf' -size +10M -ls",
                    );
                },
            ),
            ShowcaseStory::new(
                "Step 5: Executing",
                "Command is being executed",
                |frame, area| {
                    render_flow(
                        frame,
                        area,
                        FlowStep::Executing,
                        "list all PDF files larger than 10MB",
                        "find . -name '*.pdf' -size +10M -ls",
                    );
                },
            ),
            ShowcaseStory::new(
                "Step 6: Complete",
                "Command completed successfully with output",
                |frame, area| {
                    render_flow(
                        frame,
                        area,
                        FlowStep::Complete,
                        "list all PDF files larger than 10MB",
                        "find . -name '*.pdf' -size +10M -ls",
                    );
                },
            ),
        ]
    }
}
