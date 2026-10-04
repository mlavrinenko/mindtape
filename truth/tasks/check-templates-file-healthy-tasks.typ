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
  links: depends-on("give-a-template-slot-an-example-value.typ"),
  status: done(2026, 10, 4),
)

== Problem

+ In a scratch project, set `[tasks.fields.kind] values = ["fix"]` and add `.mindtape/templates/bug.typ` whose `task.with` hard-codes `kind: "bug"`.

+ Run `mt check`.

+ 🔴 It prints `ok: 0 task(s) valid`. Only `mt add -t bug "a crash"` finds the problem: `kind: `bug` is not an allowed value`, `nothing was written`.


== TO-BE

- 🟢 `mt check` evaluates every template `mt add --list` lists: it splices the required slots as `mt add` does, fills each slot with its `example`, else its `default`, and validates the result as a candidate task. Every breach counts.

- 🟢 A slot with neither an `example` nor a `default` is a template error naming the slot, and the template goes unchecked until it has one. A slot `mt add` splices for a required field takes its example from config (`required_core_slots`): a choice's default or first value, the first allowed assignee or actor, a scale's or framework input's midpoint, one of the first estimate unit.

- 🟢 A template is filed at the status `mt add` stamps by default, and a template's own `status:` names that status (`done(…)` files at `done`, dated today), so `tweak.typ` files done and passes `tweak-lands-done`.

- 🟢 A breach is reported at the template's file and line, and a template that fails to evaluate is one breach carrying the Typst error:

  ```
  error: .mindtape/templates/bug.typ:4: kind: `bug` is not an allowed value
  ```

- 🟢 The built-in fallback template reports at `[templates] fallback-severity`, `"warning"` by default, settable in any config layer; a project's own templates report errors.


== Changes

- `crates/mindtape-cli/tests/check_templates_e2e.rs`: red tests first — a stale literal, example and default, a slot with neither, a rule breach on an example, the fallback severity.

- `crates/mindtape/src/template_fill.rs`: `fill_examples` fills each slot with its example, else its default.

- `crates/mindtape/src/template_check.rs`: one breach per slot with neither, and every breach of the filled task moved onto the template's file and line.

- `crates/mindtape-cli/src/check_templates.rs`: splice and stamp each template as `mt add` does and check them as one `check_candidates` batch.

- `crates/mindtape-cli/src/check.rs`: run the template pass beside the task pass and count its breaches.

- `crates/mindtape/src/config.rs`, `defaults.toml`: `[templates] fallback-severity`.

- `crates/mindtape/src/add_slots.rs`: each spliced slot carries an `example:` derived from config, red tests first in `add_slots_tests.rs`.

- `crates/mindtape-cli/src/task_edit.rs`, `add.rs`: `filing_status` reads a template's own `status:` for `mt add` and the check alike.

- `.mindtape/templates/*.typ`, `examples/*/.mindtape/templates/task.typ`: an example for every slot without a default; a goal-rated template rates one goal `helps[…]` in its example.

- `www/content/templates.typ`, `CHANGELOG.md`: the check and the config key.


== Alternatives

- Fill each slot with its `default` and a made-up placeholder, and skip breaches on fields the caller fills: it guesses which breaches the caller would clear, and checks nothing a placeholder stands in.
- A `check-templates` gate in this repo's `Justfile` that runs `mt add --print` per template: guards this repo only, and every slot needs a hand-picked value kept outside the template.
- Validate templates in `mt add` only: that is today's behaviour, and it fires after the config change has shipped.


== Out of scope

- Fixing a stale template automatically.


== Dogfood

`just check` runs `mt check` over this repo's eleven templates and `examples/api-service`'s through `check-examples`, so a config change that strands one fails the gate.
