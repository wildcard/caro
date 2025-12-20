//! Output summarization for large command outputs

/// Summarizes large command outputs to reduce storage size
pub struct OutputSummarizer {
    /// Maximum lines to include in head section
    head_lines: usize,
    /// Maximum lines to include in tail section
    tail_lines: usize,
    /// Maximum length per line before truncation
    max_line_length: usize,
}

impl OutputSummarizer {
    /// Create a new OutputSummarizer with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a custom summarizer
    pub fn with_config(head_lines: usize, tail_lines: usize, max_line_length: usize) -> Self {
        Self {
            head_lines,
            tail_lines,
            max_line_length,
        }
    }

    /// Summarize the output, keeping head and tail sections
    pub fn summarize(&self, output: &str) -> String {
        let lines: Vec<&str> = output.lines().collect();
        let total_lines = lines.len();

        if total_lines <= self.head_lines + self.tail_lines {
            // Output is small enough, just truncate long lines
            return self.truncate_lines(&lines);
        }

        let mut summary = String::new();

        // Add head section
        summary.push_str("=== HEAD ===\n");
        for line in lines.iter().take(self.head_lines) {
            summary.push_str(&self.truncate_line(line));
            summary.push('\n');
        }

        // Add summary of omitted lines
        let omitted = total_lines - self.head_lines - self.tail_lines;
        summary.push_str(&format!("\n... ({} lines omitted) ...\n\n", omitted));

        // Add tail section
        summary.push_str("=== TAIL ===\n");
        for line in lines.iter().skip(total_lines - self.tail_lines) {
            summary.push_str(&self.truncate_line(line));
            summary.push('\n');
        }

        // Add metadata
        summary.push_str(&format!(
            "\n[Summary: {} total lines, {} bytes, showing first {} and last {} lines]",
            total_lines,
            output.len(),
            self.head_lines,
            self.tail_lines
        ));

        summary
    }

    /// Summarize output with additional statistical information
    pub fn summarize_with_stats(&self, output: &str) -> SummarizedOutput {
        let lines: Vec<&str> = output.lines().collect();
        let total_lines = lines.len();
        let total_bytes = output.len();

        // Count error patterns
        let error_count = lines
            .iter()
            .filter(|l| {
                let lower = l.to_lowercase();
                lower.contains("error") || lower.contains("failed") || lower.contains("exception")
            })
            .count();

        let warning_count = lines
            .iter()
            .filter(|l| {
                let lower = l.to_lowercase();
                lower.contains("warning") || lower.contains("warn")
            })
            .count();

        let summary_text = self.summarize(output);

        SummarizedOutput {
            summary: summary_text,
            total_lines,
            total_bytes,
            error_count,
            warning_count,
            truncated: total_lines > self.head_lines + self.tail_lines,
        }
    }

    /// Truncate a single line if it exceeds max length
    fn truncate_line(&self, line: &str) -> String {
        if line.len() <= self.max_line_length {
            line.to_string()
        } else {
            format!(
                "{}... [truncated {} chars]",
                &line[..self.max_line_length],
                line.len() - self.max_line_length
            )
        }
    }

    /// Truncate all lines in a collection
    fn truncate_lines(&self, lines: &[&str]) -> String {
        lines
            .iter()
            .map(|l| self.truncate_line(l))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for OutputSummarizer {
    fn default() -> Self {
        Self {
            head_lines: 50,
            tail_lines: 50,
            max_line_length: 500,
        }
    }
}

/// Result of summarization with statistics
#[derive(Debug, Clone)]
pub struct SummarizedOutput {
    /// The summarized text
    pub summary: String,
    /// Total number of lines in original output
    pub total_lines: usize,
    /// Total bytes in original output
    pub total_bytes: usize,
    /// Number of lines containing error patterns
    pub error_count: usize,
    /// Number of lines containing warning patterns
    pub warning_count: usize,
    /// Whether the output was truncated
    pub truncated: bool,
}

impl SummarizedOutput {
    /// Check if the output contained errors
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Check if the output contained warnings
    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_small_output_not_truncated() {
        let summarizer = OutputSummarizer::with_config(5, 5, 100);
        let output = "line 1\nline 2\nline 3";

        let summary = summarizer.summarize(output);
        assert!(summary.contains("line 1"));
        assert!(summary.contains("line 2"));
        assert!(summary.contains("line 3"));
        assert!(!summary.contains("omitted"));
    }

    #[test]
    fn test_large_output_truncated() {
        let summarizer = OutputSummarizer::with_config(2, 2, 100);
        let lines: Vec<String> = (0..100).map(|i| format!("line {}", i)).collect();
        let output = lines.join("\n");

        let summary = summarizer.summarize(&output);
        assert!(summary.contains("line 0"));
        assert!(summary.contains("line 1"));
        assert!(summary.contains("line 98"));
        assert!(summary.contains("line 99"));
        assert!(summary.contains("omitted"));
    }

    #[test]
    fn test_long_line_truncation() {
        let summarizer = OutputSummarizer::with_config(5, 5, 20);
        let output = "this is a very long line that should be truncated because it exceeds the max length";

        let summary = summarizer.summarize(output);
        assert!(summary.contains("truncated"));
    }

    #[test]
    fn test_summarize_with_stats() {
        let summarizer = OutputSummarizer::with_config(5, 5, 100);
        let output = "success\nERROR: something failed\nwarning: deprecated\nok";

        let result = summarizer.summarize_with_stats(output);
        assert_eq!(result.total_lines, 4);
        assert_eq!(result.error_count, 1);
        assert_eq!(result.warning_count, 1);
        assert!(result.has_errors());
        assert!(result.has_warnings());
    }

    #[test]
    fn test_default_summarizer() {
        let summarizer = OutputSummarizer::default();
        assert_eq!(summarizer.head_lines, 50);
        assert_eq!(summarizer.tail_lines, 50);
        assert_eq!(summarizer.max_line_length, 500);
    }
}
