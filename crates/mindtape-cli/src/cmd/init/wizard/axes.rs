//! The `mt init` configuration axes: one enum per axis, its choices and their
//! TOML fragments.
//!
//! This is the declaration half. The walk that asks a user over these axes —
//! and the composing of their answers — lives in [`super`]; the
//! shared contract between the two is [`super::Axis`]. The [`super::axis!`]
//! macro, textually in scope here because this module is declared after it,
//! generates each enum, its [`super::Axis`] impl and its `clap::ValueEnum`.

use super::Axis;

axis!(Workflow ("Which statuses do you want?", default = Flow) {
    TodoDone = "todo-done": "todo and done, any move allowed"
        => include_str!("../axes/workflow_todo_done.toml"),
    Flow = "flow": "proposed, wip, blocked, done, cancelled, with fixed moves"
        => include_str!("../axes/workflow_flow.toml"),
    Gated = "gated": "made for agents, with a review step before done"
        => include_str!("../axes/workflow_gated.toml"),
    Personal = "personal": "inbox, next, waiting, someday"
        => include_str!("../axes/workflow_personal.toml"),
});

axis!(Priority ("How should tasks be ranked?", default = Bands) {
    Off = "off": "a plain number"
        => "# priority: a plain number, no bands.\n[tasks.priority]\nbands = []\n",
    Bands = "bands": "low, normal or high"
        => "# priority: the default low / normal / high bands.\n",
    Ice = "ice": "an ICE score, or a plain number"
        => "# priority: ICE scoring (impact x confidence x ease); raw ints still allowed.\n\
            [tasks.priority]\nframework = \"ice\"\n",
    IceStrict = "ice-strict": "an ICE score on every open task, no plain numbers"
        => "# priority: ICE scoring on every open task, no raw fallback, from the\n\
            # mindtape:ice pack in `extends`: https://mlavrinenko.github.io/mindtape/packs/\n\
            extends = [\"mindtape:ice\"]\n",
});

axis!(Organization ("How do you want to organize tasks?", default = Folders) {
    Flat = "flat": "all in one tasks/ folder"
        => "# organization: one flat tasks/ root, no folder-derived tags.\n\
            [[tasks.roots]]\npath = \"tasks\"\n",
    Folders = "folders": "in folders under tasks/, each folder a tag"
        => "# organization: folders under tasks/ become tags automatically.\n\
            [[tasks.roots]]\npath = \"tasks\"\nderive-tags = true\n\n\
            [tasks.derive-tags]\nenabled = true\ncollect = [\"segments\"]\n",
});

axis!(Estimate ("Which units can an estimate use?", default = Off) {
    Off = "off": "any unit you like"
        => "# estimate: free-form, no unit allow-list.\n",
    DaysHours = "days-hours": "days and hours only"
        => "# estimate: days and hours.\n[tasks.estimate]\nunits = [\"days\", \"hours\"]\n",
});

axis!(Required ("What must every open task have?", default = None) {
    None = "none": "nothing extra"
        => "# required fields: nothing beyond the defaults.\n",
    Difficulty = "difficulty": "a difficulty rating"
        => "# required fields: difficulty on every open task.\n\
            [tasks.difficulty]\nrequired = \"!terminal\"\n",
});

axis!(Headers ("Which sections must a task body have?", default = Off) {
    Off = "off": "none, write the body any way you like"
        => "# body headers: no required sections.\n",
    Spec = "spec": "Summary, Acceptance criteria, Non-goals and Files"
        => include_str!("../axes/headers_spec.toml"),
});

axis!(Recurrence ("How do you write repeating tasks?", default = Off) {
    Off = "off": "with every(...) only"
        => "# recurrence: the built-in every(...) rules only.\n",
    Rrule = "rrule": "with every(...) or rrule(\"FREQ=...\")"
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
