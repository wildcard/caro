//! Metric Dashboard Component
//!
//! Displays multiple system metrics in a compact dashboard layout with visual indicators,
//! sparklines, and color-coded thresholds. Ideal for monitoring system resources like
//! CPU usage, memory, disk space, and network I/O.
//!
//! ## Features
//! - Grid-based layout (2x2, 3x2) responsive to terminal size
//! - Color-coded metric values based on thresholds (Green/Yellow/Red)
//! - ASCII sparklines for visualizing trends
//! - Support for various units (%, GB, MB/s)
//! - Critical alerts and warnings
//! - Historical comparison with change indicators (↑/↓)
//!
//! ## Example
//! ```text
//! ┌─────────────────────────────────────┐
//! │ CPU Usage          Memory Usage     │
//! │ 45%  ▓▓▓▓▓░░░░░   78%  ▓▓▓▓▓▓▓▓░░ │
//! │ Normal             High             │
//! ├─────────────────────────────────────┤
//! │ Disk Space         Network I/O      │
//! │ 234 GB / 512 GB    ↑ 2.3 MB/s      │
//! │ 46%                ↓ 1.1 MB/s      │
//! └─────────────────────────────────────┘
//! ```

use crate::tui::showcase::{ComponentMetadata, ShowcaseComponent, ShowcaseStory};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Represents a single metric with value, unit, and status
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub max_value: Option<f64>,
    pub unit: String,
    pub status: MetricStatus,
    pub sparkline: Option<Vec<f64>>,
    pub change: Option<f64>,
}

/// Status level for a metric based on thresholds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricStatus {
    Healthy,
    Warning,
    Critical,
}

impl Metric {
    /// Creates a new metric with the given parameters
    pub fn new(name: impl Into<String>, value: f64, unit: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value,
            max_value: None,
            unit: unit.into(),
            status: MetricStatus::Healthy,
            sparkline: None,
            change: None,
        }
    }

    /// Sets the maximum value for percentage calculation
    pub fn with_max(mut self, max: f64) -> Self {
        self.max_value = Some(max);
        self
    }

    /// Sets the status of the metric
    pub fn with_status(mut self, status: MetricStatus) -> Self {
        self.status = status;
        self
    }

    /// Sets sparkline data for the metric
    pub fn with_sparkline(mut self, data: Vec<f64>) -> Self {
        self.sparkline = Some(data);
        self
    }

    /// Sets change indicator (positive for increase, negative for decrease)
    pub fn with_change(mut self, change: f64) -> Self {
        self.change = Some(change);
        self
    }

    /// Gets the color for this metric based on its status
    fn status_color(&self) -> Color {
        match self.status {
            MetricStatus::Healthy => Color::Green,
            MetricStatus::Warning => Color::Yellow,
            MetricStatus::Critical => Color::Red,
        }
    }

    /// Gets the status text for display
    fn status_text(&self) -> &'static str {
        match self.status {
            MetricStatus::Healthy => "Normal",
            MetricStatus::Warning => "High",
            MetricStatus::Critical => "Critical",
        }
    }

    /// Generates a simple progress bar for the metric
    fn progress_bar(&self, width: usize) -> String {
        let percentage = if let Some(max) = self.max_value {
            (self.value / max * 100.0).min(100.0)
        } else {
            self.value.min(100.0)
        };

        let filled = ((percentage / 100.0) * width as f64) as usize;
        let empty = width.saturating_sub(filled);

        format!("{}{}", "▓".repeat(filled), "░".repeat(empty))
    }

    /// Generates ASCII sparkline from data
    fn render_sparkline(&self) -> String {
        if let Some(ref data) = self.sparkline {
            if data.is_empty() {
                return String::new();
            }

            let max = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let min = data.iter().cloned().fold(f64::INFINITY, f64::min);
            let range = max - min;

            if range == 0.0 {
                return "▄".repeat(data.len());
            }

            data.iter()
                .map(|&v| {
                    let normalized = (v - min) / range;
                    if normalized > 0.75 {
                        '█'
                    } else if normalized > 0.5 {
                        '▓'
                    } else if normalized > 0.25 {
                        '▒'
                    } else {
                        '░'
                    }
                })
                .collect()
        } else {
            String::new()
        }
    }

    /// Formats the value with appropriate precision and units
    fn format_value(&self) -> String {
        if let Some(max) = self.max_value {
            format!("{:.1} {} / {:.1} {}", self.value, self.unit, max, self.unit)
        } else {
            format!("{:.1}{}", self.value, self.unit)
        }
    }

    /// Formats the change indicator
    fn format_change(&self) -> Option<String> {
        self.change.map(|c| {
            if c > 0.0 {
                format!("↑ +{:.1}%", c)
            } else if c < 0.0 {
                format!("↓ {:.1}%", c)
            } else {
                "→ 0%".to_string()
            }
        })
    }
}

/// The main Metric Dashboard component
pub struct MetricDashboardComponent;

impl MetricDashboardComponent {
    /// Renders a single metric card in the given area
    fn render_metric_card(
        frame: &mut Frame,
        area: Rect,
        metric: &Metric,
        show_sparkline: bool,
        show_progress: bool,
    ) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(metric.name.clone())
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = block.inner(area);

        // Create content lines
        let mut lines = vec![];

        // Line 1: Value with color
        let value_line = Line::from(vec![Span::styled(
            metric.format_value(),
            Style::default()
                .fg(metric.status_color())
                .add_modifier(Modifier::BOLD),
        )]);
        lines.push(value_line);

        // Line 2: Progress bar if enabled
        if show_progress {
            let bar = metric.progress_bar(inner.width.saturating_sub(2) as usize);
            lines.push(Line::from(bar));
        }

        // Line 3: Sparkline if enabled
        if show_sparkline {
            let sparkline = metric.render_sparkline();
            if !sparkline.is_empty() {
                lines.push(Line::from(sparkline));
            }
        }

        // Line 4: Status
        lines.push(Line::from(vec![Span::styled(
            metric.status_text(),
            Style::default().fg(metric.status_color()),
        )]));

        // Line 5: Change indicator if available
        if let Some(change_text) = metric.format_change() {
            let change_color = if metric.change.unwrap_or(0.0) > 0.0 {
                Color::Red
            } else if metric.change.unwrap_or(0.0) < 0.0 {
                Color::Green
            } else {
                Color::Gray
            };
            lines.push(Line::from(vec![Span::styled(
                change_text,
                Style::default().fg(change_color),
            )]));
        }

        let paragraph = Paragraph::new(lines).block(block);
        frame.render_widget(paragraph, area);
    }

    /// Renders a grid of metrics (2x2 or 3x2)
    fn render_metric_grid(
        frame: &mut Frame,
        area: Rect,
        metrics: &[Metric],
        show_sparkline: bool,
        show_progress: bool,
    ) {
        // Determine grid layout based on number of metrics
        let (rows, cols) = match metrics.len() {
            0..=2 => (1, metrics.len()),
            3..=4 => (2, 2),
            5..=6 => (2, 3),
            _ => (3, 3),
        };

        // Create vertical layout for rows
        let row_constraints: Vec<Constraint> = (0..rows)
            .map(|_| Constraint::Percentage(100 / rows as u16))
            .collect();

        let row_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(area);

        // Render each row
        for (row_idx, row_chunk) in row_chunks.iter().enumerate() {
            // Create horizontal layout for columns
            let col_constraints: Vec<Constraint> = (0..cols)
                .map(|_| Constraint::Percentage(100 / cols as u16))
                .collect();

            let col_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints)
                .split(*row_chunk);

            // Render each column
            for (col_idx, col_chunk) in col_chunks.iter().enumerate() {
                let metric_idx = row_idx * cols + col_idx;
                if metric_idx < metrics.len() {
                    Self::render_metric_card(
                        frame,
                        *col_chunk,
                        &metrics[metric_idx],
                        show_sparkline,
                        show_progress,
                    );
                }
            }
        }
    }
}

impl ShowcaseComponent for MetricDashboardComponent {
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::new(
            "MetricDashboard",
            "System monitoring dashboard with metrics, sparklines, and color-coded alerts",
        )
        .with_category("Display")
        .with_version("1.0.0")
    }

    fn stories(&self) -> Vec<ShowcaseStory> {
        vec![
            // Story 1: Basic Metrics
            ShowcaseStory::new(
                "Basic Metrics",
                "Simple dashboard with 4 key metrics (CPU, Memory, Disk, Network)",
                |frame, area| {
                    let metrics = vec![
                        Metric::new("CPU Usage", 45.0, "%").with_status(MetricStatus::Healthy),
                        Metric::new("Memory Usage", 78.0, "%").with_status(MetricStatus::Warning),
                        Metric::new("Disk Space", 234.0, "GB")
                            .with_max(512.0)
                            .with_status(MetricStatus::Healthy),
                        Metric::new("Network I/O", 2.3, "MB/s").with_status(MetricStatus::Healthy),
                    ];
                    MetricDashboardComponent::render_metric_grid(
                        frame, area, &metrics, false, false,
                    );
                },
            ),
            // Story 2: With Sparklines
            ShowcaseStory::new(
                "With Sparklines",
                "Metrics with mini ASCII graphs showing recent trends",
                |frame, area| {
                    let metrics = vec![
                        Metric::new("CPU Usage", 45.0, "%")
                            .with_status(MetricStatus::Healthy)
                            .with_sparkline(vec![30.0, 35.0, 40.0, 42.0, 45.0, 43.0, 45.0]),
                        Metric::new("Memory Usage", 78.0, "%")
                            .with_status(MetricStatus::Warning)
                            .with_sparkline(vec![65.0, 68.0, 72.0, 75.0, 76.0, 77.0, 78.0]),
                        Metric::new("Disk Space", 234.0, "GB")
                            .with_max(512.0)
                            .with_status(MetricStatus::Healthy)
                            .with_sparkline(vec![220.0, 225.0, 228.0, 230.0, 232.0, 233.0, 234.0]),
                        Metric::new("Network I/O", 2.3, "MB/s")
                            .with_status(MetricStatus::Healthy)
                            .with_sparkline(vec![1.2, 1.8, 2.0, 2.5, 2.3, 2.1, 2.3]),
                    ];
                    MetricDashboardComponent::render_metric_grid(
                        frame, area, &metrics, true, false,
                    );
                },
            ),
            // Story 3: Color-Coded
            ShowcaseStory::new(
                "Color-Coded",
                "Metrics with red/yellow/green thresholds for quick status assessment",
                |frame, area| {
                    let metrics = vec![
                        Metric::new("CPU Usage", 45.0, "%").with_status(MetricStatus::Healthy),
                        Metric::new("Memory Usage", 78.0, "%").with_status(MetricStatus::Warning),
                        Metric::new("Disk Space", 92.0, "%").with_status(MetricStatus::Critical),
                        Metric::new("Network I/O", 2.3, "MB/s").with_status(MetricStatus::Healthy),
                    ];
                    MetricDashboardComponent::render_metric_grid(
                        frame, area, &metrics, false, true,
                    );
                },
            ),
            // Story 4: Compact Layout
            ShowcaseStory::new(
                "Compact Layout",
                "Dense 2x2 grid layout for space-constrained displays",
                |frame, area| {
                    let metrics = vec![
                        Metric::new("CPU", 45.0, "%"),
                        Metric::new("RAM", 78.0, "%"),
                        Metric::new("Disk", 46.0, "%"),
                        Metric::new("Net", 2.3, "MB/s"),
                    ];
                    MetricDashboardComponent::render_metric_grid(
                        frame, area, &metrics, false, false,
                    );
                },
            ),
            // Story 5: With Units
            ShowcaseStory::new(
                "With Units",
                "Metrics showing various units: percentages, bytes, MB/s, etc.",
                |frame, area| {
                    let metrics = vec![
                        Metric::new("CPU Usage", 45.2, "%").with_status(MetricStatus::Healthy),
                        Metric::new("Memory", 6.2, "GB")
                            .with_max(8.0)
                            .with_status(MetricStatus::Warning),
                        Metric::new("Disk Free", 234.5, "GB")
                            .with_max(512.0)
                            .with_status(MetricStatus::Healthy),
                        Metric::new("Upload", 2.3, "MB/s").with_status(MetricStatus::Healthy),
                    ];
                    MetricDashboardComponent::render_metric_grid(
                        frame, area, &metrics, false, true,
                    );
                },
            ),
            // Story 6: Critical Alert
            ShowcaseStory::new(
                "Critical Alert",
                "Dashboard with one or more metrics in critical state",
                |frame, area| {
                    let metrics = vec![
                        Metric::new("CPU Usage", 95.0, "%").with_status(MetricStatus::Critical),
                        Metric::new("Memory Usage", 97.0, "%").with_status(MetricStatus::Critical),
                        Metric::new("Disk Space", 98.0, "%").with_status(MetricStatus::Critical),
                        Metric::new("Network I/O", 0.1, "MB/s").with_status(MetricStatus::Healthy),
                    ];
                    MetricDashboardComponent::render_metric_grid(
                        frame, area, &metrics, false, true,
                    );
                },
            ),
            // Story 7: Historical Comparison
            ShowcaseStory::new(
                "Historical Comparison",
                "Metrics with change indicators showing increase/decrease over time",
                |frame, area| {
                    let metrics = vec![
                        Metric::new("CPU Usage", 45.0, "%")
                            .with_status(MetricStatus::Healthy)
                            .with_change(5.2),
                        Metric::new("Memory Usage", 78.0, "%")
                            .with_status(MetricStatus::Warning)
                            .with_change(-2.1),
                        Metric::new("Disk Space", 234.0, "GB")
                            .with_max(512.0)
                            .with_status(MetricStatus::Healthy)
                            .with_change(0.5),
                        Metric::new("Network I/O", 2.3, "MB/s")
                            .with_status(MetricStatus::Healthy)
                            .with_change(-15.3),
                    ];
                    MetricDashboardComponent::render_metric_grid(
                        frame, area, &metrics, false, false,
                    );
                },
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric_creation() {
        let metric = Metric::new("CPU", 50.0, "%");
        assert_eq!(metric.name, "CPU");
        assert_eq!(metric.value, 50.0);
        assert_eq!(metric.unit, "%");
        assert_eq!(metric.status, MetricStatus::Healthy);
    }

    #[test]
    fn test_metric_with_max_value() {
        let metric = Metric::new("Memory", 4.0, "GB").with_max(8.0);
        assert_eq!(metric.max_value, Some(8.0));
        assert_eq!(metric.format_value(), "4.0 GB / 8.0 GB");
    }

    #[test]
    fn test_metric_status_color() {
        let healthy = Metric::new("Test", 50.0, "%").with_status(MetricStatus::Healthy);
        let warning = Metric::new("Test", 50.0, "%").with_status(MetricStatus::Warning);
        let critical = Metric::new("Test", 50.0, "%").with_status(MetricStatus::Critical);

        assert_eq!(healthy.status_color(), Color::Green);
        assert_eq!(warning.status_color(), Color::Yellow);
        assert_eq!(critical.status_color(), Color::Red);
    }

    #[test]
    fn test_metric_status_text() {
        let healthy = Metric::new("Test", 50.0, "%").with_status(MetricStatus::Healthy);
        let warning = Metric::new("Test", 50.0, "%").with_status(MetricStatus::Warning);
        let critical = Metric::new("Test", 50.0, "%").with_status(MetricStatus::Critical);

        assert_eq!(healthy.status_text(), "Normal");
        assert_eq!(warning.status_text(), "High");
        assert_eq!(critical.status_text(), "Critical");
    }

    #[test]
    fn test_progress_bar_generation() {
        let metric = Metric::new("Test", 50.0, "%");
        let bar = metric.progress_bar(10);
        assert_eq!(bar.chars().count(), 10);
        assert!(bar.contains('▓'));
        assert!(bar.contains('░'));
    }

    #[test]
    fn test_progress_bar_with_max_value() {
        let metric = Metric::new("Test", 4.0, "GB").with_max(8.0);
        let bar = metric.progress_bar(10);
        // 4/8 = 50% should give us 5 filled and 5 empty
        assert_eq!(bar, "▓▓▓▓▓░░░░░");
    }

    #[test]
    fn test_progress_bar_full() {
        let metric = Metric::new("Test", 100.0, "%");
        let bar = metric.progress_bar(10);
        assert_eq!(bar, "▓▓▓▓▓▓▓▓▓▓");
    }

    #[test]
    fn test_progress_bar_empty() {
        let metric = Metric::new("Test", 0.0, "%");
        let bar = metric.progress_bar(10);
        assert_eq!(bar, "░░░░░░░░░░");
    }

    #[test]
    fn test_sparkline_rendering() {
        let metric =
            Metric::new("Test", 50.0, "%").with_sparkline(vec![10.0, 30.0, 50.0, 70.0, 90.0]);
        let sparkline = metric.render_sparkline();
        assert_eq!(sparkline.chars().count(), 5);
    }

    #[test]
    fn test_sparkline_empty() {
        let metric = Metric::new("Test", 50.0, "%").with_sparkline(vec![]);
        let sparkline = metric.render_sparkline();
        assert!(sparkline.is_empty());
    }

    #[test]
    fn test_sparkline_constant_values() {
        let metric = Metric::new("Test", 50.0, "%").with_sparkline(vec![50.0, 50.0, 50.0]);
        let sparkline = metric.render_sparkline();
        // All values the same should produce same character
        assert_eq!(sparkline, "▄▄▄");
    }

    #[test]
    fn test_change_indicator_positive() {
        let metric = Metric::new("Test", 50.0, "%").with_change(5.2);
        let change = metric.format_change();
        assert_eq!(change, Some("↑ +5.2%".to_string()));
    }

    #[test]
    fn test_change_indicator_negative() {
        let metric = Metric::new("Test", 50.0, "%").with_change(-3.1);
        let change = metric.format_change();
        assert_eq!(change, Some("↓ -3.1%".to_string()));
    }

    #[test]
    fn test_change_indicator_zero() {
        let metric = Metric::new("Test", 50.0, "%").with_change(0.0);
        let change = metric.format_change();
        assert_eq!(change, Some("→ 0%".to_string()));
    }

    #[test]
    fn test_change_indicator_none() {
        let metric = Metric::new("Test", 50.0, "%");
        let change = metric.format_change();
        assert_eq!(change, None);
    }

    #[test]
    fn test_component_metadata() {
        let component = MetricDashboardComponent;
        let metadata = component.metadata();
        assert_eq!(metadata.name, "MetricDashboard");
        assert_eq!(metadata.category, "Display");
        assert!(!metadata.description.is_empty());
    }

    #[test]
    fn test_component_has_seven_stories() {
        let component = MetricDashboardComponent;
        let stories = component.stories();
        assert_eq!(stories.len(), 7);
    }

    #[test]
    fn test_story_names() {
        let component = MetricDashboardComponent;
        let stories = component.stories();
        let expected_names = vec![
            "Basic Metrics",
            "With Sparklines",
            "Color-Coded",
            "Compact Layout",
            "With Units",
            "Critical Alert",
            "Historical Comparison",
        ];

        for (story, expected_name) in stories.iter().zip(expected_names.iter()) {
            assert_eq!(story.name, *expected_name);
        }
    }

    #[test]
    fn test_value_formatting_without_max() {
        let metric = Metric::new("CPU", 45.5, "%");
        assert_eq!(metric.format_value(), "45.5%");
    }

    #[test]
    fn test_value_formatting_with_max() {
        let metric = Metric::new("Memory", 4.2, "GB").with_max(8.0);
        assert_eq!(metric.format_value(), "4.2 GB / 8.0 GB");
    }

    #[test]
    fn test_progress_bar_overflow_protection() {
        let metric = Metric::new("Test", 150.0, "%");
        let bar = metric.progress_bar(10);
        // Should clamp to 100%
        assert_eq!(bar, "▓▓▓▓▓▓▓▓▓▓");
    }

    #[test]
    fn test_sparkline_gradient_characters() {
        let metric =
            Metric::new("Test", 50.0, "%").with_sparkline(vec![0.0, 25.0, 50.0, 75.0, 100.0]);
        let sparkline = metric.render_sparkline();
        // Should use different characters for different ranges
        assert!(
            sparkline.contains('░')
                || sparkline.contains('▒')
                || sparkline.contains('▓')
                || sparkline.contains('█')
        );
    }
}
