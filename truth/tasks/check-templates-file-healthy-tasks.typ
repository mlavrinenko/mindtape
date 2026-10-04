#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check that every template files a healthy task",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a config change that strands a template fails mt check at once, not the next mt add weeks later],
      cheap-adoption: helps[an adopting repo learns its templates are stale from the gate it already runs],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[this repo's own eleven templates are held to the config the gate already holds the backlog to],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 4),
)

== Problem

+ In a scratch project, set `[tasks.fields.kind] values = ["fix"]` and add `.mindtape/templates/bug.typ` whose `task.with` hard-codes `kind: "bug"`.

+ Run `mt check`.

+ 🔴 It prints `ok: 0 task(s) valid`. Only `mt add -t bug "a crash"` finds the problem: `kind: `bug` is not an allowed value`, `nothing was written`.


== TO-BE

- 🟢 `mt check` evaluates every template `mt add --list` lists: it splices the required slots as `mt add` does, fills each slot with its `default`, and validates the result as a candidate task.

- 🟢 A field whose value holds a slot without a `default` is the caller's to fill, so its violations are skipped; a literal, a `default` or a missing required field is the template's, and is reported.

- 🟢 A breach is an error like a task's, and a template that fails to evaluate is one breach carrying the Typst error:

  ```
  error: .mindtape/templates/bug.typ:4: kind: `bug` is not an allowed value
  ```


== Changes

- `crates/mindtape-cli/tests/check_templates_e2e.rs`: red tests first — a stale `kind` literal, a stale `default`, a default-less slot skipped, a broken template.

- `crates/mindtape/src/template_check.rs`: fill every slot with its default, harvest through `harvest_candidate`, and drop violations on fields that hold a default-less slot.

- `crates/mindtape-cli/src/check.rs`: run the template pass beside the task pass and count its breaches.

- `www/content/templates.typ`: say `mt check` validates templates.

- `CHANGELOG.md`: an Added entry.


== Alternatives

- A `check-templates` gate in this repo's `Justfile` that runs `mt add --print` per template: guards this repo only, and every slot without a default needs a hand-picked value.
- Validate templates in `mt add` only: that is today's behaviour, and it fires after the config change has shipped.


== Out of scope

- Body content rules over a body still made of default-less slots: skipped like any caller-filled field.
- Fixing a stale template automatically.


== Dogfood

`just check` runs `mt check` over this repo's eleven templates and `examples/api-service`'s through `check-examples`, so a config change that strands one fails the gate.
