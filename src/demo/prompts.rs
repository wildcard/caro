// Demo-specific system prompts for showcasing capabilities

use crate::models::ShellType;

/// Get the demo mode system prompt
///
/// This prompt is designed to showcase cmdai's capabilities by:
/// - Generating multiple high-quality alternatives
/// - Providing detailed explanations
/// - Being critical of inappropriate or off-topic requests
/// - Suggesting follow-up commands
/// - Highlighting tool capabilities
pub fn get_demo_system_prompt(shell: ShellType) -> String {
    format!(
        r#"You are cmdai, an expert shell command generator in DEMO MODE.

Your purpose is to showcase the BEST capabilities of this tool to convert natural language into safe, efficient POSIX shell commands for {shell}.

DEMO MODE BEHAVIOR:
1. Generate 3-5 high-quality command alternatives that showcase different approaches
2. Provide detailed, educational explanations that highlight the tool's understanding
3. Be CRITICAL and JUDGMENTAL of requests that:
   - Are off-topic or inappropriate for a shell command tool
   - Try to joke around or test the system with nonsense
   - Request impossible or unrealistic operations
   - Fall outside the scope of shell command generation

4. For inappropriate requests, respond with SASSY, SNARKY commentary like:
   - "Really? You want me to generate a command for THAT? Let me suggest something actually useful instead..."
   - "I'm a professional shell command generator, not a toy. Here's what you SHOULD be asking..."
   - "That's cute, but how about we focus on actual shell operations? Try this instead..."

5. ALWAYS redirect poor requests to showcase actual capabilities:
   - Suggest 2-3 impressive commands they could try instead
   - Highlight specific features (safety validation, multi-shell support, intelligent parsing)
   - Guide them toward becoming a contributor by showing the tool's potential

6. For good requests, be ENTHUSIASTIC and EDUCATIONAL:
   - Explain WHY each command is the best approach
   - Show advanced techniques and best practices
   - Demonstrate the tool's ability to handle edge cases
   - Suggest powerful follow-up commands

OUTPUT REQUIREMENTS:
- Multiple alternatives showing different techniques
- Rich explanations with rationale
- Suggestions for next steps or related commands
- Highlight safety considerations and best practices
- Use this JSON format:
{{
  "command": "primary_command_here",
  "explanation": "detailed explanation showcasing understanding",
  "alternatives": [
    "alternative_1_with_different_approach",
    "alternative_2_showing_advanced_technique",
    "alternative_3_demonstrating_edge_case_handling"
  ],
  "suggestions": [
    "follow_up_command_1",
    "related_capability_2"
  ],
  "critique": "optional sassy commentary if request was inappropriate"
}}

SHELL: {shell}
SAFETY: Always prioritize safe, POSIX-compliant commands
GOAL: Convert viewers into contributors by showing amazing capabilities!

Remember: You're showcasing the BEST of what this tool can do!"#,
        shell = shell
    )
}

/// Get critique messages for different types of inappropriate requests
pub fn get_critique_for_request(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();

    // Detect joke/test requests
    if input_lower.contains("hello")
        || input_lower.contains("hi there")
        || input_lower.contains("test")
        || input_lower.contains("joke")
    {
        return Some(
            "Cute greeting, but I'm a shell command generator, not a chatbot. \
             Let me show you something actually impressive instead..."
                .to_string(),
        );
    }

    // Detect impossible/nonsense requests
    if input_lower.contains("make me coffee")
        || input_lower.contains("order pizza")
        || input_lower.contains("do my homework")
    {
        return Some(
            "I generate SHELL COMMANDS, not magic spells. \
             Here's what I can actually do for you..."
                .to_string(),
        );
    }

    // Detect overly vague requests
    if input_lower.split_whitespace().count() <= 2
        && !input_lower.contains("ls")
        && !input_lower.contains("pwd")
        && !input_lower.contains("cd")
    {
        return Some(
            "That's a bit vague. Let me show you how specific, powerful commands look..."
                .to_string(),
        );
    }

    // Detect Windows-specific on Unix shell
    if input_lower.contains("powershell")
        || input_lower.contains("cmd.exe")
        || input_lower.contains("windows")
    {
        return Some(
            "This is a POSIX-focused tool. Let me demonstrate proper Unix/Linux commands instead..."
                .to_string(),
        );
    }

    None
}

/// Get showcase command suggestions based on the request context
pub fn get_showcase_suggestions(input: &str) -> Vec<String> {
    let input_lower = input.to_lowercase();
    let mut suggestions = Vec::new();

    // File operations
    if input_lower.contains("file") || input_lower.contains("find") {
        suggestions.push("Try: 'find all PDF files larger than 10MB'".to_string());
        suggestions.push("Try: 'recursively search for TODO comments in source code'".to_string());
    }

    // System operations
    if input_lower.contains("system") || input_lower.contains("process") {
        suggestions.push("Try: 'show top 5 memory-consuming processes'".to_string());
        suggestions.push("Try: 'monitor CPU usage in real-time'".to_string());
    }

    // Network operations
    if input_lower.contains("network") || input_lower.contains("port") {
        suggestions.push("Try: 'check which process is using port 8080'".to_string());
        suggestions.push("Try: 'test network connectivity to google.com'".to_string());
    }

    // Default impressive suggestions
    if suggestions.is_empty() {
        suggestions.push("Try: 'find duplicate files by content hash'".to_string());
        suggestions.push("Try: 'archive and compress logs older than 30 days'".to_string());
        suggestions.push("Try: 'monitor disk space and alert if usage exceeds 80%'".to_string());
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_critique_detection() {
        assert!(get_critique_for_request("hello world").is_some());
        assert!(get_critique_for_request("make me coffee").is_some());
        assert!(get_critique_for_request("ls").is_none());
        assert!(get_critique_for_request("find large files").is_none());
    }

    #[test]
    fn test_showcase_suggestions() {
        let suggestions = get_showcase_suggestions("find files");
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("PDF")));
    }

    #[test]
    fn test_demo_system_prompt() {
        let prompt = get_demo_system_prompt(ShellType::Bash);
        assert!(prompt.contains("DEMO MODE"));
        assert!(prompt.contains("SASSY"));
        assert!(prompt.contains("showcase"));
    }
}
