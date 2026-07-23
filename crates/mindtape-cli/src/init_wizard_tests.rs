#![allow(clippy::unwrap_used)]

use std::io::Cursor;

use super::{
    Answers, Axis, Estimate, Flags, Headers, Organisation, Priority, Recurrence, Required,
    Workflow, assemble, parse_answer_comment, resolve,
};

/// Drive `resolve` with a scripted answer stream, capturing the prompts.
fn walk(flags: &Flags, interactive: bool, script: &str) -> (Answers, String) {
    let mut reader = Cursor::new(script.as_bytes().to_vec());
    let mut writer = Vec::<u8>::new();
    let answers = resolve(flags, interactive, &mut reader, &mut writer).unwrap();
    (answers, String::from_utf8(writer).unwrap())
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
        recurrence: Some(Recurrence::Rrule),
    };
    let (answers, prompts) = walk(&flags, true, "");
    assert!(prompts.is_empty(), "all questions suppressed: {prompts:?}");
    assert_eq!(answers.workflow, Workflow::TodoDone);
    assert_eq!(answers.headers, Headers::Spec);
    assert_eq!(answers.recurrence, Recurrence::Rrule);
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
    assert!(prompts.contains(Recurrence::TITLE), "{prompts}");
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
    assert_eq!(answers.recurrence, Recurrence::Rrule);
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

/// A strict-framework priority that is required is refused, and the message
/// names the conflict.
#[test]
fn strict_framework_priority_is_refused() {
    let mut answers = all_default();
    answers.priority = Priority::IceStrict;
    let err = assemble(answers).unwrap_err().to_string();
    assert!(err.contains("strict framework"), "{err}");
    assert!(err.contains("allow-manual"), "{err}");
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
