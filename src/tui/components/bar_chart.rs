//! Bar Chart Component
//!
//! Displays data as ASCII/Unicode bar charts with labels and values.
//! Supports horizontal and vertical orientations, multiple colors,
//! stacked bars, and percentage views.
//!
//! ## Features
//! - Horizontal and vertical bar charts
//! - Stacked bars with multiple segments
//! - Color-coded bars and segments
//! - Automatic scaling to available space
//! - Labels and value display
//! - Percentage view mode
//! - Grouped bars for side-by-side comparison
//!
//! ## Example
//! ```text
//! Sales     ▓▓▓▓▓▓▓▓▓▓▓▓░░░░  75%
//! Marketing ▓▓▓▓▓▓▓▓░░░░░░░░  50%
//! Support   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ 100%
//! ```

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Represents a single bar or segment in a bar chart
#[derive(Debug, Clone)]
pub struct BarData {
    /// Label for this bar (e.g., "Sales", "Q1")
    pub label: String,
    /// Numeric value for this bar
    pub value: f64,
    /// Color for this bar
    pub color: Color,
}

impl BarData {
    /// Creates a new bar with the given label, value, and color
    pub fn new(label: impl Into<String>, value: f64, color: Color) -> Self {
        Self {
            label: label.into(),
            value,
            color,
        }
    }
}

/// Represents a stacked bar with multiple segments
#[derive(Debug, Clone)]
pub struct StackedBarData {
    /// Label for this stacked bar
    pub label: String,
    /// Segments in this stacked bar
    pub segments: Vec<BarSegment>,
}

/// A segment within a stacked bar
#[derive(Debug, Clone)]
pub struct BarSegment {
    /// Value for this segment
    pub value: f64,
    /// Color for this segment
    pub color: Color,
    /// Optional label for this segment
    pub label: Option<String>,
}

impl BarSegment {
    /// Creates a new bar segment
    pub fn new(value: f64, color: Color) -> Self {
        Self {
            value,
            color,
            label: None,
        }
    }

    /// Sets the label for this segment
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

impl StackedBarData {
    /// Creates a new stacked bar with the given label and segments
    pub fn new(label: impl Into<String>, segments: Vec<BarSegment>) -> Self {
        Self {
            label: label.into(),
            segments,
        }
    }

    /// Gets the total value of all segments
    pub fn total_value(&self) -> f64 {
        self.segments.iter().map(|s| s.value).sum()
    }
}

/// Orientation for bar charts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarChartOrientation {
    /// Horizontal bars (left to right)
    Horizontal,
    /// Vertical bars (bottom to top)
    Vertical,
}

/// The main Bar Chart component
pub struct BarChartComponent;

impl BarChartComponent {
    /// Renders a horizontal bar with label and value
    ///
    /// Returns a vector of characters representing the bar
    fn render_horizontal_bar(value: f64, max_value: f64, width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        let percentage = if max_value > 0.0 {
            (value / max_value * 100.0).min(100.0).max(0.0)
        } else {
            0.0
        };

        let filled = ((percentage / 100.0) * width as f64).round() as usize;
        let empty = width.saturating_sub(filled);

        format!("{}{}", "▓".repeat(filled), "░".repeat(empty))
    }

    /// Renders a stacked horizontal bar
    fn render_stacked_horizontal_bar(segments: &[BarSegment], max_value: f64, width: usize) -> Vec<(String, Color)> {
        if width == 0 || max_value == 0.0 {
            return vec![];
        }

        let mut result = Vec::new();

        for segment in segments {
            let percentage = (segment.value / max_value * 100.0).min(100.0).max(0.0);
            let filled = ((percentage / 100.0) * width as f64).round() as usize;

            if filled > 0 {
                result.push(("▓".repeat(filled), segment.color));
            }
        }

        result
    }

    /// Renders vertical bars as a multi-line string
    ///
    /// Returns lines from top to bottom
    fn render_vertical_bars(values: &[f64], max_value: f64, height: usize) -> Vec<String> {
        if height == 0 || values.is_empty() {
            return vec![];
        }

        let mut lines = Vec::new();

        // Render from top to bottom
        for row in (0..height).rev() {
            let threshold = ((row + 1) as f64 / height as f64) * max_value;
            let mut line = String::new();

            for &value in values {
                if value >= threshold {
                    line.push('▓');
                } else {
                    line.push(' ');
                }
                line.push(' '); // Space between bars
            }

            lines.push(line);
        }

        lines
    }

    /// Formats a percentage value for display
    fn format_percentage(value: f64, max_value: f64) -> String {
        if max_value > 0.0 {
            let percentage = (value / max_value * 100.0).min(100.0).max(0.0);
            format!("{:3.0}%", percentage)
        } else {
            "  0%".to_string()
        }
    }

    /// Formats a raw value for display
    fn format_value(value: f64) -> String {
        if value.abs() < 1000.0 {
            format!("{:.1}", value)
        } else {
            format!("{:.0}", value)
        }
    }

    /// Renders a single horizontal bar chart story
    fn render_horizontal_bars_story(
        frame: &mut Frame,
        area: Rect,
        bars: &[BarData],
        show_percentage: bool,
        title: &str,
    ) {
        let max_value = bars.iter().map(|b| b.value).fold(0.0, f64::max);
        let max_label_width = bars.iter().map(|b| b.label.len()).max().unwrap_or(0);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title);

        let inner = block.inner(area);
        let bar_width = inner.width.saturating_sub(max_label_width as u16 + 2 + 6) as usize;

        let mut lines = Vec::new();

        for bar in bars {
            let bar_str = Self::render_horizontal_bar(bar.value, max_value, bar_width);
            let value_str = if show_percentage {
                Self::format_percentage(bar.value, max_value)
            } else {
                Self::format_value(bar.value)
            };

            let line = Line::from(vec![
                Span::raw(format!("{:width$} ", bar.label, width = max_label_width)),
                Span::styled(bar_str, Style::default().fg(bar.color)),
                Span::raw(" "),
                Span::styled(value_str, Style::default().fg(bar.color).add_modifier(Modifier::BOLD)),
            ]);

            lines.push(line);
        }

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Renders a vertical bar chart story
    fn render_vertical_bars_story(
        frame: &mut Frame,
        area: Rect,
        bars: &[BarData],
        title: &str,
    ) {
        let max_value = bars.iter().map(|b| b.value).fold(0.0, f64::max);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title);

        let inner = block.inner(area);
        let chart_height = inner.height.saturating_sub(2) as usize; // Leave space for labels

        let values: Vec<f64> = bars.iter().map(|b| b.value).collect();
        let bar_lines = Self::render_vertical_bars(&values, max_value, chart_height);

        let mut lines: Vec<Line> = bar_lines.iter().map(|l| Line::from(l.clone())).collect();

        // Add a separator line
        lines.push(Line::from("─".repeat(inner.width as usize)));

        // Add labels
        let mut label_line_spans = Vec::new();
        for (i, bar) in bars.iter().enumerate() {
            if i > 0 {
                label_line_spans.push(Span::raw(" "));
            }
            label_line_spans.push(Span::styled(
                format!("{:4}", bar.label),
                Style::default().fg(bar.color),
            ));
        }
        lines.push(Line::from(label_line_spans));

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Renders a stacked bar chart story
    fn render_stacked_bars_story(
        frame: &mut Frame,
        area: Rect,
        stacked_bars: &[StackedBarData],
        title: &str,
    ) {
        let max_value = stacked_bars.iter().map(|b| b.total_value()).fold(0.0, f64::max);
        let max_label_width = stacked_bars.iter().map(|b| b.label.len()).max().unwrap_or(0);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title);

        let inner = block.inner(area);
        let bar_width = inner.width.saturating_sub(max_label_width as u16 + 2) as usize;

        let mut lines = Vec::new();

        for stacked_bar in stacked_bars {
            let segments_rendered = Self::render_stacked_horizontal_bar(
                &stacked_bar.segments,
                max_value,
                bar_width,
            );

            let mut spans = vec![
                Span::raw(format!("{:width$} ", stacked_bar.label, width = max_label_width)),
            ];

            for (bar_str, color) in segments_rendered {
                spans.push(Span::styled(bar_str, Style::default().fg(color)));
            }

            lines.push(Line::from(spans));
        }

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }
}

impl ShowcaseComponent for BarChartComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "BarChart",
            "Display data as ASCII/Unicode bar charts with labels, colors, and multiple orientations",
        )
        .with_category("Display")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            // Story 1: Horizontal Bars
            ShowcaseStory::new(
                "Horizontal Bars",
                "Classic left-to-right bars with labels and percentage values",
                |frame, area| {
                    let bars = vec![
                        BarData::new("Sales", 75.0, Color::Green),
                        BarData::new("Marketing", 50.0, Color::Blue),
                        BarData::new("Support", 100.0, Color::Cyan),
                        BarData::new("Engineering", 90.0, Color::Magenta),
                    ];
                    BarChartComponent::render_horizontal_bars_story(
                        frame, area, &bars, true, "Department Performance",
                    );
                },
            ),
            // Story 2: Vertical Bars
            ShowcaseStory::new(
                "Vertical Bars",
                "Column chart style with bottom-to-top bars",
                |frame, area| {
                    let bars = vec![
                        BarData::new("Q1", 45.0, Color::Red),
                        BarData::new("Q2", 67.0, Color::Yellow),
                        BarData::new("Q3", 89.0, Color::Green),
                        BarData::new("Q4", 92.0, Color::Cyan),
                    ];
                    BarChartComponent::render_vertical_bars_story(
                        frame, area, &bars, "Quarterly Revenue",
                    );
                },
            ),
            // Story 3: Stacked Bars
            ShowcaseStory::new(
                "Stacked Bars",
                "Multiple values stacked in each bar with different colors",
                |frame, area| {
                    let stacked_bars = vec![
                        StackedBarData::new(
                            "Product A",
                            vec![
                                BarSegment::new(30.0, Color::Green),
                                BarSegment::new(20.0, Color::Yellow),
                                BarSegment::new(10.0, Color::Red),
                            ],
                        ),
                        StackedBarData::new(
                            "Product B",
                            vec![
                                BarSegment::new(40.0, Color::Green),
                                BarSegment::new(15.0, Color::Yellow),
                                BarSegment::new(5.0, Color::Red),
                            ],
                        ),
                        StackedBarData::new(
                            "Product C",
                            vec![
                                BarSegment::new(50.0, Color::Green),
                                BarSegment::new(10.0, Color::Yellow),
                                BarSegment::new(15.0, Color::Red),
                            ],
                        ),
                    ];
                    BarChartComponent::render_stacked_bars_story(
                        frame, area, &stacked_bars, "Product Mix Analysis",
                    );
                },
            ),
            // Story 4: Color-Coded
            ShowcaseStory::new(
                "Color-Coded",
                "Different colors per bar based on performance level",
                |frame, area| {
                    let bars = vec![
                        BarData::new("Excellent", 95.0, Color::Green),
                        BarData::new("Good", 75.0, Color::Cyan),
                        BarData::new("Average", 50.0, Color::Yellow),
                        BarData::new("Poor", 25.0, Color::Red),
                        BarData::new("Critical", 10.0, Color::Magenta),
                    ];
                    BarChartComponent::render_horizontal_bars_story(
                        frame, area, &bars, true, "Performance Ratings",
                    );
                },
            ),
            // Story 5: With Labels
            ShowcaseStory::new(
                "With Labels",
                "Category labels and numeric values displayed alongside bars",
                |frame, area| {
                    let bars = vec![
                        BarData::new("CPU Usage", 45.3, Color::Green),
                        BarData::new("Memory", 78.9, Color::Yellow),
                        BarData::new("Disk I/O", 23.1, Color::Cyan),
                        BarData::new("Network", 67.5, Color::Blue),
                    ];
                    BarChartComponent::render_horizontal_bars_story(
                        frame, area, &bars, false, "System Resources",
                    );
                },
            ),
            // Story 6: Percentage View
            ShowcaseStory::new(
                "Percentage View",
                "Bars showing percentages normalized to 100%",
                |frame, area| {
                    let bars = vec![
                        BarData::new("Completed", 234.0, Color::Green),
                        BarData::new("In Progress", 156.0, Color::Yellow),
                        BarData::new("Pending", 89.0, Color::Blue),
                        BarData::new("Blocked", 34.0, Color::Red),
                    ];
                    BarChartComponent::render_horizontal_bars_story(
                        frame, area, &bars, true, "Task Distribution",
                    );
                },
            ),
            // Story 7: Small Values
            ShowcaseStory::new(
                "Edge Cases",
                "Handling small values, zeros, and edge cases gracefully",
                |frame, area| {
                    let bars = vec![
                        BarData::new("Zero", 0.0, Color::Gray),
                        BarData::new("Tiny", 0.5, Color::Blue),
                        BarData::new("Small", 5.0, Color::Cyan),
                        BarData::new("Medium", 50.0, Color::Green),
                        BarData::new("Large", 100.0, Color::Yellow),
                    ];
                    BarChartComponent::render_horizontal_bars_story(
                        frame, area, &bars, true, "Edge Case Handling",
                    );
                },
            ),
        ]
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    // Data Structure Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_bar_data_creation() {
        let bar = BarData::new("Test", 50.0, Color::Green);
        assert_eq!(bar.label, "Test");
        assert_eq!(bar.value, 50.0);
        assert_eq!(bar.color, Color::Green);
    }

    #[test]
    fn test_bar_segment_creation() {
        let segment = BarSegment::new(25.0, Color::Red);
        assert_eq!(segment.value, 25.0);
        assert_eq!(segment.color, Color::Red);
        assert_eq!(segment.label, None);
    }

    #[test]
    fn test_bar_segment_with_label() {
        let segment = BarSegment::new(25.0, Color::Red).with_label("Segment 1");
        assert_eq!(segment.label, Some("Segment 1".to_string()));
    }

    #[test]
    fn test_stacked_bar_data_creation() {
        let segments = vec![
            BarSegment::new(10.0, Color::Green),
            BarSegment::new(20.0, Color::Blue),
        ];
        let stacked_bar = StackedBarData::new("Test", segments);
        assert_eq!(stacked_bar.label, "Test");
        assert_eq!(stacked_bar.segments.len(), 2);
    }

    #[test]
    fn test_stacked_bar_total_value() {
        let segments = vec![
            BarSegment::new(10.0, Color::Green),
            BarSegment::new(20.0, Color::Blue),
            BarSegment::new(15.0, Color::Red),
        ];
        let stacked_bar = StackedBarData::new("Test", segments);
        assert_eq!(stacked_bar.total_value(), 45.0);
    }

    #[test]
    fn test_stacked_bar_total_value_empty() {
        let stacked_bar = StackedBarData::new("Test", vec![]);
        assert_eq!(stacked_bar.total_value(), 0.0);
    }

    // ------------------------------------------------------------------------
    // Horizontal Bar Rendering Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_render_horizontal_bar_full() {
        let bar = BarChartComponent::render_horizontal_bar(100.0, 100.0, 10);
        assert_eq!(bar, "▓▓▓▓▓▓▓▓▓▓");
        assert_eq!(bar.chars().count(), 10);
    }

    #[test]
    fn test_render_horizontal_bar_half() {
        let bar = BarChartComponent::render_horizontal_bar(50.0, 100.0, 10);
        assert_eq!(bar, "▓▓▓▓▓░░░░░");
        assert_eq!(bar.chars().count(), 10);
    }

    #[test]
    fn test_render_horizontal_bar_empty() {
        let bar = BarChartComponent::render_horizontal_bar(0.0, 100.0, 10);
        assert_eq!(bar, "░░░░░░░░░░");
        assert_eq!(bar.chars().count(), 10);
    }

    #[test]
    fn test_render_horizontal_bar_quarter() {
        let bar = BarChartComponent::render_horizontal_bar(25.0, 100.0, 10);
        // Should round to nearest: 25% of 10 = 2.5, rounds to 2 or 3
        assert!(bar.starts_with("▓▓"));
        assert_eq!(bar.chars().count(), 10);
    }

    #[test]
    fn test_render_horizontal_bar_zero_width() {
        let bar = BarChartComponent::render_horizontal_bar(50.0, 100.0, 0);
        assert_eq!(bar, "");
    }

    #[test]
    fn test_render_horizontal_bar_overflow() {
        let bar = BarChartComponent::render_horizontal_bar(150.0, 100.0, 10);
        // Should clamp to 100%
        assert_eq!(bar, "▓▓▓▓▓▓▓▓▓▓");
    }

    #[test]
    fn test_render_horizontal_bar_zero_max() {
        let bar = BarChartComponent::render_horizontal_bar(50.0, 0.0, 10);
        // Should handle zero max gracefully
        assert_eq!(bar, "░░░░░░░░░░");
    }

    #[test]
    fn test_render_horizontal_bar_negative_value() {
        let bar = BarChartComponent::render_horizontal_bar(-10.0, 100.0, 10);
        // Negative should clamp to 0%
        assert_eq!(bar, "░░░░░░░░░░");
    }

    // ------------------------------------------------------------------------
    // Vertical Bar Rendering Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_render_vertical_bars_basic() {
        let values = vec![50.0, 100.0, 75.0];
        let lines = BarChartComponent::render_vertical_bars(&values, 100.0, 4);

        assert_eq!(lines.len(), 4);
        // Top line should only show the 100% bar (middle one)
        assert!(lines[0].contains('▓'));
    }

    #[test]
    fn test_render_vertical_bars_empty_values() {
        let values = vec![];
        let lines = BarChartComponent::render_vertical_bars(&values, 100.0, 4);
        assert_eq!(lines.len(), 0);
    }

    #[test]
    fn test_render_vertical_bars_zero_height() {
        let values = vec![50.0, 100.0];
        let lines = BarChartComponent::render_vertical_bars(&values, 100.0, 0);
        assert_eq!(lines.len(), 0);
    }

    #[test]
    fn test_render_vertical_bars_all_zeros() {
        let values = vec![0.0, 0.0, 0.0];
        let lines = BarChartComponent::render_vertical_bars(&values, 100.0, 4);

        assert_eq!(lines.len(), 4);
        // All lines should be empty (spaces)
        for line in lines {
            assert!(!line.contains('▓'));
        }
    }

    // ------------------------------------------------------------------------
    // Stacked Bar Rendering Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_render_stacked_horizontal_bar_basic() {
        let segments = vec![
            BarSegment::new(30.0, Color::Green),
            BarSegment::new(20.0, Color::Yellow),
        ];
        let result = BarChartComponent::render_stacked_horizontal_bar(&segments, 100.0, 10);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].1, Color::Green);
        assert_eq!(result[1].1, Color::Yellow);
    }

    #[test]
    fn test_render_stacked_horizontal_bar_empty() {
        let segments = vec![];
        let result = BarChartComponent::render_stacked_horizontal_bar(&segments, 100.0, 10);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_render_stacked_horizontal_bar_zero_width() {
        let segments = vec![BarSegment::new(50.0, Color::Green)];
        let result = BarChartComponent::render_stacked_horizontal_bar(&segments, 100.0, 0);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_render_stacked_horizontal_bar_zero_max() {
        let segments = vec![BarSegment::new(50.0, Color::Green)];
        let result = BarChartComponent::render_stacked_horizontal_bar(&segments, 0.0, 10);
        assert_eq!(result.len(), 0);
    }

    // ------------------------------------------------------------------------
    // Formatting Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_format_percentage_full() {
        let formatted = BarChartComponent::format_percentage(100.0, 100.0);
        assert_eq!(formatted, "100%");
    }

    #[test]
    fn test_format_percentage_half() {
        let formatted = BarChartComponent::format_percentage(50.0, 100.0);
        assert_eq!(formatted, " 50%");
    }

    #[test]
    fn test_format_percentage_zero() {
        let formatted = BarChartComponent::format_percentage(0.0, 100.0);
        assert_eq!(formatted, "  0%");
    }

    #[test]
    fn test_format_percentage_overflow() {
        let formatted = BarChartComponent::format_percentage(150.0, 100.0);
        assert_eq!(formatted, "100%");
    }

    #[test]
    fn test_format_percentage_zero_max() {
        let formatted = BarChartComponent::format_percentage(50.0, 0.0);
        assert_eq!(formatted, "  0%");
    }

    #[test]
    fn test_format_value_small() {
        let formatted = BarChartComponent::format_value(45.3);
        assert_eq!(formatted, "45.3");
    }

    #[test]
    fn test_format_value_large() {
        let formatted = BarChartComponent::format_value(1234.5);
        assert_eq!(formatted, "1234");
    }

    #[test]
    fn test_format_value_zero() {
        let formatted = BarChartComponent::format_value(0.0);
        assert_eq!(formatted, "0.0");
    }

    // ------------------------------------------------------------------------
    // Component Metadata Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_component_metadata() {
        let component = BarChartComponent;
        let metadata = component.metadata();
        assert_eq!(metadata.name, "BarChart");
        assert_eq!(metadata.category, "Display");
        assert!(!metadata.description.is_empty());
    }

    #[test]
    fn test_component_has_seven_stories() {
        let component = BarChartComponent;
        let stories = component.stories();
        assert_eq!(stories.len(), 7);
    }

    #[test]
    fn test_story_names() {
        let component = BarChartComponent;
        let stories = component.stories();
        let expected_names = vec![
            "Horizontal Bars",
            "Vertical Bars",
            "Stacked Bars",
            "Color-Coded",
            "With Labels",
            "Percentage View",
            "Edge Cases",
        ];

        for (story, expected_name) in stories.iter().zip(expected_names.iter()) {
            assert_eq!(story.name, *expected_name);
        }
    }

    #[test]
    fn test_all_stories_have_descriptions() {
        let component = BarChartComponent;
        let stories = component.stories();

        for story in stories {
            assert!(!story.description.is_empty(), "Story '{}' has no description", story.name);
        }
    }
}
