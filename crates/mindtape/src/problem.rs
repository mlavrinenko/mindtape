//! One config problem: the message, and what an unknown name in it could have
//! been. The CLI turns the second into `hint:` lines, so no message carries a
//! `did you mean` clause of its own.

use std::fmt;
use std::ops::Deref;

use crate::suggest::Unknown;

/// A problem found while checking a config or a filter, led by the dotted key
/// it is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    /// The message, without a did-you-mean or a list of legal names.
    pub text: String,
    /// The nearest known name and the legal names, for an unknown name.
    pub unknown: Option<Unknown>,
}

impl Problem {
    /// A problem that has no unknown name to suggest.
    #[must_use]
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            unknown: None,
        }
    }

    /// `self` with what the unknown name could have been. Empty when the
    /// caller has nothing to offer, so no hint is made.
    #[must_use]
    pub fn with(mut self, unknown: Unknown) -> Self {
        if unknown.nearest.is_some() || !unknown.names.is_empty() {
            self.unknown = Some(unknown);
        }
        self
    }
}

impl From<String> for Problem {
    fn from(text: String) -> Self {
        Self::new(text)
    }
}

impl From<&str> for Problem {
    fn from(text: &str) -> Self {
        Self::new(text)
    }
}

impl Deref for Problem {
    type Target = str;

    fn deref(&self) -> &str {
        &self.text
    }
}

impl fmt::Display for Problem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}

impl PartialEq<str> for Problem {
    fn eq(&self, other: &str) -> bool {
        self.text == other
    }
}

impl PartialEq<&str> for Problem {
    fn eq(&self, other: &&str) -> bool {
        self.text == *other
    }
}

impl Problem {
    /// `self` led by the dotted key it is about: `<key>: <text>`.
    #[must_use]
    pub fn under(mut self, key: &str) -> Self {
        self.text = format!("{key}: {}", self.text);
        self
    }

    /// `problems` as one problem: the texts joined one per line, and the
    /// first one's unknown name.
    #[must_use]
    pub fn join(problems: Vec<Self>) -> Self {
        let text = problems
            .iter()
            .map(|problem| problem.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let unknown = problems.into_iter().next().and_then(|first| first.unknown);
        Self { text, unknown }
    }
}
