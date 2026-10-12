//! The `mt init` configuration axes: one enum per axis, its choices and their
//! TOML fragments.
//!
//! This is the declaration half. The walk that asks a user over these axes —
//! and the composing of their answers — lives in [`super`]; the
//! shared contract between the two is [`super::Axis`]. The [`super::axis!`]
//! macro, textually in scope here because this module is declared after it,
//! generates each enum, its [`super::Axis`] impl and its `clap::ValueEnum`.

use super::Axis;

axis!(Workflow ("Workflow — status legend and transitions:", default = Flow) {
    TodoDone = "todo-done": "todo then done, nothing enforced"
        => include_str!("../axes/workflow_todo_done.toml"),
    Flow = "flow": "a lifecycle with enforced transitions"
        => include_str!("../axes/workflow_flow.toml"),
    Gated = "gated": "agent lifecycle with a review gate"
        => include_str!("../axes/workflow_gated.toml"),
    Personal = "personal": "inbox / next / waiting / someday"
        => include_str!("../axes/workflow_personal.toml"),
});

axis!(Priority ("Priority — how tasks are ranked:", default = Bands) {
    Off = "off": "a plain number, no bands"
        => "# priority: a plain number, no bands.\n[tasks.priority]\nbands = []\n",
    Bands = "bands": "the default low / normal / high bands"
        => "# priority: the default low / normal / high bands.\n",
    Ice = "ice": "ICE scoring, raw ints still allowed"
        => "# priority: ICE scoring (impact x confidence x ease); raw ints still allowed.\n\
            [tasks.priority]\nframework = \"ice\"\n",
    IceStrict = "ice-strict": "ICE scoring enforced on every task (no raw fallback)"
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
        => include_str!("../axes/headers_spec.toml"),
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
const SPEC_TEMPLATE: &str = include_str!("../presets/gated/templates/task.typ");

impl Headers {
    /// Templates this choice must scaffold beside the config.
    pub(super) fn templates(self) -> Vec<(String, String)> {
        match self {
            Self::Off => Vec::new(),
            Self::Spec => vec![("task.typ".to_owned(), SPEC_TEMPLATE.to_owned())],
        }
    }
}
