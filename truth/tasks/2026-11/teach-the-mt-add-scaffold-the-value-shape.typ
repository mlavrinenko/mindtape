#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Teach the mt add scaffold the value shape",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("stateless", "mt", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a slot example shows what a good answer looks like],
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: helps[the format sits in the scaffold, not in a second command],
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the scaffold and `mt add --describe` read the same example],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 20),
)

== AS-IS

+ Run `mt add --describe` in this repo. A slot that declares an `example:` prints it:

```
title  string  required  # Task title, imperative verb first
    e.g. Test that a task without a due date sorts last
```

+ Set `[add] comments = true` and run `mt add --edit` in a terminal.
+ 🔴 The comment above that slot stops at the kind and the requirement. The example never reaches the editor:

```
# Task title, imperative verb first [string, required]
title:
```

+ A `date` slot's format appears only in `mt add --describe`, and `17.11.2026` reads as a valid answer until the fill rejects it.

== TO-BE

- 🟢 A slot's comment carries its `example:`, indented under the prompt.
- 🟢 A `date` slot's comment spells the format and shows a sample.
- 🟢 A `date` slot with a default pre-fills it. An untouched slot keeps the default.

```
# Task title, imperative verb first [string, required]
# e.g. Sort tasks without a due date last
title:
# due [date, optional]
# YYYY-MM-DD, e.g. 2026-11-17
due:
```

== Changes

- `crates/mindtape-cli/src/cmd/add/interactive_tests.rs`: the red test first, an example line and a date format line in the scaffold.
- `crates/mindtape-cli/src/cmd/add/interactive.rs`: `push_comment` prints `example_string`, and a `date` slot gets its format line.
- `www/content/templates.typ`, `CHANGELOG.md`: what the scaffold's comments carry.

== Alternatives

- Print the example as the value instead of a comment: `mt add` would file it whenever the caller says nothing.
- Leave the scaffold bare and point at `--describe`: the reader runs a second command to learn the shape.

== Out of scope

- The re-open loop. `keep-the-mt-add-buffer-when-the-fill-rejects-it.typ` carries it.
- A prompt wizard or a TUI.

== Dogfood

`examples/api-service` keeps the default `[add] comments = true`. Its scaffold gains the examples. This repo keeps comments off and reads `mt add --describe` instead.
