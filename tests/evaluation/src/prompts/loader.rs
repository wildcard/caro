//! Template-based prompt loading with variable substitution.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Prompt loader with template support
pub struct PromptLoader {
    prompts_dir: PathBuf,
}

impl PromptLoader {
    /// Create a new prompt loader
    pub fn new<P: AsRef<Path>>(prompts_dir: P) -> Self {
        Self {
            prompts_dir: prompts_dir.as_ref().to_path_buf(),
        }
    }

    /// Render a template with variable substitution
    ///
    /// Supports {{variable_name}} syntax for simple variable substitution.
    pub fn render_template(&self, template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Load and render a prompt template
    pub fn load_and_render(
        &self,
        name: &str,
        version: &str,
        variables: &HashMap<String, String>,
    ) -> Result<String, std::io::Error> {
        let prompt_path = self.prompts_dir.join(version).join(format!("{}.md", name));

        let template = std::fs::read_to_string(prompt_path)?;
        Ok(self.render_template(&template, variables))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_template_rendering() {
        let loader = PromptLoader::new("prompts");
        let template = "Hello {{name}}, you are {{age}} years old.";

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("age".to_string(), "30".to_string());

        let result = loader.render_template(template, &vars);
        assert_eq!(result, "Hello Alice, you are 30 years old.");
    }

    #[test]
    fn test_multiple_variable_occurrences() {
        let loader = PromptLoader::new("prompts");
        let template = "{{name}} is {{age}}. Did you know {{name}} is {{age}}?";

        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Bob".to_string());
        vars.insert("age".to_string(), "25".to_string());

        let result = loader.render_template(template, &vars);
        assert_eq!(result, "Bob is 25. Did you know Bob is 25?");
    }
}
