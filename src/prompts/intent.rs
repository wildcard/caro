//! Intent categorisation as a typed decision (#1463, ADR-017).
//!
//! Before this module, "what kind of request is this?" was a prompt
//! instruction ("STEP 1: CATEGORIZE") plus a substring match in
//! `TemplateLibrary::find_template`. Both produce a label with no
//! probability, so nothing downstream can tell a confident classification
//! from a coin flip.
//!
//! [`IntentCategory`] names the template categories as a closed set, and
//! [`classify_intent`] returns a [`Choice`] over them from a deterministic
//! prior: word coverage of each template's `intent_pattern` by the query.
//! It costs nothing, needs no model, and is the baseline any LLM refinement
//! (via [`crate::decision::parse_choice_json`]) has to beat in eval.

use std::str::FromStr;

use crate::decision::Choice;

use super::command_templates::CommandTemplate;

/// Closed set of request categories, mirroring `CommandTemplate::category`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntentCategory {
    Listing,
    Filtering,
    TextSearch,
    Ranking,
    Counting,
    Disk,
    Process,
    Network,
    Archive,
    Permissions,
    /// No template category matched.
    Other,
}

impl IntentCategory {
    /// Every category, in the order used for tie-breaking (destructive or
    /// state-changing categories first so a tie fails toward caution).
    pub const ALL: [IntentCategory; 11] = [
        IntentCategory::Permissions,
        IntentCategory::Process,
        IntentCategory::Archive,
        IntentCategory::Network,
        IntentCategory::Disk,
        IntentCategory::Filtering,
        IntentCategory::TextSearch,
        IntentCategory::Ranking,
        IntentCategory::Counting,
        IntentCategory::Listing,
        IntentCategory::Other,
    ];

    /// The `CommandTemplate::category` string for this variant.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Listing => "listing",
            Self::Filtering => "filtering",
            Self::TextSearch => "text_search",
            Self::Ranking => "ranking",
            Self::Counting => "counting",
            Self::Disk => "disk",
            Self::Process => "process",
            Self::Network => "network",
            Self::Archive => "archive",
            Self::Permissions => "permissions",
            Self::Other => "other",
        }
    }
}

impl FromStr for IntentCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "listing" => Ok(Self::Listing),
            "filtering" => Ok(Self::Filtering),
            "text_search" | "text-search" | "search" => Ok(Self::TextSearch),
            "ranking" => Ok(Self::Ranking),
            "counting" => Ok(Self::Counting),
            "disk" => Ok(Self::Disk),
            "process" => Ok(Self::Process),
            "network" => Ok(Self::Network),
            "archive" => Ok(Self::Archive),
            "permissions" => Ok(Self::Permissions),
            "other" => Ok(Self::Other),
            _ => Err(()),
        }
    }
}

impl std::fmt::Display for IntentCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Words that carry no intent signal on their own.
const STOPWORDS: &[&str] = &[
    "a", "an", "the", "of", "in", "on", "by", "to", "for", "with", "and", "or", "me", "my", "all",
    "this", "that", "is", "are", "be", "show", "get", "please", "i", "want", "need", "how", "do",
    "can", "you",
];

/// Tokenise for intent matching: lowercase, alphanumeric words, stopwords
/// removed, a trailing plural `s` stripped so "file" and "files" agree.
pub(crate) fn words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() > 1 && !STOPWORDS.contains(w))
        .map(|w| {
            if w.len() > 3 && w.ends_with('s') && !w.ends_with("ss") {
                w[..w.len() - 1].to_string()
            } else {
                w.to_string()
            }
        })
        .collect()
}

/// Fraction of a template's intent-pattern words present in the query.
/// `0.0` when the pattern has no signal words.
pub(crate) fn pattern_coverage(query_words: &[String], template: &CommandTemplate) -> f64 {
    let pattern_words = words(&template.intent_pattern);
    if pattern_words.is_empty() {
        return 0.0;
    }
    let hits = pattern_words
        .iter()
        .filter(|w| query_words.contains(w))
        .count();
    hits as f64 / pattern_words.len() as f64
}

/// Deterministic prior over [`IntentCategory`] for `query`, given the
/// available templates: each category's weight is the best coverage any of
/// its templates achieves. If no template overlaps at all, the whole mass
/// goes to [`IntentCategory::Other`].
pub fn classify_intent(query: &str, templates: &[CommandTemplate]) -> Choice<IntentCategory> {
    let query_words = words(query);
    let mut pairs: Vec<(IntentCategory, f64)> = IntentCategory::ALL
        .iter()
        .map(|cat| {
            let best = templates
                .iter()
                .filter(|t| t.category == cat.as_str())
                .map(|t| pattern_coverage(&query_words, t))
                .fold(0.0_f64, f64::max);
            (*cat, best)
        })
        .collect();
    if pairs.iter().all(|(_, w)| *w == 0.0) {
        if let Some(other) = pairs.iter_mut().find(|(c, _)| *c == IntentCategory::Other) {
            other.1 = 1.0;
        }
    }
    Choice::from_weights(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::capability_profile::CapabilityProfile;
    use crate::prompts::command_templates::TemplateLibrary;

    fn templates() -> Vec<CommandTemplate> {
        TemplateLibrary::for_profile(&CapabilityProfile::ubuntu())
            .all_templates()
            .to_vec()
    }

    #[test]
    fn prior_ranks_file_queries_as_listing_or_filtering() {
        // Regression guard for #1463.
        let t = templates();
        let c = classify_intent("find files larger than 100MB", &t);
        assert_eq!(c.argmax().unwrap().0, &IntentCategory::Filtering);

        let c = classify_intent("list all files in this directory", &t);
        assert_eq!(c.argmax().unwrap().0, &IntentCategory::Listing);

        let c = classify_intent("kill the process named node", &t);
        assert_eq!(c.argmax().unwrap().0, &IntentCategory::Process);
    }

    #[test]
    fn unrelated_query_is_other_with_full_mass() {
        let c = classify_intent("bake a sourdough loaf", &templates());
        assert_eq!(c.argmax(), Some((&IntentCategory::Other, 1.0)));
    }

    #[test]
    fn distribution_is_normalised_and_confidence_is_bounded() {
        let c = classify_intent("count lines in file and list files", &templates());
        let total: f64 = c.dist.iter().map(|(_, p)| p).sum();
        assert!((total - 1.0).abs() < 1e-9);
        assert!(c.confidence() > 0.0 && c.confidence() <= 1.0);
    }

    #[test]
    fn from_str_round_trips_every_category() {
        for cat in IntentCategory::ALL {
            assert_eq!(cat.as_str().parse::<IntentCategory>(), Ok(cat));
        }
        assert!("bogus".parse::<IntentCategory>().is_err());
    }
}
