//! Per-kind link-target validators: the `[[tasks.links.kinds]] validate` axis.
//!
//! Every edge kind sits on one axis — what shape its targets take. The default,
//! `local-task-path`, makes a target a path from the linking task's file to
//! another task's file, resolved against the loaded task set. Any other validator
//! declares the kind's targets to be *outside* the backlog (a commit, a pull
//! request, a build), which is what lets a task record the artifact that
//! discharges it.
//!
//! Two levels of checking live here, sharing one dispatch ([`accepts`]):
//!
//! - Config level — [`validate_link_kinds`] rejects a `validate = "regex"` kind
//!   with no sibling `regex = "…"` pattern (or an uncompilable one) at load, so
//!   a pattern-less regex kind is never silently permissive.
//! - Target level — [`Validators`] compiles each kind's pattern once per
//!   validation pass and reports a malformed target, naming the kind, the
//!   validator, and the offending value.
//!
//! Graph semantics follow from the same field rather than a second flag: see
//! [`TargetValidator::targets_tasks`], which
//! [`crate::link::kind_targets_tasks`] and the `*_kinds` helpers read to keep
//! non-task kinds out of stem resolution, cardinality, cycle detection, and
//! readiness.

use std::collections::HashMap;

use regex::Regex;
use serde::Deserialize;

use crate::config::Config;
use crate::link::{LinkKind, LinksConfig};

/// What shape a link kind's targets take — the `validate` key of
/// `[[tasks.links.kinds]]`.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum TargetValidator {
    /// A path from the linking task's file to another task's file (the
    /// default, and the only validator whose edges join the task graph).
    #[default]
    LocalTaskPath,
    /// An RFC 3986 URI: a scheme, then only URI-legal characters.
    Url,
    /// A hexadecimal git object name, 7 to 64 characters — abbreviated SHA-1
    /// through full SHA-256.
    GitSha,
    /// A UUID, hyphenated, as 32 bare hex digits, in braces, or with a
    /// `urn:uuid:` prefix.
    Uuid,
    /// A project-supplied pattern, taken from the kind's sibling `regex` key.
    Regex,
    /// Any non-empty text, with no shape check.
    Any,
}

impl TargetValidator {
    /// Whether this kind's targets name local task files. `true` only for
    /// [`Self::LocalTaskPath`]: every other validator describes something
    /// outside the backlog, so its edges are excluded from stem resolution,
    /// cardinality, cycle detection, and readiness.
    #[must_use]
    pub fn targets_tasks(self) -> bool {
        matches!(self, Self::LocalTaskPath)
    }

    /// The validator's config spelling, as it appears in a diagnostic.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::LocalTaskPath => "local-task-path",
            Self::Url => "url",
            Self::GitSha => "git-sha",
            Self::Uuid => "uuid",
            Self::Regex => "regex",
            Self::Any => "any",
        }
    }
}

/// Config-level problems with the declared kinds' validators, one message per
/// offending kind. Folded into [`Config::validate`], so a malformed validator
/// declaration fails at load rather than silently accepting every target.
pub(crate) fn validate_link_kinds(config: &Config) -> Vec<String> {
    config
        .tasks
        .links
        .kinds
        .iter()
        .filter_map(kind_problem)
        .collect()
}

/// The one problem, if any, with `kind`'s validator declaration.
fn kind_problem(kind: &LinkKind) -> Option<String> {
    let label = format!("tasks.links.kinds.{}", kind.name);
    match (kind.validate, kind.regex.as_deref()) {
        (TargetValidator::Regex, None) => Some(format!(
            "{label}: `validate = \"regex\"` needs a sibling `regex = \"...\"` pattern"
        )),
        (TargetValidator::Regex, Some(pattern)) => Regex::new(pattern)
            .err()
            .map(|err| format!("{label}: invalid `regex` pattern `{pattern}`: {err}")),
        (other, Some(_)) => Some(format!(
            "{label}: `regex = \"...\"` applies only to `validate = \"regex\"`, not `{}`",
            other.name()
        )),
        (_, None) => None,
    }
}

/// One kind's compiled validator: the validator itself plus its pattern, when
/// it has one. A pattern that failed to compile stays `None` — [`accepts`] then
/// rejects every target rather than waving them through.
struct Compiled {
    validator: TargetValidator,
    pattern: Option<Regex>,
}

/// Per-kind target validators for one project, compiled once per validation
/// pass. Holds only the kinds that do *not* target tasks: a `local-task-path`
/// kind's real check is resolution against the loaded task set, which the
/// caller runs instead.
pub(crate) struct Validators<'a> {
    by_kind: HashMap<&'a str, Compiled>,
}

impl<'a> Validators<'a> {
    /// Compile every non-task kind's validator from `config`.
    pub(crate) fn build(config: &'a LinksConfig) -> Self {
        let by_kind = config
            .kinds
            .iter()
            .filter(|kind| !kind.validate.targets_tasks())
            .map(|kind| {
                let pattern = kind
                    .regex
                    .as_deref()
                    .and_then(|pattern| Regex::new(pattern).ok());
                (
                    kind.name.as_str(),
                    Compiled {
                        validator: kind.validate,
                        pattern,
                    },
                )
            })
            .collect();
        Self { by_kind }
    }

    /// The violation message for a malformed `target` under `kind`'s validator,
    /// or `None` when the target is well-formed (or `kind` targets tasks, whose
    /// check is resolution, not shape).
    pub(crate) fn problem(&self, kind: &str, target: &str) -> Option<String> {
        let compiled = self.by_kind.get(kind)?;
        if accepts(compiled.validator, target, compiled.pattern.as_ref()) {
            return None;
        }
        Some(format!(
            "`{kind}` edge target `{target}` is not a valid `{}`",
            compiled.validator.name()
        ))
    }
}

/// The single dispatch every bundled validator goes through: does `value` have
/// the shape `validator` demands? `pattern` carries the compiled sibling
/// pattern for [`TargetValidator::Regex`] and is unused otherwise.
fn accepts(validator: TargetValidator, value: &str, pattern: Option<&Regex>) -> bool {
    match validator {
        // Resolution against the loaded task set is this validator's real
        // check; a shape check here would double-report it.
        TargetValidator::LocalTaskPath => true,
        TargetValidator::Any => !value.is_empty(),
        TargetValidator::Url => is_uri(value),
        TargetValidator::GitSha => is_git_sha(value),
        TargetValidator::Uuid => uuid::Uuid::parse_str(value).is_ok(),
        TargetValidator::Regex => pattern.is_some_and(|re| re.is_match(value)),
    }
}

/// Whether `value` has RFC 3986 absolute-URI shape: a scheme, a `:`, and a
/// non-empty remainder built only from URI-legal characters. Deliberately
/// syntactic — it neither resolves the target nor validates percent-triplets.
fn is_uri(value: &str) -> bool {
    let Some((scheme, rest)) = value.split_once(':') else {
        return false;
    };
    is_scheme(scheme) && !rest.is_empty() && rest.chars().all(is_uri_char)
}

/// RFC 3986 `scheme`: `ALPHA *( ALPHA / DIGIT / "+" / "-" / "." )`.
fn is_scheme(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

/// RFC 3986 `unreserved` / `reserved` / the `%` of a percent-encoding.
fn is_uri_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
        || matches!(
            c,
            '-' | '.'
                | '_'
                | '~'
                | ':'
                | '/'
                | '?'
                | '#'
                | '['
                | ']'
                | '@'
                | '!'
                | '$'
                | '&'
                | '\''
                | '('
                | ')'
                | '*'
                | '+'
                | ','
                | ';'
                | '='
                | '%'
        )
}

/// Whether `value` is a hexadecimal git object name: 7 to 64 characters, which
/// spans an abbreviated SHA-1 through a full SHA-256 digest.
fn is_git_sha(value: &str) -> bool {
    (7..=64).contains(&value.len()) && value.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
#[path = "links_validate_tests.rs"]
mod tests;
