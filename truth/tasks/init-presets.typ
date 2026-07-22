#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "init presets",
  priority: ice(
    impact: 9,
    confidence: 0.75,
    ease: 3,
  ),
  difficulty: 3,
  kind: "umbrella",
  tags: ("cli", "docs"),
  status: wip(2026, 7, 22)[
    resolution, the four built-in shapes, template bundling, the e2e matrix and
    the docs have landed; three children carry the rest
  ],
)

== Summary

Umbrella for the `mt init` milestone: replace the two-mode scaffold with named presets over independent configuration axes, so a first run produces a shape someone actually wants rather than a file they have to learn before using.

Research across CLI task trackers found no sweet spot on config size — todo.txt keeps users because it has no surface, Taskwarrior keeps users because it has a large one — so the tool ships a few coherent shapes instead of picking one. A preset is also what makes a project self-contained: only the project's own config travels with the repository, so a shape left in a personal defaults layer is invisible to anyone who clones it.

This task coordinates and does no work of its own. It stays open while its children land.

== Scope

Landed:

- Four built-in presets — `todone`, `flow` (default), `personal`, `gated` — none named for its position or its audience.
- Presets authored annotated and rendered quiet by stripping whole-line comments, so `--preset` picks content and `--annotate` picks commentary, and the two renderings cannot drift.
- Resolution: user preset, built-in, explicit path, then an error listing what exists. Never a fall-through to a fetch.
- Templates bundled with the config, because a preset that constrains task bodies without shipping the template producing them scaffolds a project whose first `mt add` fails.
- The e2e matrix: every preset through init, add, check and a full status walk to terminal.
- Docs across README, quickstart, cli and config, plus the `outdatty` groups.

Children:

- `init-wizard` — the TTY-gated questionnaire over the axes.
- `preset-save` — capturing the current project as a reusable user preset.
- `preset-overlay-and-compose` — the `[overlay]` key and the compose-time array-conflict error.

Out of scope, permanently: remote presets. A preset carries Typst that gets evaluated, and the field record is bad — cookiecutter ships unsandboxed hooks by design and carried a 9.8 command injection, git carried a 9.0 hook-execution-during-clone, and npm `create-*` typosquats have exfiltrated CI secrets. If ever revisited, copy cargo-generate explicit consent or direnv content-hash allow.
