//! STE-lite checker
//!
//! Mechanical checks taken from ASD-STE100 (Simplified Technical English,
//! Issue 9, 2025), the controlled English of aircraft maintenance manuals.
//! "STE-lite" means the sentence, paragraph, voice and word rules, but not
//! the full approved dictionary.
//!
//! The checker flags violations. It cannot judge if a text is good, and the
//! passive-voice check is a heuristic. Use it as a regression guard for text
//! that users read, such as explain mode.
//!
//! Background: `docs/research/2026-10-03-legible-output-ste100.md`.

/// The kind of text, which sets the sentence length limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextKind {
    /// Instructions: STE rule 5.1, 20 words or fewer per sentence.
    Procedural,
    /// Descriptions: STE rule 6.3, 25 words or fewer per sentence.
    Descriptive,
}

impl TextKind {
    /// The maximum number of words in one sentence.
    pub fn max_words(self) -> usize {
        match self {
            TextKind::Procedural => 20,
            TextKind::Descriptive => 25,
        }
    }
}

/// STE limits a paragraph to six sentences on one topic.
pub const MAX_PARAGRAPH_SENTENCES: usize = 6;

/// One STE-lite violation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SteIssue {
    SentenceTooLong {
        sentence: String,
        words: usize,
        max: usize,
    },
    ParagraphTooLong {
        sentences: usize,
    },
    UnapprovedWord {
        word: &'static str,
        use_instead: &'static str,
    },
    PassiveVoice {
        phrase: String,
    },
}

/// Words and phrases that STE-lite replaces. An empty replacement means
/// "delete the word".
const UNAPPROVED: &[(&str, &str)] = &[
    ("utilize", "use"),
    ("utilise", "use"),
    ("leverage", "use"),
    ("commence", "start"),
    ("prior to", "before"),
    ("in order to", "to"),
    ("subsequently", "then"),
    ("numerous", "many"),
    ("simply", ""),
    ("just", ""),
    ("basically", ""),
    ("obviously", ""),
    ("easily", ""),
    ("very", ""),
    ("really", ""),
];

const PASSIVE_AUX: &[&str] = &["is", "are", "was", "were", "be", "been", "being"];

const IRREGULAR_PARTICIPLES: &[&str] = &[
    "done", "shown", "given", "made", "found", "written", "seen", "known", "taken", "kept", "sent",
    "built", "hidden",
];

/// Check `text` against the STE-lite rules.
///
/// Inline code in backticks counts as one word, because STE allows
/// technical names. Paragraphs are separated by a blank line.
pub fn check(text: &str, kind: TextKind) -> Vec<SteIssue> {
    let text = mask_code_spans(text);
    let mut issues = Vec::new();

    for paragraph in text.split("\n\n").filter(|p| !p.trim().is_empty()) {
        let sentences = split_sentences(paragraph);
        if sentences.len() > MAX_PARAGRAPH_SENTENCES {
            issues.push(SteIssue::ParagraphTooLong {
                sentences: sentences.len(),
            });
        }
        for sentence in sentences {
            let words = sentence
                .split_whitespace()
                .filter(|w| w.chars().any(char::is_alphanumeric))
                .count();
            if words > kind.max_words() {
                issues.push(SteIssue::SentenceTooLong {
                    sentence: sentence.to_string(),
                    words,
                    max: kind.max_words(),
                });
            }
        }
    }

    let normalized = format!(" {} ", lowercase_words(&text).join(" "));
    for (word, use_instead) in UNAPPROVED {
        if normalized.contains(&format!(" {} ", word)) {
            issues.push(SteIssue::UnapprovedWord { word, use_instead });
        }
    }

    let words = lowercase_words(&text);
    for pair in words.windows(2) {
        if PASSIVE_AUX.contains(&pair[0].as_str()) && is_participle(&pair[1]) {
            issues.push(SteIssue::PassiveVoice {
                phrase: format!("{} {}", pair[0], pair[1]),
            });
        }
    }

    issues
}

/// Replace each `code span` with a single placeholder word.
fn mask_code_spans(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_code = false;
    for c in text.chars() {
        if c == '`' {
            if !in_code {
                out.push_str("CODE");
            }
            in_code = !in_code;
        } else if !in_code {
            out.push(c);
        }
    }
    out
}

/// Split a paragraph into sentences at `.`, `!`, `?` followed by
/// whitespace or the end, and at line breaks (list items).
fn split_sentences(paragraph: &str) -> Vec<&str> {
    let mut sentences = Vec::new();
    for line in paragraph.lines() {
        let mut start = 0;
        let chars: Vec<(usize, char)> = line.char_indices().collect();
        for (i, &(pos, c)) in chars.iter().enumerate() {
            let at_boundary = chars.get(i + 1).is_none_or(|&(_, n)| n.is_whitespace());
            if matches!(c, '.' | '!' | '?') && at_boundary {
                sentences.push(&line[start..pos + 1]);
                start = pos + 1;
            }
        }
        sentences.push(&line[start..]);
    }
    sentences.retain(|s| s.chars().any(char::is_alphanumeric));
    sentences.iter().map(|s| s.trim()).collect()
}

fn lowercase_words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn is_participle(word: &str) -> bool {
    (word.len() > 3 && word.ends_with("ed")) || IRREGULAR_PARTICIPLES.contains(&word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_text_has_no_issues() {
        let text =
            "`find` searches a directory tree for files. It starts at the path that you give.";
        assert_eq!(check(text, TextKind::Descriptive), vec![]);
    }

    #[test]
    fn flags_procedural_sentence_over_20_words() {
        let text = "Run the command in the directory that holds the files that you want to search \
                    so that it can find all of them for you.";
        assert!(matches!(
            check(text, TextKind::Procedural).as_slice(),
            [SteIssue::SentenceTooLong { max: 20, .. }]
        ));
    }

    #[test]
    fn descriptive_limit_is_25_words() {
        // 23 words: too long for a procedure, acceptable for a description.
        let text = "The command reads each file in the tree and shows each line that has the \
                    pattern that you gave it on the line.";
        assert!(check(text, TextKind::Descriptive).is_empty());
        assert!(!check(text, TextKind::Procedural).is_empty());
    }

    #[test]
    fn code_span_counts_as_one_word() {
        let text =
            "Run `find . -type f -name '*.rs' -mtime -1 -size +100k -print0 | xargs -0 ls -l` now.";
        assert!(check(text, TextKind::Procedural).is_empty());
    }

    #[test]
    fn flags_paragraph_over_six_sentences() {
        let text = "One. Two. Three. Four. Five. Six. Seven.";
        assert_eq!(
            check(text, TextKind::Descriptive),
            vec![SteIssue::ParagraphTooLong { sentences: 7 }]
        );
    }

    #[test]
    fn flags_unapproved_words_and_phrases() {
        let issues = check("Utilize grep prior to sorting.", TextKind::Procedural);
        assert!(issues.contains(&SteIssue::UnapprovedWord {
            word: "utilize",
            use_instead: "use"
        }));
        assert!(issues.contains(&SteIssue::UnapprovedWord {
            word: "prior to",
            use_instead: "before"
        }));
    }

    #[test]
    fn unapproved_word_must_match_whole_word() {
        // "adjust" contains "just", "justify" starts with it.
        assert!(check("Adjust the path to justify it.", TextKind::Procedural).is_empty());
    }

    #[test]
    fn flags_passive_voice() {
        let issues = check(
            "The files are deleted by the command.",
            TextKind::Descriptive,
        );
        assert_eq!(
            issues,
            vec![SteIssue::PassiveVoice {
                phrase: "are deleted".to_string()
            }]
        );
    }

    #[test]
    fn decimal_point_does_not_split_sentence() {
        let sentences = split_sentences("Use version 1.4 or later. Then run it.");
        assert_eq!(sentences, vec!["Use version 1.4 or later.", "Then run it."]);
    }
}
