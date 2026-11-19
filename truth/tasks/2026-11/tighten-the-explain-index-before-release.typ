#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Tighten the explain index before release",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 3),
  tags: ("cli", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a session writes `priority: ice(...)` from the index alone.],
      enforced-practice: helps[the index says which rules refuse the work and which only warn.],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: helps[the index answers what each line means without a page beside it.],
    ),
    process: (
      sound-codebase: helps[the continuation indent derives from the label, so no call site can misplace it.],
      docs-match-code: helps[the page and the CHANGELOG follow the same shape.],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 19),
  links: parent("make-mt-explain-the-session-brief.typ"),
)

== AS-IS

+ Run `mt explain` in this repo.
+ 🔴 `priority: ice (confidence, ease, impact)` hides the bounds. `mt check` refuses `confidence: 8`, and the index never says `confidence` stops at 1.
+ 🔴 `rules:` prints 64 names with no severity, so `keep-work-inside-changes-plan` reads as strict as the ten that refuse the work.
+ 🔴 A wrapped rule list continues at the same two-space indent as its section label. The tail reads as a new section.
+ Run `mt explain` in `examples/minimal`.
+ 🔴 `rm: terminal only` reads as a claim about `rm`, not as the `[rm] default-filter`.
+ Read the last line.
+ 🔴 `next:` promises the command to run next, and names `mt explain`'s own subcommands.

```text
$ mt explain | sed -n '8,9p;17,19p'
  priority: ice (confidence, ease, impact)
  difficulty: 0-9
rules:
  tasks.rules: tag-task-stateful-or-stateless, start-title-with-verb, keep-one-milestone-in-progress,
  keep-one-task-in-progress, cap-judgment-work-at-max-source-files, keep-work-inside-changes-plan,
```

== TO-BE

- 🟢 Each priority input carries what it accepts, and the score's scale follows an arrow.
- 🟢 Each block carries the rules that check it, so no rule appears twice.
- 🟢 A section prints one line per severity it holds, `error` before `warn`.
- 🟢 A wrapped list continues under its label, indented past it.
- 🟢 A key the project leaves open prints what its absence permits: `any`, `none`, or the policy word.
- 🟢 The index ends on a `more:` line naming the listings that go deeper.

```text
$ mt explain
project:
  roots: truth/tasks (tag: mt)
  ...
task:
  priority: ice (confidence 0-1, ease 1-10, impact 1-10) -> 0-9
  ...
  rules (error): tag-task-stateful-or-stateless, start-title-with-verb,
    keep-one-milestone-in-progress, keep-one-task-in-progress,
  rules (warn): keep-work-inside-changes-plan
  body.headers (error): allow-only-declared-sections, ...
commits:
  trailer: Refs
  rules (error): name-task-in-refs-trailer, ...
add:
  hooks: file-by-month, sweeps
flip:
  hooks (error): review-stale-task-before-starting, close-only-on-fresh-gate
rm:
  filter: terminal

more: mt explain <add|rules|tags|views|dicts|templates>
```

== Changes

- `crates/mindtape/src/explain.rs`: `PriorityShape.inputs` carries an `InputShape` per input, with its `kind`, `min`, `max` and `values`.
- `crates/mindtape-cli/src/cmd/explain/index.rs`: the per-block rules, the severity split, the input line, the derived continuation, the inline consequences and the `more:` line.
- `crates/mindtape-cli/src/cmd/explain/index_tests.rs`: the red tests first, one per shape.
- `crates/mindtape-cli/src/diag/mod.rs`: `more_line`, beside `next_line`.
- `crates/mindtape-cli/src/render/objects.rs`: `rules` nests a severity map under each section, and `add` carries the hook names.
- `crates/mindtape-cli/src/cmd/schema/registry.rs`: `explain` moves to version 4.
- `schema/output/explain/4.schema.json`: the new shape, beside the earlier versions.
- `www/content/cli/explain.typ`: the blocks, the severity split and the `more:` line.
- `CHANGELOG.md`: the entry under `[Unreleased]`.

== Alternatives

- YAML through a formatter: `mt explain --json | yq -y` already prints it, at 191 lines against 52. The text form is a brief, not data.
- `priority: ice (confidence 0-9, ease 0-9, impact 0-9)`: wrong. The inputs do not share a scale, and `confidence` stops at 1.
- `must:` and `should:` for the severities: a third vocabulary beside the `error` and `warn` that `mt explain rules` and `mt check` print.
- A nested `rules: {section: {severity: [...]}}` in the text: three lines per section where the inline label takes two.
- `next:` kept: the command already answered. The line offers detail rather than a way out.
- `kinds: any kind` and `actors: any actors`: the noun repeats the key. `any` alone says it, and `none` covers an empty registry.

== Out of scope

- Grouping the rules by the moment they run. `tasks.rules` runs at add and flip, and a moment grouping repeats it. The block each rule lives in is the one axis where it appears once.
- The `absent` key in the JSON. It stays, and a consumer reads the explicit list.
- The `absent:` line itself. `omit-the-absent-line-when-nothing-is-absent` owns it.

== Dogfood

This repo's `mt explain` prints the ICE bounds an agent needs to write `priority: ice(...)`, and splits its 64 rules into 63 errors and one warning.
