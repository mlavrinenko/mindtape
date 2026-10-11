#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a template whose text slot example holds markup mt add escapes",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: helps[a code span in a filed task renders as code, not as backslashed backticks],
      implementable-tasks: neutral,
      enforced-practice: helps[a template can no longer teach input its own slot mangles],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 12),
)

== Problem

+ Run `mt add --template finding --describe`. The `context` slot is `text`, and its example reads #raw("Running `just check` while fixing the sort order").
+ Fill `context` the way the example shows and run `mt add --template finding -f -`.
+ 🔴 The task reads #raw("Running \\`just check\\`"): a `text` slot escapes markup, so the code span files as literal backticks. `tweak`'s `why` and `sweep-mutants`' `scope` show code spans the same way, 196 task files under `truth/tasks/` carry the escape, and the skill's warning about it, added by `document-that-mt-add-text-slots-escape-markup-instead-of-rendering-it`, did not stop the next one.


== TO-BE

- 🟢 `mt check` refuses a `text` slot whose example `mt add` would escape, at the slot's line, as in #raw("slot `context` is `text`, so `mt add` escapes the ` in its example; make it `kind: \"raw\"` or keep markup out of the example").
- 🟢 This repo's templates file a code span as code, and no task under `truth/tasks/` carries an escaped backtick.


== Changes

- `crates/mindtape/src/template/check_tests.rs`: the red test first, a `text` slot with a code span in its example as one breach at its line.
- `crates/mindtape/src/template/check.rs`: `escaped_examples`, one breach per `text` slot whose example changes under `escape_markup`.
- `crates/mindtape-cli/src/cmd/check/templates.rs`: add those breaches to each template's own.
- `crates/mindtape-cli/tests/it/check_templates_e2e.rs`: `mt check` names the slot and line.
- `.mindtape/templates/finding.typ`, `tweak.typ`, `sweep-mutants.typ`: make the four prose slots `raw`.
- `truth/tasks/*.typ`: turn each escaped code span back into a code span.
- `www/content/templates.typ`: say what the check refuses.


== Alternatives

- A body rule refusing a backslashed backtick in every task. It catches the damage after `mt add` wrote it, in this repo only, and a task that discusses escaping trips it.
- A warning when `mt add` escapes a backtick in a caller's value. It fires on every fill and leaves the template that taught the input unchanged.
- Escaping only backticks in the check. A `#` or `_` in a `text` example files escaped too, so the example still misleads.


== Out of scope

- What `mt add` does with a caller's own value. A `text` slot still escapes it, which is what the kind is for.


== Dogfood

Every `mt check` in this repo holds its eleven templates to the rule, and the pre-commit hook runs it.
