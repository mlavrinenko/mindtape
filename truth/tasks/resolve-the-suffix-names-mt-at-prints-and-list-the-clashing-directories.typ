#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Resolve the suffix names mt at prints and list the clashing directories",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("mt", "cli"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a name `mt at` prints works after `@`, so two same-named repos need no extra at.toml entry],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a member name and the token that selects it go through one suffix rule],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 18),
)

== Problem

+ List `~/projects/blog` and `~/oss/blog` in one `at.toml` set, `[at.both]`.
+ Run `mt @blog ls`.
+ 🔴 It prints `error: can't resolve: '@blog': @blog matches 2 directories`, naming neither directory and giving no hint.
+ Run `mt at both`. It lists the two as `projects/blog` and `oss/blog`.
+ Run `mt @oss/blog ls`.
+ 🔴 It prints `invalid name "oss/blog": must be one path segment`, so the name `mt at` shows can't be typed.


== Environment

- `mt` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 `mt @blog ls` names each matching directory with its suffix name, and hints how to pick one, as in `mt @projects/blog ls`.
- 🟢 `mt @oss/blog ls` lists the tasks under `~/oss/blog`: a token of several segments selects the directory whose path ends in them.
- 🟢 An `at.toml` entry name stays one path segment.


== Changes

- `crates/mindtape-cli/tests/it/at_e2e.rs`: the red tests first, the clash error and `@oss/blog`.
- `crates/mindtape/src/at/resolve.rs`: match a directory by its path suffix, and name the clashing ones.
- `crates/mindtape/src/at/mod.rs`: the clash carries its candidates, and a token may hold `/`.
- `crates/mindtape-cli/src/cmd/at.rs`: render the clash as a `Diag` with a note per directory and a hint.
- `www/content/read-many-projects.typ`: say a suffix name works after `@`.


== Out of scope

- A `/` in an `at.toml` entry name. An entry is a TOML key, and one segment keeps it apart from a path.


== Dogfood

Not applicable. This repo has one checkout per name, so no two directories clash.

