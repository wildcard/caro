// JUnit XML report generation

pub struct JUnitReporter;

impl JUnitReporter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JUnitReporter {
    fn default() -> Self {
        Self::new()
    }
}
