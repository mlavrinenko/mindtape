#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give a template slot an example value",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a template shows a filled-in value for each slot, so a caller sees what a good answer looks like],
      enforced-practice: helps[the template check fills each slot with its example and counts every breach, instead of guessing which ones the caller will fix],
      cheap-adoption: neutral,
      typst-over-markdown: helps[a body slot's example is a Typst raw block, highlighted and written as markup],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 4),
)

== Problem

+ Write a template whose slot has no `default`:

  ```typ
  priority: ice(impact: slot("impact", kind: "int"), confidence: 0.8, ease: 5),
  ```

+ Ask what a caller should put there with `mt add -t <stem> --describe`.

+ 🔴 It prints the kind and the prompt. No slot can carry a sample value, so the template check in `check-templates-file-healthy-tasks` has to invent a placeholder and guess which breaches the caller would clear.


== TO-BE

- 🟢 A slot takes `example:`, a value `mt add` never files and Typst never evaluates:

  ````typ
  title: slot("title", example: "Retry failed invoice payments"),
  impact: slot("impact", kind: "int", example: 7),
  cheap-adoption: slot("cheap-adoption", kind: "raw", default: "neutral", example: ```typc
    helps[one table instead of a Typst check]
  ```),

  == Problem
  #slot("problem", kind: "raw", example: ```typ
    + Run `mt check`.
    + 🔴 It prints `ok`.
  ```)
  ````

- 🟢 `default:` and `example:` of a `raw` or `text` slot take a raw block as well as a string, dedented as Typst dedents it.

- 🟢 `mt add --describe` shows each slot's example, in text and in JSON.


== Changes

- `crates/mindtape/src/template_tests.rs`: red tests first — a scalar example, a raw-block example and default dedented, an example never filed.

- `crates/mindtape/src/template.rs`: `SlotSpec` carries `example`, and slot discovery reads a raw block as a `default` or `example` literal.

- `lib/prelude.typ`: `slot` takes `example: none` and ignores it.

- `crates/mindtape-cli/src/add.rs`: `--describe` prints the example.

- `www/content/templates.typ`, `CHANGELOG.md`: the `example:` argument and raw-block literals.


== Alternatives

- A values file per template (`feat.example.yaml`): a second file that drifts from the slots it fills.
- Reuse `default` as the example: `mt add` would file it whenever the caller says nothing.


== Out of scope

- The template check itself, which consumes `example`: `check-templates-file-healthy-tasks`.
- Prefilling the interactive editor with the example.


== Dogfood

`check-templates-file-healthy-tasks` gives this repo's templates their examples, and `just check` fills every one of them.
