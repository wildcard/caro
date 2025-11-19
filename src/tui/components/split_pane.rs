//! Split pane component for demonstrating layout patterns
//!
//! This component showcases split-screen layouts with resizable sections,
//! similar to tmux, vim, or IDE panels.

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Split pane component demonstrating various layout patterns
pub struct SplitPaneComponent;

impl ShowcaseComponent for SplitPaneComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "SplitPane",
            "Split-screen layouts with resizable sections and nested panes",
        )
        .with_category("Display")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            // Story 1: Vertical Split 50/50
            ShowcaseStory::new(
                "Vertical Split 50/50",
                "Two equal vertical panes side by side",
                |frame, area| {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(area);

                    let left_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  Left Pane"),
                        Line::from(""),
                        Line::from("  This is the left side"),
                        Line::from("  of a vertical split."),
                        Line::from(""),
                        Line::from("  50% width"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Left")
                            .style(Style::default().fg(Color::Cyan)),
                    );

                    let right_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  Right Pane"),
                        Line::from(""),
                        Line::from("  This is the right side"),
                        Line::from("  of a vertical split."),
                        Line::from(""),
                        Line::from("  50% width"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Right")
                            .style(Style::default().fg(Color::Magenta)),
                    );

                    frame.render_widget(left_pane, chunks[0]);
                    frame.render_widget(right_pane, chunks[1]);
                },
            ),
            // Story 2: Horizontal Split 50/50
            ShowcaseStory::new(
                "Horizontal Split 50/50",
                "Two equal horizontal panes stacked vertically",
                |frame, area| {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(area);

                    let top_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  Top Pane"),
                        Line::from(""),
                        Line::from("  This occupies the upper half"),
                        Line::from("  50% height"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Top")
                            .style(Style::default().fg(Color::Yellow)),
                    );

                    let bottom_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  Bottom Pane"),
                        Line::from(""),
                        Line::from("  This occupies the lower half"),
                        Line::from("  50% height"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Bottom")
                            .style(Style::default().fg(Color::Green)),
                    );

                    frame.render_widget(top_pane, chunks[0]);
                    frame.render_widget(bottom_pane, chunks[1]);
                },
            ),
            // Story 3: Asymmetric 70/30
            ShowcaseStory::new(
                "Asymmetric 70/30",
                "Asymmetric layout with 70% main area and 30% sidebar",
                |frame, area| {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                        .split(area);

                    let main_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  Main Content Area (70%)"),
                        Line::from(""),
                        Line::from("  This is the primary workspace,"),
                        Line::from("  taking up most of the screen."),
                        Line::from(""),
                        Line::from("  Perfect for code, documents,"),
                        Line::from("  or main application content."),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Main Content")
                            .style(Style::default().fg(Color::Blue)),
                    );

                    let sidebar_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from(" Sidebar (30%)"),
                        Line::from(""),
                        Line::from(" Navigation"),
                        Line::from(" Tools"),
                        Line::from(" Settings"),
                        Line::from(""),
                        Line::from(" Secondary"),
                        Line::from(" content"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Sidebar")
                            .style(Style::default().fg(Color::Gray)),
                    );

                    frame.render_widget(main_pane, chunks[0]);
                    frame.render_widget(sidebar_pane, chunks[1]);
                },
            ),
            // Story 4: Three-Way Split
            ShowcaseStory::new(
                "Three-Way Split",
                "Three equal columns for multi-panel layouts",
                |frame, area| {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(33),
                            Constraint::Percentage(34),
                            Constraint::Percentage(33),
                        ])
                        .split(area);

                    let left_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from(" Left Column"),
                        Line::from(""),
                        Line::from(" File tree"),
                        Line::from(" Navigation"),
                        Line::from(""),
                        Line::from(" 33% width"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Left (33%)")
                            .style(Style::default().fg(Color::Cyan)),
                    );

                    let center_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  Center Column"),
                        Line::from(""),
                        Line::from("  Main editor"),
                        Line::from("  Content area"),
                        Line::from(""),
                        Line::from("  34% width"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Center (34%)")
                            .style(Style::default().fg(Color::Yellow)),
                    );

                    let right_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from(" Right Column"),
                        Line::from(""),
                        Line::from(" Preview"),
                        Line::from(" Output"),
                        Line::from(""),
                        Line::from(" 33% width"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Right (33%)")
                            .style(Style::default().fg(Color::Magenta)),
                    );

                    frame.render_widget(left_pane, chunks[0]);
                    frame.render_widget(center_pane, chunks[1]);
                    frame.render_widget(right_pane, chunks[2]);
                },
            ),
            // Story 5: Nested Splits
            ShowcaseStory::new(
                "Nested Splits",
                "Split within a split - complex nested layout",
                |frame, area| {
                    // First split: left and right
                    let outer_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(area);

                    // Left pane - single panel
                    let left_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  Left Panel"),
                        Line::from(""),
                        Line::from("  This panel occupies"),
                        Line::from("  the entire left half."),
                        Line::from(""),
                        Line::from("  No nested splits here."),
                        Line::from(""),
                        Line::from("  50% width, full height"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Left Panel (50%)")
                            .style(Style::default().fg(Color::Blue)),
                    );

                    // Right side - nested vertical split
                    let inner_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(outer_chunks[1]);

                    let top_right_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  Top-Right"),
                        Line::from(""),
                        Line::from("  Nested split!"),
                        Line::from("  50% of right side"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Top-Right (25%)")
                            .style(Style::default().fg(Color::Green)),
                    );

                    let bottom_right_pane = Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  Bottom-Right"),
                        Line::from(""),
                        Line::from("  Nested split!"),
                        Line::from("  50% of right side"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Bottom-Right (25%)")
                            .style(Style::default().fg(Color::Magenta)),
                    );

                    frame.render_widget(left_pane, outer_chunks[0]);
                    frame.render_widget(top_right_pane, inner_chunks[0]);
                    frame.render_widget(bottom_right_pane, inner_chunks[1]);
                },
            ),
            // Story 6: With Borders and Titles
            ShowcaseStory::new(
                "With Borders and Titles",
                "Styled panes with custom borders and titles",
                |frame, area| {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(area);

                    let fancy_left = Paragraph::new(vec![
                        Line::from(""),
                        Line::from(vec![
                            Span::styled(
                                "  Styled Content",
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]),
                        Line::from(""),
                        Line::from(vec![
                            Span::raw("  Custom "),
                            Span::styled("colors", Style::default().fg(Color::Green)),
                            Span::raw(" and "),
                            Span::styled(
                                "styles",
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::ITALIC),
                            ),
                        ]),
                        Line::from(""),
                        Line::from("  Beautiful borders"),
                        Line::from("  Descriptive titles"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Editor Panel ")
                            .title_style(
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .border_style(Style::default().fg(Color::Cyan)),
                    );

                    let fancy_right = Paragraph::new(vec![
                        Line::from(""),
                        Line::from(vec![
                            Span::styled(
                                "  Terminal Output",
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]),
                        Line::from(""),
                        Line::from(vec![
                            Span::raw("  $ "),
                            Span::styled("cargo", Style::default().fg(Color::Yellow)),
                            Span::raw(" test"),
                        ]),
                        Line::from(vec![
                            Span::styled(
                                "  ✓ All tests passed!",
                                Style::default().fg(Color::Green),
                            ),
                        ]),
                        Line::from(""),
                        Line::from("  Themed borders"),
                        Line::from("  Styled titles"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Terminal ")
                            .title_style(
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            )
                            .border_style(Style::default().fg(Color::Green)),
                    );

                    frame.render_widget(fancy_left, chunks[0]);
                    frame.render_widget(fancy_right, chunks[1]);
                },
            ),
            // Story 7: Highlighted Active Pane
            ShowcaseStory::new(
                "Highlighted Active Pane",
                "Visual indication of which pane has focus",
                |frame, area| {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(33),
                            Constraint::Percentage(34),
                            Constraint::Percentage(33),
                        ])
                        .split(area);

                    // Inactive pane (dim)
                    let inactive_left = Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  Inactive"),
                        Line::from(""),
                        Line::from("  This pane is"),
                        Line::from("  not focused."),
                        Line::from(""),
                        Line::from("  Dim borders"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Inactive")
                            .style(Style::default().fg(Color::DarkGray)),
                    );

                    // Active pane (highlighted)
                    let active_center = Paragraph::new(vec![
                        Line::from(""),
                        Line::from(vec![
                            Span::styled(
                                "  ACTIVE PANE",
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]),
                        Line::from(""),
                        Line::from(vec![
                            Span::styled("  This pane has focus!", Style::default().fg(Color::White)),
                        ]),
                        Line::from(""),
                        Line::from(vec![
                            Span::styled(
                                "  Bright borders",
                                Style::default().fg(Color::Yellow),
                            ),
                        ]),
                        Line::from(vec![
                            Span::styled(
                                "  Bold title",
                                Style::default().fg(Color::Yellow),
                            ),
                        ]),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" ACTIVE ")
                            .title_style(
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD | Modifier::SLOW_BLINK),
                            )
                            .border_style(Style::default().fg(Color::Yellow)),
                    );

                    // Inactive pane (dim)
                    let inactive_right = Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  Inactive"),
                        Line::from(""),
                        Line::from("  This pane is"),
                        Line::from("  not focused."),
                        Line::from(""),
                        Line::from("  Dim borders"),
                    ])
                    .alignment(Alignment::Left)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Inactive")
                            .style(Style::default().fg(Color::DarkGray)),
                    );

                    frame.render_widget(inactive_left, chunks[0]);
                    frame.render_widget(active_center, chunks[1]);
                    frame.render_widget(inactive_right, chunks[2]);
                },
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    // ========================================================================
    // METADATA TESTS (TDD Phase 1)
    // ========================================================================

    #[test]
    fn test_metadata_has_correct_name() {
        let component = SplitPaneComponent;
        let metadata = component.metadata();
        assert_eq!(metadata.name, "SplitPane");
    }

    #[test]
    fn test_metadata_has_correct_description() {
        let component = SplitPaneComponent;
        let metadata = component.metadata();
        assert_eq!(
            metadata.description,
            "Split-screen layouts with resizable sections and nested panes"
        );
    }

    #[test]
    fn test_metadata_has_display_category() {
        let component = SplitPaneComponent;
        let metadata = component.metadata();
        assert_eq!(metadata.category, "Display");
    }

    #[test]
    fn test_metadata_has_version() {
        let component = SplitPaneComponent;
        let metadata = component.metadata();
        assert_eq!(metadata.version, "1.0.0");
    }

    // ========================================================================
    // STORY STRUCTURE TESTS (TDD Phase 2)
    // ========================================================================

    #[test]
    fn test_component_has_at_least_six_stories() {
        let component = SplitPaneComponent;
        let stories = component.stories();
        assert!(
            stories.len() >= 6,
            "SplitPaneComponent should have at least 6 stories, found {}",
            stories.len()
        );
    }

    #[test]
    fn test_story_names_are_descriptive() {
        let component = SplitPaneComponent;
        let stories = component.stories();

        let expected_names = vec![
            "Vertical Split 50/50",
            "Horizontal Split 50/50",
            "Asymmetric 70/30",
            "Three-Way Split",
            "Nested Splits",
            "With Borders and Titles",
            "Highlighted Active Pane",
        ];

        for (i, expected_name) in expected_names.iter().enumerate() {
            assert!(
                stories.get(i).is_some(),
                "Missing story at index {}: '{}'",
                i,
                expected_name
            );
            assert_eq!(
                stories[i].name, *expected_name,
                "Story {} name mismatch",
                i
            );
        }
    }

    #[test]
    fn test_all_stories_have_descriptions() {
        let component = SplitPaneComponent;
        let stories = component.stories();

        for story in stories {
            assert!(
                !story.description.is_empty(),
                "Story '{}' should have a non-empty description",
                story.name
            );
        }
    }

    // ========================================================================
    // RENDER TESTS (TDD Phase 3)
    // ========================================================================

    #[test]
    fn test_all_stories_render_without_panic() {
        let component = SplitPaneComponent;
        let stories = component.stories();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        for story in stories {
            let result = terminal.draw(|frame| {
                let area = frame.size();
                (story.render)(frame, area);
            });

            assert!(
                result.is_ok(),
                "Story '{}' should render without errors",
                story.name
            );
        }
    }

    #[test]
    fn test_renders_with_small_terminal() {
        let component = SplitPaneComponent;
        let stories = component.stories();

        let backend = TestBackend::new(30, 10);
        let mut terminal = Terminal::new(backend).unwrap();

        for story in stories {
            let result = terminal.draw(|frame| {
                let area = frame.size();
                (story.render)(frame, area);
            });

            assert!(
                result.is_ok(),
                "Story '{}' should handle small terminal sizes",
                story.name
            );
        }
    }

    #[test]
    fn test_renders_with_large_terminal() {
        let component = SplitPaneComponent;
        let stories = component.stories();

        let backend = TestBackend::new(200, 60);
        let mut terminal = Terminal::new(backend).unwrap();

        for story in stories {
            let result = terminal.draw(|frame| {
                let area = frame.size();
                (story.render)(frame, area);
            });

            assert!(
                result.is_ok(),
                "Story '{}' should handle large terminal sizes",
                story.name
            );
        }
    }

    // ========================================================================
    // LAYOUT CALCULATION TESTS (TDD Phase 4)
    // ========================================================================

    #[test]
    fn test_vertical_split_creates_two_equal_areas() {
        let area = Rect::new(0, 0, 100, 50);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].width, 50);
        assert_eq!(chunks[1].width, 50);
    }

    #[test]
    fn test_horizontal_split_creates_two_equal_areas() {
        let area = Rect::new(0, 0, 100, 50);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].height, 25);
        assert_eq!(chunks[1].height, 25);
    }

    #[test]
    fn test_asymmetric_split_70_30() {
        let area = Rect::new(0, 0, 100, 50);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].width > chunks[1].width);
    }

    #[test]
    fn test_three_way_split_creates_three_areas() {
        let area = Rect::new(0, 0, 100, 50);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(area);

        assert_eq!(chunks.len(), 3);
    }

    #[test]
    fn test_nested_split_creates_correct_structure() {
        let area = Rect::new(0, 0, 100, 50);

        // First split: left and right
        let outer_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Nested split on the right side
        let inner_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(outer_chunks[1]);

        assert_eq!(outer_chunks.len(), 2);
        assert_eq!(inner_chunks.len(), 2);
    }
}
