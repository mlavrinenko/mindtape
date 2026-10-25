//! Config types for `[tasks.body.content]` — the section-content gating rules
//! ([`ContentRuleConfig`]) compiled by [`crate::body::content_gate`].
//!
//! Where `[tasks.body.headers]` constrains the heading TREE (which headings
//! exist, their order, their vocabulary), this constrains what is written
//! UNDER a heading: the section's content, everything from the heading down
//! to the next heading of the same or higher level.

use serde::{Deserialize, Deserializer};

use crate::body::headers::{HeaderPathSegment, duplicate_rule_names, rule_label};

/// Rules for what a body section must contain.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ContentConfig {
    /// Ordered list of section-content gating rules.
    pub rules: Vec<ContentRuleConfig>,
}

/// One entry in `[[tasks.body.content.rules]]`.
///
/// `path` selects sections by heading path, the same way a header rule does.
/// `assert` is checked against each matched section's content. Like a header
/// rule, it takes an optional `when` filter and `message` override.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContentRuleConfig {
    /// The name a report shows for the rule (`body.content.rules.<name>`).
    /// Optional: an unnamed rule reports by its position instead
    /// (`body.content.rules[0]`). Two rules in this array sharing a name is a
    /// config error naming both.
    #[serde(default)]
    pub name: Option<String>,
    /// Path through the heading tree selecting the sections to assert over.
    pub path: Vec<HeaderPathSegment>,
    /// The assertion made over each matched section's content.
    pub assert: ContentAssert,
    /// Optional filter expression gating this rule to matching tasks only.
    pub when: Option<String>,
    /// Override for the default violation message.
    pub message: Option<String>,
}

/// The assertion a content rule makes over a matched section.
///
/// - `"non-empty"` — the section must contain any non-whitespace content.
/// - `{ min-words = N }` / `{ max-words = N }` — at least / at most `N` words
///   (any whitespace-delimited token containing an alphanumeric character,
///   over the whole section slice — markup, code blocks, and nested
///   subsection headings all count).
/// - `{ must-match = "regex" }` / `{ must-not-match = "regex" }` — the
///   section's source text as written must / must not match the regex.
/// - `{ density = { pattern = "regex", max = N, per = "section" } }` — no
///   single unit may carry more than `N` matches of the regex; `per` is
///   `section` (the default), `paragraph`, or `sentence`. Unlike the word
///   counts and the regex asserts above, code, fenced or inline, is excluded
///   from matching entirely — see [`crate::body::content_gate`] for the unit-split
///   rules.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentAssert {
    /// The section must contain any non-whitespace content.
    NonEmpty,
    /// The section must contain at least this many words.
    MinWords(usize),
    /// The section must contain at most this many words.
    MaxWords(usize),
    /// The section's source text must match this regex.
    MustMatch(String),
    /// The section's source text must not match this regex.
    MustNotMatch(String),
    /// No single unit of the section may carry more than `max` matches of
    /// `pattern`.
    Density(DensitySpec),
}

/// Fails when one unit of the section carries more than `max` matches of
/// `pattern`, as in `{ density = { pattern, max, per } }`.
#[derive(Debug, Clone, PartialEq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DensitySpec {
    /// Rust regex counted within each unit.
    pub pattern: String,
    /// The most matches of `pattern` a single unit may carry.
    pub max: usize,
    /// Unit to count matches in: `section` (the default), `paragraph` or
    /// `sentence`.
    #[serde(default)]
    pub per: DensityUnit,
}

/// Unit granularity for a [`ContentAssert::Density`] assertion.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DensityUnit {
    /// The whole matched section content is one unit (the default).
    #[default]
    Section,
    /// Blank-line-separated paragraphs; each list item is its own paragraph.
    Paragraph,
    /// Each paragraph split further into sentences.
    Sentence,
}

impl<'de> Deserialize<'de> for DensityUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        match text.as_str() {
            "section" => Ok(Self::Section),
            "paragraph" => Ok(Self::Paragraph),
            "sentence" => Ok(Self::Sentence),
            other => Err(serde::de::Error::custom(format!(
                "invalid density unit `{other}`; expected `section`, `paragraph`, or `sentence`"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for ContentAssert {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AssertVisitor;

        impl<'de> serde::de::Visitor<'de> for AssertVisitor {
            type Value = ContentAssert;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str(
                    "a string `\"non-empty\"` or a table with one of \
                             `min-words`, `max-words`, `must-match`, `must-not-match`, `density`",
                )
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "non-empty" => Ok(ContentAssert::NonEmpty),
                    other => Err(serde::de::Error::custom(format!(
                        "invalid assertion `{other}`; expected `\"non-empty\"` or a table \
                         with one of `min-words`, `max-words`, `must-match`, `must-not-match`, \
                         `density`"
                    ))),
                }
            }

            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut out: Option<ContentAssert> = None;
                while let Some(key) = map.next_key::<String>()? {
                    let value = match key.as_str() {
                        "min-words" => ContentAssert::MinWords(map.next_value()?),
                        "max-words" => ContentAssert::MaxWords(map.next_value()?),
                        "must-match" => ContentAssert::MustMatch(map.next_value()?),
                        "must-not-match" => ContentAssert::MustNotMatch(map.next_value()?),
                        "density" => ContentAssert::Density(map.next_value()?),
                        other => {
                            return Err(serde::de::Error::unknown_field(
                                other,
                                &[
                                    "min-words",
                                    "max-words",
                                    "must-match",
                                    "must-not-match",
                                    "density",
                                ],
                            ));
                        }
                    };
                    if out.is_some() {
                        return Err(serde::de::Error::custom(
                            "assertion table must carry exactly one of `min-words`, \
                             `max-words`, `must-match`, `must-not-match`, `density`",
                        ));
                    }
                    out = Some(value);
                }
                out.ok_or_else(|| serde::de::Error::missing_field("assertion"))
            }
        }

        deserializer.deserialize_any(AssertVisitor)
    }
}

impl ContentRuleConfig {
    /// How this rule identifies itself in a report: `body.content.rules.<name>`
    /// if it has one, else its position, `body.content.rules[<index>]` — the
    /// same convention [`crate::body::headers::HeaderRuleConfig::label`] uses.
    #[must_use]
    pub fn label(&self, index: usize) -> String {
        rule_label(self.name.as_deref(), "body.content.rules", index)
    }
}

/// Cross-check for `[[tasks.body.content.rules]]`, folded into
/// [`crate::config::Config::validate`]: no two entries share a `name`.
pub(crate) fn validate_content_rules(config: &crate::config::Config) -> Vec<String> {
    duplicate_rule_names(
        config
            .tasks
            .body
            .content
            .rules
            .iter()
            .map(|rule| rule.name.as_deref()),
        "tasks.body.content.rules",
    )
}

#[cfg(test)]
#[path = "content_tests.rs"]
mod tests;
