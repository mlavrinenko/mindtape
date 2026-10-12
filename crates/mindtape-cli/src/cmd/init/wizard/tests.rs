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

/// Every build carries the `rrule` expansion engine, so the recurrence axis
/// offers each of its choices.
#[test]
fn the_recurrence_axis_offers_rrule() {
    assert_eq!(Recurrence::Rrule.unavailable(), None);
    assert_eq!(Recurrence::offered(), Recurrence::VARIANTS);
    assert_eq!(Workflow::offered().len(), Workflow::VARIANTS.len());
    assert_eq!(Priority::offered().len(), Priority::VARIANTS.len());
}

/// Declining the extras gate asks the recurrence question, with `rrule` listed
/// on its menu and picked by its number.
#[test]
fn declining_the_gate_lists_the_rrule_menu_entry() {
    let (answers, prompts) = walk(
        &Flags::default(),
        true,
        "flow\nbands\nfolders\nn\noff\nnone\noff\n2\n",
    );
    assert!(prompts.contains(Recurrence::TITLE), "{prompts}");
    assert!(prompts.contains("2) rrule"), "{prompts}");
    assert_eq!(answers.recurrence, Recurrence::Rrule);
}

/// Always true, as a call rather than a literal so the generated condition is
/// an expression clippy has no simplification for.
fn always() -> bool {
    true
}

// A three-choice axis with the middle one hidden. No real axis hides a choice,
// so this probe is where the menu and the flag meet one: a hidden choice
// between offered ones.
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

/// A flag naming a choice this build cannot honour fails the run, naming the
/// slug and the reason. Composing it instead would defer the failure to
/// `mt check` in a later session, where nothing connects it back to the answer.
#[test]
fn an_unavailable_flag_is_refused_by_name_and_reason() {
    let mut reader = Cursor::new(Vec::new());
    let mut writer = Vec::<u8>::new();
    let refusal = super::choose(Some(Probe::Hidden), true, &mut reader, &mut writer).unwrap_err();
    let message = crate::diag::plain(&refusal);
    assert!(message.contains("'hidden'"), "{message}");
    assert!(
        message.contains("hides this one unconditionally"),
        "{message}"
    );
    assert!(writer.is_empty(), "a refused flag prompts nothing");
}

// A two-choice axis with one hidden: the shape an axis takes when this build
// can honour only its default.
axis!(Lone ("Lone — one choice this build can honour:", default = Kept) {
    Kept = "kept": "offered" => "",
    Gone = "gone": "unavailable" => "",
}
unavailable {
    Gone when always() => "the lone axis hides this one unconditionally",
});

/// An axis left with one offered choice is no question: it resolves to the
/// default and prompts nothing.
#[test]
fn an_axis_with_one_offered_choice_is_not_asked() {
    let (chosen, prompts) = ask::<Lone>("gone\n");
    assert_eq!(chosen, Lone::Kept);
    assert!(prompts.is_empty(), "{prompts}");
}

/// Input that ends at the first prompt takes every default: each axis and the
/// extras gate read EOF and fall back rather than loop.
#[test]
fn eof_at_every_prompt_takes_the_defaults() {
    let (answers, prompts) = walk(&Flags::default(), true, "");
    assert!(prompts.contains(Workflow::TITLE), "{prompts}");
    assert!(prompts.contains("Accept defaults"), "{prompts}");
    assert!(!prompts.contains(Estimate::TITLE), "{prompts}");
    assert_eq!(answers, all_default());
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

/// Any single pre-answered axis counts: `any` is a disjunction, so a flag on
/// only the fourth-through-seventh fields must still report true.
#[test]
fn any_is_true_for_each_single_flag() {
    let cases = [
        Flags {
            workflow: Some(Workflow::Gated),
            ..Flags::default()
        },
        Flags {
            priority: Some(Priority::Ice),
            ..Flags::default()
        },
        Flags {
            organisation: Some(Organisation::Flat),
            ..Flags::default()
        },
        Flags {
            estimate: Some(Estimate::DaysHours),
            ..Flags::default()
        },
        Flags {
            required: Some(Required::Difficulty),
            ..Flags::default()
        },
        Flags {
            headers: Some(Headers::Spec),
            ..Flags::default()
        },
        Flags {
            recurrence: Some(Recurrence::Off),
            ..Flags::default()
        },
    ];
    for flags in cases {
        assert!(flags.any(), "{flags:?} must count as some flag");
    }
    assert!(!Flags::default().any());
}

/// The extras are skipped only when *all four* are pre-answered. Three flags
/// with the fourth missing must still offer the accept-defaults gate; reading
/// the headers line as `||` would treat the three as complete.
#[test]
fn three_of_four_extra_flags_still_offer_the_gate() {
    let flags = Flags {
        estimate: Some(Estimate::DaysHours),
        required: Some(Required::Difficulty),
        headers: Some(Headers::Spec),
        ..Flags::default()
    };
    // workflow, priority, organisation, then accept the gate.
    let (_, prompts) = walk(&flags, true, "flow\nbands\nfolders\ny\n");
    assert!(prompts.contains("Accept defaults"), "{prompts}");
}

/// A lone recurrence flag does not complete the extras either: the gate is
/// still offered. The concluding `||` would let one flag stand in for all
/// four.
#[test]
fn a_lone_recurrence_flag_still_offers_the_gate() {
    let flags = Flags {
        recurrence: Some(Recurrence::Off),
        ..Flags::default()
    };
    let (_, prompts) = walk(&flags, true, "flow\nbands\nfolders\ny\n");
    assert!(prompts.contains("Accept defaults"), "{prompts}");
}

/// The default choice is the one the menu marks; the marker is not on every
/// other choice instead. Inverting the equality would mark `last`.
#[test]
fn the_menu_marks_the_default_choice() {
    let (_chosen, prompts) = ask::<Probe>("1\n");
    let marked: Vec<&str> = prompts
        .lines()
        .filter(|line| line.contains("[default]"))
        .collect();
    assert_eq!(marked.len(), 1, "exactly one default marker: {prompts}");
    assert!(
        marked.iter().any(|line| line.contains("first")),
        "the default choice must carry the marker: {prompts}"
    );
}
