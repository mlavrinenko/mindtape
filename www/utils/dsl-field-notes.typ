// Hand-written prose per core task-DSL field, keyed by field name. The field
// list and each field's default come from www/generated/dsl-fields.json,
// read straight off lib/prelude.typ's own `task` signature; this dict carries
// only what a machine cannot state — what the field means. `just
// check-doc-facts` fails naming any dsl-fields.json entry with no matching
// key here.
//
// Refs: truth/tasks/derive-the-drift-prone-doc-facts-from-their-sources.typ

#import "site.typ": u

#let dsl-field-notes = (
  "title": [Display title. The only field the built-in template requires.],
  "status": [A legend value dated today or explicitly: `wip(2026, 7, 6)`. A
    trailing block is a note: `wip(2026, 7, 6)[behind a flag]`.],
  "priority": [A number on the scale, or a framework call — see
    #link(u("/priority/"))[Priority].],
  "difficulty": [The sibling scale to priority, same shape.],
  "kind": [A config-defined enum value (e.g. `"feature"`, `"bug"`).],
  "assignee": [The actor currently working the task: a plain string.
    Orthogonal to status; never ownership.],
  "estimate": [An opaque `unit: quantity` dict, e.g. `(days: 1, hours: 2)`.
    Units are never converted or compared — only the allowed set is
    checked.],
  "links": [Typed edges to other tasks — a sum of helper calls, e.g.
    `depends-on("a.typ") + related("b.typ")[why]`. See "Task links" below.],
  "tags": [A tuple of strings: `("backend", "auth")`.],
  "due": [The task's deadline: `datetime(year: …, month: …, day: …)`.],
  "start": [When work on the task begins: `datetime(year: …, month: …, day:
    …)`.],
  "reviewed": [The date the task was last reviewed, stamped by `mt review`
    (idempotent within a day); never set by hand in the common case.],
  "recur": [A recurrence rule + policy: `every(weeks: 1)`. See "Recurring
    tasks" below.],
  "id": [An explicit identity override; unset, identity is the filename
    stem.],
)
