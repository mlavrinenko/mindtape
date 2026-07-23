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

use anyhow::{Result, anyhow, bail};

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
    /// Resolve a slug back to its choice.
    fn from_slug(slug: &str) -> Option<Self> {
        Self::VARIANTS
            .iter()
            .copied()
            .find(|value| value.slug() == slug)
    }
}

/// Define an axis enum, its [`Axis`] impl, and its `clap::ValueEnum` derivation.
macro_rules! axis {
    (
        $name:ident ($title:literal, default = $default:ident) {
            $( $variant:ident = $slug:literal : $blurb:literal => $frag:expr ),+ $(,)?
        }
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
        }
    };
}

axis!(Workflow ("Workflow — status legend and transitions:", default = Flow) {
    TodoDone = "todo-done": "todo then done, nothing enforced"
        => include_str!("axes/workflow_todo_done.toml"),
    Flow = "flow": "a lifecycle with enforced transitions"
        => include_str!("axes/workflow_flow.toml"),
    Gated = "gated": "agent lifecycle with a review gate"
        => include_str!("axes/workflow_gated.toml"),
    Personal = "personal": "inbox / next / waiting / someday"
        => include_str!("axes/workflow_personal.toml"),
});

axis!(Priority ("Priority — how tasks are ranked:", default = Bands) {
    Off = "off": "a plain number, no bands"
        => "# priority: a plain number, no bands.\n[tasks.priority]\nbands = []\n",
    Bands = "bands": "the default low / normal / high bands"
        => "# priority: the default low / normal / high bands.\n",
    Ice = "ice": "ICE scoring, raw ints still allowed"
        => "# priority: ICE scoring (impact x confidence x ease); raw ints still allowed.\n\
            [tasks.priority]\nframework = \"ice\"\n",
    IceStrict = "ice-strict": "ICE scoring enforced on every task (no template ships it)"
        => "# priority: ICE scoring enforced on every task, no raw fallback.\n\
            [tasks.priority]\nframework = \"ice\"\nallow-manual = false\nrequired = true\n",
});

axis!(Organisation ("Organisation — how tasks map to folders:", default = Folders) {
    Flat = "flat": "one flat tasks/ root"
        => "# organisation: one flat tasks/ root, no folder-derived tags.\n\
            [[tasks.roots]]\npath = \"tasks\"\n",
    Folders = "folders": "folders under tasks/ become tags"
        => "# organisation: folders under tasks/ become tags automatically.\n\
            [[tasks.roots]]\npath = \"tasks\"\nderive-tags = true\n\n\
            [tasks.derive-tags]\nenabled = true\ncollect = [\"segments\"]\n",
});

axis!(Estimate ("Estimate — units a task may book:", default = Off) {
    Off = "off": "free-form, no unit allow-list"
        => "# estimate: free-form, no unit allow-list.\n",
    DaysHours = "days-hours": "days and hours"
        => "# estimate: days and hours.\n[tasks.estimate]\nunits = [\"days\", \"hours\"]\n",
});

axis!(Required ("Required fields — what every open task must carry:", default = None) {
    None = "none": "nothing beyond the defaults"
        => "# required fields: nothing beyond the defaults.\n",
    Difficulty = "difficulty": "difficulty on every open task (the routing signal)"
        => "# required fields: difficulty on every open task.\n\
            [tasks.difficulty]\nrequired = \"!terminal\"\n",
});

axis!(Headers ("Body headers — required sections in a task body:", default = Off) {
    Off = "off": "no required sections"
        => "# body headers: no required sections.\n",
    Spec = "spec": "Summary / Acceptance criteria / Non-goals / Files"
        => include_str!("axes/headers_spec.toml"),
});

axis!(Recurrence ("Recurrence — how recurring rules are written:", default = Off) {
    Off = "off": "the built-in every(...) rules only"
        => "# recurrence: the built-in every(...) rules only.\n",
    Rrule = "rrule": "also accept rrule(\"FREQ=...\") strings"
        => "# recurrence: also accept RFC-5545 rrule(\"FREQ=...\") strings.\n\
            [tasks.recur]\nrrule = true\n",
});

/// The `spec` body-headers choice needs a template that produces its sections,
/// or the very first `mt add` writes a task its own config rejects. Ship the
/// same task template the `gated` preset does.
const SPEC_TEMPLATE: &str = include_str!("presets/gated/templates/task.typ");

impl Headers {
    /// Templates this choice must scaffold beside the config.
    fn templates(self) -> Vec<(String, String)> {
        match self {
            Self::Off => Vec::new(),
            Self::Spec => vec![("task.typ".to_owned(), SPEC_TEMPLATE.to_owned())],
        }
    }
}

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
/// run, where every un-flagged axis takes its default without a prompt.
///
/// # Errors
/// Propagates IO errors from reading the answers or writing the prompts.
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

/// A flag wins outright (suppressing the question); otherwise ask when `ask`,
/// else take the default.
fn choose<A: Axis, R: BufRead, W: Write>(
    flag: Option<A>,
    ask: bool,
    reader: &mut R,
    writer: &mut W,
) -> Result<A> {
    if let Some(value) = flag {
        return Ok(value);
    }
    if ask {
        return ask_axis(reader, writer);
    }
    Ok(A::default_value())
}

/// Render one axis's menu, read a choice (number, slug, or empty for the
/// default), and re-prompt on anything unrecognised. EOF resolves to the
/// default rather than looping.
fn ask_axis<A: Axis, R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> Result<A> {
    loop {
        writeln!(writer, "\n{}", A::TITLE)?;
        for (index, variant) in A::VARIANTS.iter().enumerate() {
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
            && let Some(variant) = index.checked_sub(1).and_then(|i| A::VARIANTS.get(i))
        {
            return Ok(*variant);
        }
        if let Some(variant) = A::from_slug(choice) {
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
/// Returns an error when the fragments do not compose into a valid config, when
/// the composed config fails validation, or when the answer set is incoherent
/// (see [`coherence`]).
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
    coherence(&parsed)?;
    parsed
        .validate()
        .map_err(|msg| anyhow!("the composed config fails validation: {msg}"))?;
    Ok(Outcome {
        config,
        templates: answers.headers.templates(),
    })
}

/// Refuse an answer set whose config the scaffolded template cannot satisfy.
///
/// The concrete case is a strict-framework priority: a `framework` with
/// `allow-manual = false` has no scalar form, so `mindtape::required_core_slots`
/// cannot splice it into the barebones template — the project would scaffold
/// cleanly and then fail on its first `mt add`. Caught here, it fails init with
/// a message that names the conflict instead.
fn coherence(config: &mindtape::Config) -> Result<()> {
    let priority = &config.tasks.priority;
    if priority.field.required.may_apply() && priority.framework.is_some() && !priority.allow_manual
    {
        bail!(
            "incoherent answer set: priority uses the strict framework {:?} \
             (allow-manual = false) and is required, but a strict framework has no \
             scalar form, so the scaffolded template cannot supply it. Choose a \
             non-strict priority (--priority ice), make priority optional, or start \
             from a preset that ships a matching template (mt init --preset gated).",
            priority.framework.as_deref().unwrap_or("?"),
        );
    }
    Ok(())
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
