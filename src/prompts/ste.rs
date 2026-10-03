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
    let text = mask_code_spans(&text.replace("\r\n", "\n"));
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
                    sentence,
                    words,
                    max: kind.max_words(),
                });
            }
        }
    }

    let words = lowercase_words(&text);
    let normalized = format!(" {} ", words.join(" "));
    for (word, use_instead) in UNAPPROVED {
        if normalized.contains(&format!(" {} ", word)) {
            issues.push(SteIssue::UnapprovedWord { word, use_instead });
        }
    }

    // Auxiliary + participle, with at most one adverb between ("is often used").
    for (i, aux) in words.iter().enumerate() {
        if !PASSIVE_AUX.contains(&aux.as_str()) {
            continue;
        }
        let mut j = i + 1;
        if words.get(j).is_some_and(|w| is_adverb(w)) {
            j += 1;
        }
        if words.get(j).is_some_and(|w| is_participle(w)) {
            issues.push(SteIssue::PassiveVoice {
                phrase: words[i..=j].join(" "),
            });
        }
    }

    issues
}

/// Replace each balanced `code span` with a single placeholder word.
/// An unmatched backtick stays as text, so the rest is still checked.
fn mask_code_spans(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('`') {
        let Some(len) = rest[open + 1..].find('`') else {
            break;
        };
        out.push_str(&rest[..open]);
        out.push_str("CODE");
        rest = &rest[open + len + 2..];
    }
    out.push_str(rest);
    out
}

/// Split a paragraph into sentences.
///
/// A wrapped line continues its sentence. A list item (`- `, `* `, `+ `,
/// `1. `) starts a new one. A sentence ends at `.`, `!` or `?`, plus any
/// closing quotes or brackets, when whitespace or the end follows and the
/// next word does not start with a lowercase letter ("e.g. the" does not
/// split).
fn split_sentences(paragraph: &str) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    for line in paragraph.lines().map(str::trim).filter(|l| !l.is_empty()) {
        match items.last_mut() {
            Some(last) if !is_list_item(line) => {
                last.push(' ');
                last.push_str(line);
            }
            _ => items.push(line.to_string()),
        }
    }

    let mut sentences = Vec::new();
    for item in &items {
        let chars: Vec<(usize, char)> = item.char_indices().collect();
        let mut start = 0;
        let mut i = 0;
        while i < chars.len() {
            if matches!(chars[i].1, '.' | '!' | '?') {
                let mut end = i + 1;
                while chars
                    .get(end)
                    .is_some_and(|&(_, c)| matches!(c, '"' | '\'' | ')' | ']' | '”' | '’'))
                {
                    end += 1;
                }
                let spaced = chars.get(end).is_none_or(|&(_, c)| c.is_whitespace());
                let next = chars[end..]
                    .iter()
                    .map(|&(_, c)| c)
                    .find(|c| !c.is_whitespace());
                if spaced && !next.is_some_and(char::is_lowercase) {
                    let byte_end = chars.get(end).map_or(item.len(), |&(pos, _)| pos);
                    sentences.push(item[start..byte_end].trim().to_string());
                    start = byte_end;
                    i = end;
                    continue;
                }
            }
            i += 1;
        }
        sentences.push(item[start..].trim().to_string());
    }
    sentences.retain(|s| s.chars().any(char::is_alphanumeric));
    sentences
}

fn is_list_item(line: &str) -> bool {
    ["- ", "* ", "+ "].iter().any(|m| line.starts_with(m))
        || line
            .split_once(". ")
            .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
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

const ADVERBS: &[&str] = &[
    "not", "often", "also", "always", "never", "usually", "still", "only", "then", "now",
];

fn is_adverb(word: &str) -> bool {
    ADVERBS.contains(&word) || (word.len() > 4 && word.ends_with("ly"))
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
    fn flags_passive_voice_with_adverb_between() {
        let issues = check("The flag is often used with grep.", TextKind::Descriptive);
        assert_eq!(
            issues,
            vec![SteIssue::PassiveVoice {
                phrase: "is often used".to_string()
            }]
        );
    }

    #[test]
    fn crlf_blank_line_separates_paragraphs() {
        let text = "One. Two. Three. Four.\r\n\r\nFive. Six. Seven.";
        assert!(check(text, TextKind::Descriptive).is_empty());
    }

    #[test]
    fn unmatched_backtick_does_not_hide_the_rest() {
        let text = "Run `find now. Then utilize the output.";
        assert!(
            check(text, TextKind::Procedural).contains(&SteIssue::UnapprovedWord {
                word: "utilize",
                use_instead: "use"
            })
        );
    }

    #[test]
    fn wrapped_line_continues_the_sentence() {
        let text = "Run the command in the directory that holds the files\n\
                    that you want to search so that it can find all of them for you.";
        assert!(matches!(
            check(text, TextKind::Procedural).as_slice(),
            [SteIssue::SentenceTooLong { words: 25, .. }]
        ));
    }

    #[test]
    fn list_items_are_separate_sentences() {
        let sentences = split_sentences("- Match only files\n- Match only directories");
        assert_eq!(
            sentences,
            vec!["- Match only files", "- Match only directories"]
        );
    }

    #[test]
    fn abbreviation_does_not_split_sentence() {
        assert_eq!(
            split_sentences("Use a pattern, e.g. a glob. Then run it."),
            vec!["Use a pattern, e.g. a glob.", "Then run it."]
        );
    }

    #[test]
    fn sentence_can_end_inside_quotes() {
        assert_eq!(
            split_sentences("It means \"modified today.\" Use it."),
            vec!["It means \"modified today.\"", "Use it."]
        );
    }

    #[test]
    fn decimal_point_does_not_split_sentence() {
        let sentences = split_sentences("Use version 1.4 or later. Then run it.");
        assert_eq!(sentences, vec!["Use version 1.4 or later.", "Then run it."]);
    }
}
