#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Align the projects mt at lists for one set",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("cli", "found"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: helps[a set listing reads as one column of paths],
      implementable-tasks: neutral,
      enforced-practice: neutral,
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
  status: proposed(2026, 10, 17),
  links: parent("show-every-tracked-project-from-one-mt-name.typ"),
)

== Problem

+ Write `[at.home] include = ["~/projects/home/*"]` in `at.toml`.
+ Run `mt at`. It prints `@home → ~/projects/home/*`, with the path as written.
+ Run `mt at home`.
+ 🔴 Names and paths aren't aligned, and each path is absolute:
  ```
  agent-kit  /home/me/projects/home/agent-kit
  blustroke  /home/me/projects/home/blustroke
  canibump  /home/me/projects/home/canibump
  ```


== Environment

- `mt` built from `main` after the 4q2 children landed, on Linux.


== TO-BE

- 🟢 `mt at home` pads each name to the longest one, so the paths line up in one column.
- 🟢 A path under the home directory prints as `~/…`, the same way `mt at` shows what `at.toml` says. `--json` keeps the absolute path.


== Changes

- `crates/mindtape-cli/src/cmd/at_tests.rs`: the red test first, two members with names of different lengths and paths under the home directory.
- `crates/mindtape-cli/src/cmd/at.rs`: pad the names in `show_one` and print a home path as `~/…`.


== Out of scope

- The `--json` shape, which tools read.


== Dogfood

The owner reads `mt at home` to see which of the 28 projects under `~/projects/home` the set holds.

