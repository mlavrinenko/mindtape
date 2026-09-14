//! Config types for `[tasks.body.content]` — the section-content gating rules
//! ([`ContentRuleConfig`]) compiled by [`crate::content_gate`].
//!
//! Where `[tasks.body.headers]` constrains the heading TREE (which headings
//! exist, their order, their vocabulary), this constrains what is written
//! UNDER a heading: the section's content, everything from the heading down
//! to the next heading of the same or higher level.

use serde::{Deserialize, Deserializer};

use crate::body_headers::HeaderPathSegment;

/// `[tasks.body.content]` — section-content rules.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ContentConfig {
    /// Ordered list of section-content gating rules.
    pub rules: Vec<ContentRuleConfig>,
}

/// One entry in `[[tasks.body.content.rules]]`.
///
/// `path` selects the section(s) — the heading tree paths the header rules
/// already use — and `assert` is the assertion made over each matched
/// section's content. Like every header rule, it carries the optional `when`
/// filter and `message` override.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContentRuleConfig {
    /// Path through the heading tree selecting the section(s) to assert over.
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
                             `min-words`, `max-words`, `must-match`, `must-not-match`",
                )
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "non-empty" => Ok(ContentAssert::NonEmpty),
                    other => Err(serde::de::Error::custom(format!(
                        "invalid assertion `{other}`; expected `\"non-empty\"` or a table \
                         with one of `min-words`, `max-words`, `must-match`, `must-not-match`"
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
                        other => {
                            return Err(serde::de::Error::unknown_field(
                                other,
                                &["min-words", "max-words", "must-match", "must-not-match"],
                            ));
                        }
                    };
                    if out.is_some() {
                        return Err(serde::de::Error::custom(
                            "assertion table must carry exactly one of `min-words`, \
                             `max-words`, `must-match`, `must-not-match`",
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

#[cfg(test)]
#[path = "body_content_tests.rs"]
mod tests;
