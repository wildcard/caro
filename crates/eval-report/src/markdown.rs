// Markdown report generation

pub struct MarkdownReporter;

impl MarkdownReporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownReporter {
    fn default() -> Self {
        Self::new()
    }
}
