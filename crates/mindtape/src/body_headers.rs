//! Config types for `[tasks.body.headers]` — the body-header gating rules
//! ([`HeaderRuleConfig`]) compiled into `artifact-gate` rules by [`crate::gate`].

use serde::{Deserialize, Deserializer};

/// `[tasks.body.headers]` — heading-presence rules.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct HeadersConfig {
    /// Ordered list of heading gating rules.
    pub rules: Vec<HeaderRuleConfig>,
}

/// One entry in `[[tasks.body.headers.rules]]`.
///
/// `action` selects the rule shape and which fields apply:
/// - `require` / `forbid` / `unique` — cardinality over `path`.
/// - `order` — `before` headings must precede `after` headings.
/// - `allowlist` — closed-world: under `scope` (or top-level when omitted), only
///   `allowed` headings are permitted.
#[derive(Debug, Clone, Deserialize, PartialEq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HeaderRuleConfig {
    /// Whether the target headings must exist, must not, or must be unique, or
    /// whether this is an `order`/`allowlist` rule.
    pub action: HeaderAction,
    /// Path through the heading tree (cardinality actions).
    #[serde(default)]
    pub path: Vec<HeaderPathSegment>,
    /// Path whose headings must come first (`order`).
    #[serde(default)]
    pub before: Vec<HeaderPathSegment>,
    /// Path whose headings must come after (`order`).
    #[serde(default)]
    pub after: Vec<HeaderPathSegment>,
    /// Path selecting scopes whose children are constrained (`allowlist`);
    /// omit to constrain top-level headings.
    #[serde(default)]
    pub scope: Vec<HeaderPathSegment>,
    /// The permitted headings (`allowlist`).
    #[serde(default)]
    pub allowed: Vec<HeaderPathSegment>,
    /// Optional filter expression gating this rule to matching tasks only.
    pub when: Option<String>,
    /// Override for the default violation message.
    pub message: Option<String>,
}

/// One step in a heading path.
///
/// - A bare TOML string is an exact, case-sensitive, direct-child match.
/// - `{ match = "regex" }` is a regex match; `recursive = true` finds any descendant.
#[derive(Debug, Clone, PartialEq)]
pub enum HeaderPathSegment {
    /// Exact heading text, direct-child position.
    Exact(String),
    /// Regex heading text pattern.
    Pattern {
        /// Rust regex pattern matched against heading text.
        pattern: String,
        /// Search any descendant (`true`) or only direct children (`false`, default).
        recursive: bool,
    },
}

/// The shape of a header rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderAction {
    /// The path must match at least one heading.
    Require,
    /// The path must match no headings.
    Forbid,
    /// The path must match exactly one heading.
    Unique,
    /// `before` headings must precede `after` headings.
    Order,
    /// Only `allowed` headings are permitted under the scope.
    Allowlist,
}

impl<'de> Deserialize<'de> for HeaderPathSegment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SegVisitor;

        impl<'de> serde::de::Visitor<'de> for SegVisitor {
            type Value = HeaderPathSegment;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string or `{ match = \"...\", recursive = bool }`")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(HeaderPathSegment::Exact(value.to_owned()))
            }

            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut pattern: Option<String> = None;
                let mut recursive = false;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "match" => pattern = Some(map.next_value()?),
                        "recursive" => recursive = map.next_value()?,
                        other => {
                            return Err(serde::de::Error::unknown_field(
                                other,
                                &["match", "recursive"],
                            ));
                        }
                    }
                }
                let pattern = pattern.ok_or_else(|| serde::de::Error::missing_field("match"))?;
                Ok(HeaderPathSegment::Pattern { pattern, recursive })
            }
        }

        deserializer.deserialize_any(SegVisitor)
    }
}

impl<'de> Deserialize<'de> for HeaderAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        match text.as_str() {
            "require" => Ok(Self::Require),
            "forbid" => Ok(Self::Forbid),
            "unique" => Ok(Self::Unique),
            "order" => Ok(Self::Order),
            "allowlist" => Ok(Self::Allowlist),
            other => Err(serde::de::Error::custom(format!(
                "invalid action `{other}`; expected `require`, `forbid`, `unique`, \
                 `order`, or `allowlist`"
            ))),
        }
    }
}
