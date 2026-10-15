#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rename the todone preset to todo-done",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli",),
  estimate: (src-files: 10),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[the preset and the wizard's workflow choice share one name, and `mt init --list-presets` says in plain words what each preset gives],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[each preset description states what its config.toml really enforces],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 16),
)

== Problem

+ Run `mt init --list-presets`.
+ 🔴 It lists `todone`, while the `mt init` wizard calls the same workflow `todo-done`.
+ 🔴 The descriptions read like spec notes, as in "Open or finished, nothing else constrained" and "A lifecycle with enforced transitions".


== TO-BE

- 🟢 `mt init --preset todo-done` writes the two-status preset, and `todone` is an unknown preset, with no alias.
- 🟢 `mt init --list-presets` describes each preset in plain words the product owner approved.


== Changes

- `crates/mindtape-cli/tests/it/init_e2e.rs`: the red test first, `--preset todo-done` accepted.
- `crates/mindtape-cli/src/cmd/init/preset.rs`: the new name and the four descriptions.
- `crates/mindtape-cli/src/cmd/init/presets/todone/`: move to `presets/todo-done/`.
- The tests, `Justfile`, `docs/contributing/gates.md`, `outdatty.yaml` and `outdatty.lock` that name the old path.
- `CHANGELOG.md`: the preset's new name.


== Out of scope

- Done tasks under `truth/tasks/` that name `todone`. They are history.


== Dogfood

Not applicable. This repo's config comes from no preset.
