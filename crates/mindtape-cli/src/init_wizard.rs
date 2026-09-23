//! The `mt init` questionnaire over the configuration axes.
//!
//! Someone who does not yet know which named preset they want answers a short
//! walk of questions instead. Each axis is one question and one TOML fragment;
//! the answers compose (via [`crate::init_compose`]) into one flattened
//! `.mindtape/config.toml`. A preset is just a saved answer set over the same
//! axes — one model, two entry points.
//!
//! The walk only runs on an interactive terminal with no `--preset` and no
//! `--no-input`; a per-axis flag pre-answers (and so suppresses) its question,
//! and a non-terminal run with any flag composes without prompting at all. See
//! [`resolve`] for the gating and [`crate::init::run`] for the wiring.

use std::io::{BufRead, Write};

use anyhow::{Result, anyhow};

/// One configuration axis: a menu of choices, each carrying its config fragment.
///
/// Implemented once per axis by the [`axis!`] macro, which also derives the
/// `clap::ValueEnum` that backs the axis's `mt init` flag.
trait Axis: Copy + Sized + PartialEq + 'static {
    /// The question title shown above the choices.
    const TITLE: &'static str;
    /// Every choice, in menu order.
    const VARIANTS: &'static [Self];
    /// The kebab-case slug (matches the flag value and the answer comment).
    fn slug(self) -> &'static str;
    /// One-line description shown beside the choice.
    fn blurb(self) -> &'static str;
    /// The TOML fragment this choice contributes (may be comment-only).
    fn fragment(self) -> &'static str;
    /// The choice used when the question is skipped.
    fn default_value() -> Self;
    /// `Some(reason)` when THIS build cannot act on the choice's config. Such a
    /// choice is neither listed in the menu nor accepted from a flag: a wizard
    /// is a promise that the composed config runs, and a config the binary
    /// cannot honour fails later and elsewhere, where the user cannot connect
    /// it back to the answer that caused it.
    fn unavailable(self) -> Option<&'static str> {
        None
    }
    /// Resolve a slug back to its choice.
    fn from_slug(slug: &str) -> Option<Self> {
        Self::VARIANTS
            .iter()
            .copied()
            .find(|value| value.slug() == slug)
    }
    /// The choices this build offers, in menu order.
    fn offered() -> Vec<Self> {
        Self::VARIANTS
            .iter()
            .copied()
            .filter(|value| value.unavailable().is_none())
            .collect()
    }
}

/// Define an axis enum, its [`Axis`] impl, and its `clap::ValueEnum` derivation.
///
/// The optional trailing `unavailable { … }` block declares, per choice, a
/// condition under which this build cannot honour it and the reason to report.
/// The variant stays compiled either way, so `--<axis> <slug>` keeps parsing and
/// keeps showing in `--help`; only the answer is refused, with the reason.
macro_rules! axis {
    (
        $name:ident ($title:literal, default = $default:ident) {
            $( $variant:ident = $slug:literal : $blurb:literal => $frag:expr ),+ $(,)?
        }
        $( unavailable { $( $off:ident when $cond:expr => $why:literal ),+ $(,)? } )?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
        pub enum $name { $( #[value(name = $slug)] $variant ),+ }

        impl Axis for $name {
            const TITLE: &'static str = $title;
            const VARIANTS: &'static [Self] = &[ $( Self::$variant ),+ ];
            fn slug(self) -> &'static str { match self { $( Self::$variant => $slug ),+ } }
            fn blurb(self) -> &'static str { match self { $( Self::$variant => $blurb ),+ } }
            fn fragment(self) -> &'static str { match self { $( Self::$variant => $frag ),+ } }
            fn default_value() -> Self { Self::$default }
            $( fn unavailable(self) -> Option<&'static str> {
                $( if self == Self::$off && ($cond) { return Some($why); } )+
                None
            } )?
        }
    };
}

#[path = "init_axes.rs"]
mod axes;

pub use axes::{Estimate, Headers, Organisation, Priority, Recurrence, Required, Workflow};

/// Per-axis flag values from `mt init`. A `Some` pre-answers (and suppresses)
/// that axis's question.
#[derive(Debug, Default)]
pub struct Flags {
    pub workflow: Option<Workflow>,
    pub priority: Option<Priority>,
    pub organisation: Option<Organisation>,
    pub estimate: Option<Estimate>,
    pub required: Option<Required>,
    pub headers: Option<Headers>,
    pub recurrence: Option<Recurrence>,
}

impl Flags {
    /// Whether any axis was pre-answered by a flag.
    #[must_use]
    pub fn any(&self) -> bool {
        self.workflow.is_some()
            || self.priority.is_some()
            || self.organisation.is_some()
            || self.estimate.is_some()
            || self.required.is_some()
            || self.headers.is_some()
            || self.recurrence.is_some()
    }
}

/// One resolved answer per axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Answers {
    pub workflow: Workflow,
    pub priority: Priority,
    pub organisation: Organisation,
    pub estimate: Estimate,
    pub required: Required,
    pub headers: Headers,
    pub recurrence: Recurrence,
}

/// A composed configuration plus any templates its answers require.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// The flattened `.mindtape/config.toml` body (no schema directive yet).
    pub config: String,
    /// `mt add` templates to drop beside it, as `(file name, content)`.
    pub templates: Vec<(String, String)>,
}

/// Resolve one answer per axis.
///
/// A flag pre-answers its axis and is never prompted. The three core axes
/// (workflow, priority, organisation) are asked first; the four extras hide
/// behind one "accept defaults for the rest?" gate so the walk is never a wall
/// of seven prompts. `interactive` is false for a non-terminal or `--no-input`
/// run, where every un-flagged axis takes its default without a prompt. An axis
/// this build has reduced to one honourable choice ([`Axis::unavailable`]) is
/// not a question either, and is never prompted.
///
/// # Errors
/// Propagates IO errors from reading the answers or writing the prompts, and
/// refuses a flag naming a choice this build cannot honour.
pub fn resolve<R: BufRead, W: Write>(
    flags: &Flags,
    interactive: bool,
    reader: &mut R,
    writer: &mut W,
) -> Result<Answers> {
    let workflow = choose(flags.workflow, interactive, reader, writer)?;
    let priority = choose(flags.priority, interactive, reader, writer)?;
    let organisation = choose(flags.organisation, interactive, reader, writer)?;

    let extras_flagged = flags.estimate.is_some()
        && flags.required.is_some()
        && flags.headers.is_some()
        && flags.recurrence.is_some();
    let ask_extras = interactive && !extras_flagged && !accept_defaults(reader, writer)?;

    let estimate = choose(flags.estimate, ask_extras, reader, writer)?;
    let required = choose(flags.required, ask_extras, reader, writer)?;
    let headers = choose(flags.headers, ask_extras, reader, writer)?;
    let recurrence = choose(flags.recurrence, ask_extras, reader, writer)?;

    Ok(Answers {
        workflow,
        priority,
        organisation,
        estimate,
        required,
        headers,
        recurrence,
    })
}

/// A flag wins outright (suppressing the question) unless this build cannot
/// honour it, which fails the run rather than composing a config that breaks
/// later; otherwise ask when `ask`, else take the default.
///
/// # Errors
/// Returns an error when the flagged choice is [`Axis::unavailable`].
fn choose<A: Axis, R: BufRead, W: Write>(
    flag: Option<A>,
    ask: bool,
    reader: &mut R,
    writer: &mut W,
) -> Result<A> {
    if let Some(value) = flag {
        return match value.unavailable() {
            Some(why) => Err(anyhow!("`{}` is unavailable: {why}", value.slug())),
            None => Ok(value),
        };
    }
    if ask {
        return ask_axis(reader, writer);
    }
    Ok(A::default_value())
}

/// Render one axis's menu, read a choice (number, slug, or empty for the
/// default), and re-prompt on anything unrecognised. EOF resolves to the
/// default rather than looping.
///
/// Only the choices this build can honour are listed, and they are numbered as
/// listed — a hidden choice leaves no gap in the menu. An axis left with one
/// choice is not a question, so it resolves to the default unasked.
fn ask_axis<A: Axis, R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> Result<A> {
    let offered = A::offered();
    if offered.len() < 2 {
        return Ok(A::default_value());
    }
    loop {
        writeln!(writer, "\n{}", A::TITLE)?;
        for (index, variant) in offered.iter().enumerate() {
            let marker = if *variant == A::default_value() {
                " [default]"
            } else {
                ""
            };
            writeln!(
                writer,
                "  {}) {:<11} {}{marker}",
                index + 1,
                variant.slug(),
                variant.blurb(),
            )?;
        }
        write!(writer, "> ")?;
        writer.flush()?;

        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(A::default_value());
        }
        let choice = line.trim();
        if choice.is_empty() {
            return Ok(A::default_value());
        }
        if let Ok(index) = choice.parse::<usize>()
            && let Some(variant) = index.checked_sub(1).and_then(|i| offered.get(i))
        {
            return Ok(*variant);
        }
        // Filtered, not matched separately: an axis whose only unavailable
        // choice is hidden is left with one choice and never reaches this loop,
        // so a dedicated "unavailable" message here would be unreachable. The
        // reason is reported where a user can actually ask for it — the flag.
        if let Some(variant) = A::from_slug(choice).filter(|v| v.unavailable().is_none()) {
            return Ok(variant);
        }
        writeln!(
            writer,
            "  '{choice}' is not one of the choices — pick a number or a name",
        )?;
    }
}

/// The gate between the core questions and the extras. Empty or yes accepts the
/// defaults; EOF accepts them too.
fn accept_defaults<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> Result<bool> {
    write!(
        writer,
        "\nAccept defaults for the rest (estimate, required fields, headers, recurrence)? [Y/n] ",
    )?;
    writer.flush()?;
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 {
        return Ok(true);
    }
    let answer = line.trim().to_ascii_lowercase();
    Ok(answer.is_empty() || answer == "y" || answer == "yes")
}

/// Compose the answers into one config body, verify it, and collect any
/// templates the answers require.
///
/// # Errors
/// Returns an error when the fragments do not compose into a valid config or
/// when the composed config fails validation.
pub fn assemble(answers: Answers) -> Result<Outcome> {
    let comment = render_answer_comment(answers);
    let fragments: Vec<(&str, String)> = vec![
        ("answers", comment),
        ("workflow", answers.workflow.fragment().to_owned()),
        ("priority", answers.priority.fragment().to_owned()),
        ("organisation", answers.organisation.fragment().to_owned()),
        ("estimate", answers.estimate.fragment().to_owned()),
        ("required", answers.required.fragment().to_owned()),
        ("headers", answers.headers.fragment().to_owned()),
        ("recurrence", answers.recurrence.fragment().to_owned()),
    ];
    let config = crate::init_compose::compose(&fragments)?;
    let parsed = mindtape::parse_config(&config)
        .map_err(|err| anyhow!("the composed config is not valid: {err}"))?;
    parsed
        .validate()
        .map_err(|msg| anyhow!("the composed config fails validation: {msg}"))?;
    Ok(Outcome {
        config,
        templates: answers.headers.templates(),
    })
}

/// The recorded answer block: documents how the project was made and lets
/// `preset-save` read the answers back ([`parse_answer_comment`]).
fn render_answer_comment(answers: Answers) -> String {
    format!(
        "# Composed by the `mt init` wizard; this block records the axis answers.\n\
         # answers:\n\
         #   workflow = {}\n\
         #   priority = {}\n\
         #   organisation = {}\n\
         #   estimate = {}\n\
         #   required = {}\n\
         #   headers = {}\n\
         #   recurrence = {}\n",
        answers.workflow.slug(),
        answers.priority.slug(),
        answers.organisation.slug(),
        answers.estimate.slug(),
        answers.required.slug(),
        answers.headers.slug(),
        answers.recurrence.slug(),
    )
}

/// Recover the answers from a recorded comment block, the inverse of
/// [`render_answer_comment`]. A comment line reads `#   <axis> = <slug>`; lines
/// whose value is not a valid slug for that axis are ignored, so an axis
/// fragment's own commentary cannot spoof an answer. Returns `None` unless every
/// axis is present.
///
/// The reader half of the recorded answer block: the sibling task `preset-save`
/// reads a scaffolded project's answers back through this. Only the tests
/// exercise it today, hence the allow.
#[must_use]
#[allow(dead_code)]
pub fn parse_answer_comment(text: &str) -> Option<Answers> {
    let mut workflow = None;
    let mut priority = None;
    let mut organisation = None;
    let mut estimate = None;
    let mut required = None;
    let mut headers = None;
    let mut recurrence = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.trim_start_matches('#').trim().split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        match key {
            "workflow" => workflow = workflow.or_else(|| Workflow::from_slug(value)),
            "priority" => priority = priority.or_else(|| Priority::from_slug(value)),
            "organisation" => {
                organisation = organisation.or_else(|| Organisation::from_slug(value));
            }
            "estimate" => estimate = estimate.or_else(|| Estimate::from_slug(value)),
            "required" => required = required.or_else(|| Required::from_slug(value)),
            "headers" => headers = headers.or_else(|| Headers::from_slug(value)),
            "recurrence" => recurrence = recurrence.or_else(|| Recurrence::from_slug(value)),
            _ => {}
        }
    }
    Some(Answers {
        workflow: workflow?,
        priority: priority?,
        organisation: organisation?,
        estimate: estimate?,
        required: required?,
        headers: headers?,
        recurrence: recurrence?,
    })
}

#[cfg(test)]
#[path = "init_wizard_tests.rs"]
mod tests;
