//! Config types for `[tasks.body.content]`, the section-content gating rules
//! ([`ContentRuleConfig`]) compiled by [`crate::body::content_gate`].
//!
//! Where `[tasks.body.headers]` constrains the heading tree (which headings
//! exist, their order, their vocabulary), this constrains what a heading
//! holds. That is the section's content, everything from the heading down to
//! the next heading of the same or higher level.

use serde::{Deserialize, Deserializer};

use crate::body::headers::{HeaderPathSegment, duplicate_rule_names, rule_label};

/// Rules for what a body section must contain.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct ContentConfig {
    /// Defaults every content rule inherits.
    pub words: WordDefaults,
    /// Ordered list of section-content gating rules.
    pub rules: Vec<ContentRuleConfig>,
}

/// The defaults every `words` assertion inherits.
///
/// Only `count` lives here. A bound stays on the rule that enforces it, so
/// `mt` refuses a `[tasks.body.content] words` table carrying `min` or `max`.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct WordDefaults {
    /// Unit a rule's word count uses when its own `words` table omits `count`.
    pub count: WordCount,
}

/// One entry in `[[tasks.body.content.rules]]`.
///
/// `path` selects sections by heading path, the same way a header rule does.
/// The rule checks `assert` against each matched section's content. Like a
/// header rule, it takes an optional `when` filter and `message` override.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContentRuleConfig {
    /// The name a report shows for the rule (`body.content.rules.<name>`).
    /// Optional: an unnamed rule reports by its position instead
    /// (`body.content.rules[0]`). Two rules in this array sharing a name is a
    /// config error that reports both.
    #[serde(default)]
    pub name: Option<String>,
    /// Path through the heading tree selecting the sections to assert over.
    pub path: Vec<HeaderPathSegment>,
    /// The assertion made over each matched section's content.
    pub assert: ContentAssert,
    /// Optional filter expression gating this rule to matching tasks only.
    #[schemars(extend("x-expression" = "filter"))]
    pub when: Option<String>,
    /// Override for the default violation message.
    pub message: Option<String>,
}

/// The assertion a content rule makes over a matched section.
///
/// - `"non-empty"`: the section must contain any non-whitespace content.
/// - `{ words = { min = N, max = M } }`: at least `N` and / or at most `M`
///   words, where a word is any whitespace-delimited token containing an
///   alphanumeric character. `count` chooses what the bound counts. `all`
///   (the default) sees markup, code and nested headings. `prose` drops
///   fenced code. `text` also masks inline code. A rule that omits `count`
///   inherits `[tasks.body.content] words.count`.
/// - `{ must-match = "regex" }` / `{ must-not-match = "regex" }`: the
///   section's source text as written must / must not match the regex.
/// - `{ density = { pattern = "regex", max = N, per = "section" } }`: no
///   single unit may carry more than `N` matches of the regex. `per` is
///   `section` (the default), `paragraph`, or `sentence`. Unlike the word
///   counts and the regex asserts above, the check excludes code, fenced or
///   inline, from matching entirely. See [`crate::body::content_gate`] for the
///   unit-split rules.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentAssert {
    /// The section must contain any non-whitespace content.
    NonEmpty,
    /// The section's word count must carry this bound, in this unit.
    Words(WordSpec),
    /// The section's source text must match this regex.
    MustMatch(String),
    /// The section's source text must not match this regex.
    MustNotMatch(String),
    /// No single unit of the section may carry more than `max` matches of
    /// `pattern`.
    Density(DensitySpec),
}

/// A word-count bound, as in `{ words = { min = 10, max = 120 } }`.
///
/// A `words` table must carry at least one of `min` and `max`, else it bounds
/// nothing. `count` overrides the section default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordSpec {
    /// The fewest words the section may contain.
    pub min: Option<usize>,
    /// The most words the section may contain.
    pub max: Option<usize>,
    /// Unit to count words in. Absent, it inherits
    /// `[tasks.body.content] words.count`.
    pub count: Option<WordCount>,
}

impl<'de> Deserialize<'de> for WordSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields {
            #[serde(default)]
            min: Option<usize>,
            #[serde(default)]
            max: Option<usize>,
            #[serde(default)]
            count: Option<WordCount>,
        }

        let fields = Fields::deserialize(deserializer)?;
        if fields.min.is_none() && fields.max.is_none() {
            return Err(serde::de::Error::custom(
                "a `words` table must carry `min` or `max`",
            ));
        }
        Ok(Self {
            min: fields.min,
            max: fields.max,
            count: fields.count,
        })
    }
}

/// The unit a `words` assertion counts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WordCount {
    /// Every token in the section slice, markup and code included (the
    /// default).
    #[default]
    All,
    /// The section's prose. The count drops fenced code and keeps inline code
    /// as part of the sentence it sits in.
    Prose,
    /// The section's text outside code. The count drops fenced code and masks
    /// inline raw spans.
    Text,
}

impl<'de> Deserialize<'de> for WordCount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        match text.as_str() {
            "all" => Ok(Self::All),
            "prose" => Ok(Self::Prose),
            "text" => Ok(Self::Text),
            other => Err(serde::de::Error::custom(format!(
                "invalid word count `{other}`; expected `all`, `prose`, or `text`"
            ))),
        }
    }
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

/// The unit a [`ContentAssert::Density`] assertion counts matches in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DensityUnit {
    /// The whole matched section content is one unit (the default).
    #[default]
    Section,
    /// Blank-line-separated paragraphs. Each list item is its own paragraph.
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
                             `words`, `must-match`, `must-not-match`, `density`",
                )
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                match value {
                    "non-empty" => Ok(ContentAssert::NonEmpty),
                    other => Err(serde::de::Error::custom(format!(
                        "invalid assertion `{other}`; expected `\"non-empty\"` or a table \
                         with one of `words`, `must-match`, `must-not-match`, `density`"
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
                        "words" => ContentAssert::Words(map.next_value()?),
                        "must-match" => ContentAssert::MustMatch(map.next_value()?),
                        "must-not-match" => ContentAssert::MustNotMatch(map.next_value()?),
                        "density" => ContentAssert::Density(map.next_value()?),
                        other => {
                            return Err(serde::de::Error::unknown_field(
                                other,
                                &["words", "must-match", "must-not-match", "density"],
                            ));
                        }
                    };
                    if out.is_some() {
                        return Err(serde::de::Error::custom(
                            "assertion table must carry exactly one of `words`, \
                             `must-match`, `must-not-match`, `density`",
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
    /// The name this rule reports under: `body.content.rules.<name>` if it has
    /// one, else its position, `body.content.rules[<index>]`. That is the same
    /// convention [`crate::body::headers::HeaderRuleConfig::label`] uses.
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
