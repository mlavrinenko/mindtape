#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Keep the mt add buffer when the fill rejects it",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 4,
  tags: ("stateless", "mt", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a rejected fill keeps the answers and the task gets filed instead of abandoned],
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: helps[one editor session instead of restarting the form],
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 20),
)

== AS-IS

+ Run `mt add --edit` in a terminal, in a project whose `[add] comments` is `false`.
+ Fill `title`, then change `impact` from its default to `five`.
+ Save and quit the editor.
+ 🔴 The add fails and the buffer disappears:

```
$ mt add --edit
error: slot has the wrong type: 'impact': expects int, got string
```

+ Run `mt add --edit` again. Every answer disappears, and the form starts from the scaffold.

== Environment

- `mt` 0.6.0 on Linux, with `$VISUAL` set to a terminal editor.

== TO-BE

- 🟢 A rejected buffer re-opens in the same editor, with the error as a `#` comment above the values, until it parses and fills.
- 🟢 The buffer survives a failed parse, a failed fill, an unknown key and a non-zero editor exit.
- 🟢 `[add] interactive = never` and a non-terminal run keep today's behaviour: one error, no editor.

```
$ mt add --edit
# error: slot has the wrong type: 'impact': expects int, got string
title: "Sort tasks without a due date last"
impact: five
```

== Changes

- `crates/mindtape-cli/src/cmd/add/interactive_tests.rs`: the red test first, a buffer that fails the fill coming back with the error comment.
- `crates/mindtape-cli/src/cmd/add/interactive.rs`: `run` takes the previous buffer and error, and returns the buffer it opened.
- `crates/mindtape-cli/src/cmd/add/pipeline.rs`: the fill moves inside the loop, so a rejected fill re-opens instead of returning.
- `www/content/templates.typ`, `CHANGELOG.md`: the re-open loop.

== Out of scope

- The scaffold's comments and the date format hint. `teach-the-add-scaffold-the-value-shape.typ` carries them.
- A prompt wizard or a TUI. This task keeps the editor and only stops the loss.

== Dogfood

This repo's `mt add --edit` re-opens on a bad `impact` instead of dropping the form.
