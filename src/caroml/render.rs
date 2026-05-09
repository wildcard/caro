//! Markdown renderer — `caro render <name>` produces a documentation-grade
//! `.md` from a `.caro` task file.
//!
//! v0.1 is **forward-only**: `.caro` → Markdown. v0.2 may add reverse parsing
//! so a renderable `.md` round-trips back to a `.caro`.

use crate::caroml::ast::Task;

/// Render a parsed [`Task`] as Markdown.
///
/// Layout:
///
/// ```markdown
/// # <TASK title>
///
/// > <WHY...>
///
/// **Requires:** sudo, jq
///
/// > **macos:** prefers `bsd-tools`
/// > **linux:** prefers `gnu-tools`
///
/// **Parameters:**
/// - `path` = `/var/log`
/// - `days` = `30`
///
/// 1. find log files in /var/log
///    <small>note: prefer single-pass find</small>
/// 2. delete files older than 30 days
/// 3. ...
/// ```
pub fn render_markdown(task: &Task) -> String {
    let mut s = String::new();

    s.push_str(&format!("# {}\n\n", task.title));

    if let Some(why) = &task.why {
        s.push_str(&format!("> {}\n\n", why));
    }

    if !task.needs.is_empty() {
        s.push_str(&format!("**Requires:** {}\n\n", task.needs.join(", ")));
    }

    if !task.platform_pragmas.is_empty() {
        for p in &task.platform_pragmas {
            s.push_str(&format!("> **{}:**", p.platform));
            if !p.prefer.is_empty() {
                s.push_str(&format!(
                    " prefers {}",
                    p.prefer
                        .iter()
                        .map(|x| format!("`{}`", x))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if !p.avoid.is_empty() {
                s.push_str(&format!(
                    " · avoids {}",
                    p.avoid
                        .iter()
                        .map(|x| format!("`{}`", x))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            s.push('\n');
        }
        s.push('\n');
    }

    if !task.params.is_empty() {
        s.push_str("**Parameters:**\n\n");
        for p in &task.params {
            s.push_str(&format!("- `{}` = `{}`\n", p.name, p.value));
        }
        s.push('\n');
    }

    s.push_str("**Steps:**\n\n");
    for (i, step) in task.steps.iter().enumerate() {
        s.push_str(&format!("{}. {}\n", i + 1, step.intent));
        for note in &step.notes {
            s.push_str(&format!("   <small>note: {}</small>\n", note));
        }
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caroml::ast::{Param, PlatformPragma, Step, Task};

    fn full_task() -> Task {
        Task {
            source_path: None,
            title: "Clean up old log files".into(),
            why: Some("Free disk space, runs weekly via cron".into()),
            needs: vec!["sudo".into(), "jq".into()],
            platform_pragmas: vec![
                PlatformPragma {
                    platform: "macos".into(),
                    prefer: vec!["bsd-tools".into()],
                    avoid: vec![],
                },
                PlatformPragma {
                    platform: "linux".into(),
                    prefer: vec!["gnu-tools".into()],
                    avoid: vec![],
                },
            ],
            params: vec![
                Param {
                    name: "path".into(),
                    value: "/var/log".into(),
                },
                Param {
                    name: "days".into(),
                    value: "30".into(),
                },
            ],
            steps: vec![
                Step {
                    line: 12,
                    intent: "find log files in /var/log".into(),
                    raw_intent: "find log files in {path}".into(),
                    notes: vec!["prefer single-pass find".into()],
                },
                Step {
                    line: 13,
                    intent: "delete files older than 30 days".into(),
                    raw_intent: "delete files older than {days} days".into(),
                    notes: vec![],
                },
            ],
        }
    }

    #[test]
    fn markdown_includes_title_as_h1() {
        let md = render_markdown(&full_task());
        assert!(md.starts_with("# Clean up old log files\n"));
    }

    #[test]
    fn markdown_includes_why_as_blockquote() {
        let md = render_markdown(&full_task());
        assert!(md.contains("> Free disk space, runs weekly via cron"));
    }

    #[test]
    fn markdown_lists_needs() {
        let md = render_markdown(&full_task());
        assert!(md.contains("**Requires:** sudo, jq"));
    }

    #[test]
    fn markdown_renders_per_platform_pragmas() {
        let md = render_markdown(&full_task());
        assert!(md.contains("> **macos:**"));
        assert!(md.contains("`bsd-tools`"));
        assert!(md.contains("> **linux:**"));
        assert!(md.contains("`gnu-tools`"));
    }

    #[test]
    fn markdown_renders_steps_as_numbered_list_with_notes() {
        let md = render_markdown(&full_task());
        assert!(md.contains("1. find log files in /var/log"));
        assert!(md.contains("<small>note: prefer single-pass find</small>"));
        assert!(md.contains("2. delete files older than 30 days"));
    }

    #[test]
    fn empty_optional_fields_are_omitted() {
        let mut task = full_task();
        task.why = None;
        task.needs = vec![];
        task.platform_pragmas = vec![];
        task.params = vec![];
        task.steps[0].notes = vec![];
        let md = render_markdown(&task);
        assert!(!md.contains("**Requires:**"));
        assert!(!md.contains("**Parameters:**"));
        assert!(!md.contains("> **macos:**"));
        assert!(!md.contains("<small>"));
    }
}
