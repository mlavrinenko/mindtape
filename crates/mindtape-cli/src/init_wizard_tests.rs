#![allow(clippy::unwrap_used)]

use std::io::Cursor;

use super::{
    Answers, Axis, Estimate, Flags, Headers, Organisation, Priority, Recurrence, Required,
    Workflow, assemble, parse_answer_comment, resolve,
};

/// Whether this build carries the `rrule` expansion engine. Without it the
/// wizard hides `--recurrence rrule`, which leaves the axis one choice — so
/// every recurrence expectation below is a function of this, not a constant.
const RRULE_BUILD: bool = cfg!(feature = "rrule");

/// The recurrence answer this build can actually reach through the wizard.
fn reachable_recurrence() -> Recurrence {
    if RRULE_BUILD {
        Recurrence::Rrule
    } else {
        Recurrence::default_value()
    }
}

/// Drive `resolve` with a scripted answer stream, capturing the prompts.
fn walk(flags: &Flags, interactive: bool, script: &str) -> (Answers, String) {
    let mut reader = Cursor::new(script.as_bytes().to_vec());
    let mut writer = Vec::<u8>::new();
    let answers = resolve(flags, interactive, &mut reader, &mut writer).unwrap();
    (answers, String::from_utf8(writer).unwrap())
}

/// Drive `resolve` expecting a refusal, returning the message. Only a build
/// missing an engine has an unhonourable answer to refuse.
#[cfg(not(feature = "rrule"))]
fn walk_err(flags: &Flags) -> String {
    let mut reader = Cursor::new(Vec::new());
    let mut writer = Vec::<u8>::new();
    resolve(flags, true, &mut reader, &mut writer)
        .unwrap_err()
        .to_string()
}

fn all_default() -> Answers {
    Answers {
        workflow: Workflow::default_value(),
        priority: Priority::default_value(),
        organisation: Organisation::default_value(),
        estimate: Estimate::default_value(),
        required: Required::default_value(),
        headers: Headers::default_value(),
        recurrence: Recurrence::default_value(),
    }
}

/// A non-interactive run writes no prompt and takes every default.
#[test]
fn non_interactive_never_prompts() {
    let (answers, prompts) = walk(&Flags::default(), false, "");
    assert!(
        prompts.is_empty(),
        "no prompt should be written: {prompts:?}"
    );
    assert_eq!(answers, all_default());
}

/// A non-interactive run (a non-terminal stdin or `--no-input`) with flags
/// composes from the flags plus defaults, still without prompting.
#[test]
fn non_interactive_with_flags_uses_flags_silently() {
    let flags = Flags {
        workflow: Some(Workflow::Gated),
        priority: Some(Priority::Off),
        ..Flags::default()
    };
    let (answers, prompts) = walk(&flags, false, "");
    assert!(prompts.is_empty(), "{prompts:?}");
    assert_eq!(answers.workflow, Workflow::Gated);
    assert_eq!(answers.priority, Priority::Off);
    assert_eq!(answers.organisation, Organisation::default_value());
}

/// Every axis flag pre-answers its question, so an interactive walk with all of
/// them set reads nothing and prompts nothing.
#[test]
fn all_flags_suppress_every_question() {
    let flags = Flags {
        workflow: Some(Workflow::TodoDone),
        priority: Some(Priority::Ice),
        organisation: Some(Organisation::Flat),
        estimate: Some(Estimate::DaysHours),
        required: Some(Required::Difficulty),
        headers: Some(Headers::Spec),
        recurrence: Some(reachable_recurrence()),
    };
    let (answers, prompts) = walk(&flags, true, "");
    assert!(prompts.is_empty(), "all questions suppressed: {prompts:?}");
    assert_eq!(answers.workflow, Workflow::TodoDone);
    assert_eq!(answers.headers, Headers::Spec);
    assert_eq!(answers.recurrence, reachable_recurrence());
}

/// A core-axis flag suppresses exactly its own question; the other core
/// questions are still asked.
#[test]
fn core_flag_suppresses_only_its_question() {
    let flags = Flags {
        workflow: Some(Workflow::Gated),
        ..Flags::default()
    };
    // priority, organisation, then the accept-defaults gate.
    let (answers, prompts) = walk(&flags, true, "ice\nflat\n\n");
    assert!(
        !prompts.contains(Workflow::TITLE),
        "workflow was asked: {prompts}"
    );
    assert!(prompts.contains(Priority::TITLE), "{prompts}");
    assert!(prompts.contains(Organisation::TITLE), "{prompts}");
    assert!(prompts.contains("Accept defaults"), "{prompts}");
    assert_eq!(answers.workflow, Workflow::Gated);
    assert_eq!(answers.priority, Priority::Ice);
    assert_eq!(answers.organisation, Organisation::Flat);
    // extras defaulted (gate accepted).
    assert_eq!(answers.estimate, Estimate::default_value());
}

/// An extra-axis flag suppresses its question even once the extras are being
/// asked (the accept-defaults gate declined).
#[test]
fn extra_flag_suppresses_only_its_question() {
    let flags = Flags {
        estimate: Some(Estimate::DaysHours),
        ..Flags::default()
    };
    // workflow, priority, organisation, gate=no, then required, headers, recurrence.
    let (answers, prompts) = walk(&flags, true, "flow\nbands\nfolders\nn\nnone\noff\noff\n");
    assert!(
        !prompts.contains(Estimate::TITLE),
        "estimate was asked: {prompts}"
    );
    assert!(prompts.contains(Required::TITLE), "{prompts}");
    assert!(prompts.contains(Headers::TITLE), "{prompts}");
    // Recurrence is a question only while this build offers more than one
    // answer to it; without the `rrule` engine there is nothing to choose.
    assert_eq!(
        prompts.contains(Recurrence::TITLE),
        RRULE_BUILD,
        "{prompts}"
    );
    assert_eq!(answers.estimate, Estimate::DaysHours);
    assert_eq!(answers.required, Required::None);
}

/// Accepting the defaults at the gate answers the three core questions and
/// leaves the four extras at their defaults.
#[test]
fn accept_defaults_gate_skips_the_extras() {
    let (answers, prompts) = walk(&Flags::default(), true, "gated\nice\nflat\ny\n");
    assert!(!prompts.contains(Estimate::TITLE), "{prompts}");
    assert!(!prompts.contains(Recurrence::TITLE), "{prompts}");
    assert_eq!(answers.workflow, Workflow::Gated);
    assert_eq!(answers.priority, Priority::Ice);
    assert_eq!(answers.headers, Headers::default_value());
}

/// Declining the gate walks every extra too. A choice can be given by number.
#[test]
fn declining_the_gate_walks_the_extras() {
    let (answers, _) = walk(
        &Flags::default(),
        true,
        "flow\nbands\nfolders\nn\ndays-hours\ndifficulty\nspec\nrrule\n",
    );
    assert_eq!(answers.estimate, Estimate::DaysHours);
    assert_eq!(answers.required, Required::Difficulty);
    assert_eq!(answers.headers, Headers::Spec);
    assert_eq!(answers.recurrence, reachable_recurrence());
}

/// The `rrule` choice is offered only by a build that carries its expansion
/// engine — the wizard never puts a config on the menu that `mt check` would
/// then reject on the first task to use it.
#[test]
fn rrule_is_offered_only_by_an_rrule_build() {
    assert_eq!(Recurrence::Rrule.unavailable().is_none(), RRULE_BUILD);
    assert_eq!(Recurrence::offered().len(), if RRULE_BUILD { 2 } else { 1 });
    assert!(Recurrence::offered().contains(&Recurrence::Off));
    // Every other axis is untouched: unavailability defaults to "available".
    assert_eq!(Workflow::offered().len(), Workflow::VARIANTS.len());
    assert_eq!(Priority::offered().len(), Priority::VARIANTS.len());
}

/// `--recurrence rrule` on a build without the engine fails the run, naming the
/// cargo feature. Composing it instead would defer the failure to `mt check` in
/// a different session, where nothing connects it back to the answer.
#[test]
#[cfg(not(feature = "rrule"))]
fn an_unavailable_flag_is_refused_by_name_and_reason() {
    let flags = Flags {
        recurrence: Some(Recurrence::Rrule),
        ..Flags::default()
    };
    let message = walk_err(&flags);
    assert!(message.contains("rrule"), "{message}");
    assert!(message.contains("cargo feature"), "{message}");
}

/// Declining the extras gate on a build without the engine walks the other
/// extras and never writes the word `rrule` — the menu entry that sent a real
/// project down this path is simply not there.
#[test]
#[cfg(not(feature = "rrule"))]
fn a_build_without_the_engine_never_prints_the_rrule_menu_entry() {
    let (answers, prompts) = walk(
        &Flags::default(),
        true,
        "flow\nbands\nfolders\nn\noff\nnone\noff\n",
    );
    assert!(!prompts.contains(Recurrence::TITLE), "{prompts}");
    assert!(!prompts.contains("rrule"), "{prompts}");
    assert_eq!(answers.recurrence, Recurrence::Off);
}

/// Always true, as a call rather than a literal so the generated condition is
/// an expression clippy has no simplification for.
fn always() -> bool {
    true
}

// A three-choice axis with the middle one hidden. Recurrence has two choices,
// so hiding one leaves a question the walk skips entirely — this is the shape
// that proves what the menu does when a hidden choice sits between offered
// ones, which no real axis can exercise yet.
// The title and blurbs deliberately avoid the word `hidden`, so the assertions
// below can look for it in the rendered menu and mean the slug alone.
axis!(Probe ("Probe — a middle choice this build cannot honour:", default = First) {
    First = "first": "offered" => "",
    Hidden = "hidden": "unavailable" => "",
    Last = "last": "offered" => "",
}
unavailable {
    Hidden when always() => "the probe axis hides this one unconditionally",
});

/// Drive one axis's menu with a scripted answer, returning the choice and the
/// prompt text.
fn ask<A: Axis>(script: &str) -> (A, String) {
    let mut reader = Cursor::new(script.as_bytes().to_vec());
    let mut writer = Vec::<u8>::new();
    let chosen = super::ask_axis(&mut reader, &mut writer).unwrap();
    (chosen, String::from_utf8(writer).unwrap())
}

/// A hidden choice is absent from the menu and the numbering closes over it,
/// so `2` is the second choice SHOWN, not the second variant declared.
#[test]
fn the_menu_numbers_the_offered_choices_not_the_declared_ones() {
    let (chosen, prompts) = ask::<Probe>("2\n");
    assert!(!prompts.contains("hidden"), "{prompts}");
    assert!(prompts.contains("1) first"), "{prompts}");
    assert!(prompts.contains("2) last"), "{prompts}");
    assert_eq!(chosen, Probe::Last);
}

/// The slug of a hidden choice is refused like any unrecognised word — hiding
/// it from the menu but honouring it when typed would be the same bug wearing
/// a different hat.
#[test]
fn a_hidden_slug_typed_at_the_prompt_re_prompts() {
    let (chosen, prompts) = ask::<Probe>("hidden\nlast\n");
    assert!(prompts.contains("not one of the choices"), "{prompts}");
    assert_eq!(chosen, Probe::Last);
}

/// An empty line takes the default; an unrecognised line re-prompts.
#[test]
fn empty_takes_default_and_garbage_reprompts() {
    // workflow: garbage then empty (=> default Flow); priority empty; org empty; gate empty.
    let (answers, prompts) = walk(&Flags::default(), true, "nonsense\n\n\n\n\n");
    assert!(prompts.contains("not one of the choices"), "{prompts}");
    assert_eq!(answers.workflow, Workflow::default_value());
    assert_eq!(answers.priority, Priority::default_value());
}

/// The default answer set composes to a config that parses and validates, and
/// carries a real status legend plus a tasks root.
#[test]
fn default_answers_assemble_and_validate() {
    let outcome = assemble(all_default()).unwrap();
    assert!(
        outcome.config.contains("[[tasks.status]]"),
        "{}",
        outcome.config
    );
    assert!(
        outcome.config.contains("[[tasks.roots]]"),
        "{}",
        outcome.config
    );
    assert!(outcome.templates.is_empty());
    // parse_config + validate already ran inside assemble; re-confirm here.
    let config = mindtape::parse_config(&outcome.config).unwrap();
    config.validate().unwrap();
}

/// The `spec` body-headers answer ships the task template that produces its
/// required sections.
#[test]
fn spec_headers_ship_a_template() {
    let mut answers = all_default();
    answers.headers = Headers::Spec;
    let outcome = assemble(answers).unwrap();
    assert_eq!(outcome.templates.len(), 1);
    let (name, body) = outcome.templates.first().unwrap();
    assert_eq!(name, "task.typ");
    assert!(body.contains("Acceptance criteria"));
}

/// A strict-framework priority that is required composes and validates: the
/// fallback template satisfies it through the spliced framework leaf slots.
#[test]
fn strict_framework_priority_assembles() {
    let mut answers = all_default();
    answers.priority = Priority::IceStrict;
    let outcome = assemble(answers).unwrap();
    assert!(
        outcome.config.contains("allow-manual = false"),
        "{}",
        outcome.config
    );
}

/// The recorded answer comment round-trips: parsing it back yields the same
/// answers that produced it.
#[test]
fn answer_comment_round_trips() {
    let answers = Answers {
        workflow: Workflow::Gated,
        priority: Priority::Ice,
        organisation: Organisation::Flat,
        estimate: Estimate::DaysHours,
        required: Required::Difficulty,
        headers: Headers::Spec,
        recurrence: Recurrence::Rrule,
    };
    let outcome = assemble(answers).unwrap();
    let recovered = parse_answer_comment(&outcome.config).unwrap();
    assert_eq!(recovered, answers);
}

/// A config with no answer block does not round-trip to an answer set.
#[test]
fn missing_answer_block_parses_to_none() {
    assert!(parse_answer_comment("[tasks]\nroots = [\"tasks\"]\n").is_none());
}
