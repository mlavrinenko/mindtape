#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "init presets",
  priority: ice(
    impact: 9,
    confidence: 0.75,
    ease: 3,
  ),
  difficulty: 3,
  kind: "milestone",
  tags: ("cli", "docs"),
  status: done(
    2026,
    7,
    23,
  )[all three children landed: the wizard, preset save/ls, and preset \[overlay\] with the compose-time array-conflict error. The init milestone is complete],
)

== Summary

Umbrella for the `mt init` milestone. Replace the two-mode scaffold with named presets over independent configuration axes. A first run then produces a shape someone actually wants, rather than a file they have to learn before using.

Research across CLI task trackers did not find a single sweet spot on config size. todo.txt keeps users because it has no surface, while Taskwarrior keeps users because it has a large one. The tool offers a few coherent shapes instead of picking one. A preset is also what makes a project self-contained. Only the project's own config travels with the repository, so a shape left in a personal defaults layer is invisible to anyone who clones it.

This task only coordinates. It stays open while its children land.

== Scope

Landed:

- Four built-in presets: `todone`, `flow` (default), `personal`, `gated`. None takes its name from its position or its audience.
- Presets authored annotated and rendered quiet by stripping whole-line comments, so `--preset` picks content and `--annotate` picks commentary, and the two renderings cannot drift.
- Resolution: user preset, built-in, explicit path, then an error listing what exists. Never a fall-through to a fetch.
- Templates bundled with the config, because a preset that constrains task bodies without the template that produces them scaffolds a project whose first `mt add` fails.
- The e2e matrix: every preset through init, add, check and a full status walk to terminal.
- Docs across README, quickstart, the `cli` and config pages, plus the `outdatty` groups.

Children:

- `init-wizard`: the TTY-gated questionnaire over the axes.
- `preset-save`: capturing the current project as a reusable user preset.
- `preset-overlay-and-compose`: the `[overlay]` key and the compose-time array-conflict error.

Out of scope, permanently: remote presets. A preset carries Typst that gets evaluated, and the field record is bad. cookiecutter runs hooks outside any sandbox by design and carried a 9.8 command injection. git carried a 9.0 hook-execution-during-clone, and npm `create-*` copycat packages have exfiltrated CI secrets. If ever revisited, copy the cargo-generate explicit consent or the direnv content-hash allow.
