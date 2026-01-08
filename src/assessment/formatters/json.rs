use crate::assessment::AssessmentResult;

/// Format assessment result as pretty-printed JSON
pub fn format(result: &AssessmentResult) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(result)
}
