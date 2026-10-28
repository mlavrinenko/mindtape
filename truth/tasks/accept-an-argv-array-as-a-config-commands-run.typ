#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Accept an argv array as a config command's run",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("mt", "gating"),
  estimate: (src-files: 7),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[one checked-in config runs the same command on Linux, macOS and Windows],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a checker's id fills argv items, so no id can reach a shell],
      docs-match-code: helps[hooks, waits and config pages show both forms of run],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ In a project used on Linux and Windows, write `[[on.flip]]` with `run = "scripts/check.sh"`, or a `[[tasks.waits]]` checker with `run = "gh pr view {id}"`.
+ 🔴 `mt` runs it through `sh -c` on Linux and `cmd /C` on Windows, which share no syntax, so one line can't serve both.
+ 🔴 `{id}` pastes into a shell line, so `waits.rs` must refuse every id holding a character either shell reads.


== TO-BE

- 🟢 A `run` is a string for the platform shell, as now, or an array of strings `mt` spawns directly with no shell:

  ```toml
  [[tasks.waits]]
  name = "pr"
  run = ["gh", "pr", "view", "{id}"]

  [[on.flip]]
  when = 'into("done")'
  actions = [{ run = ["cargo", "test"], gate = true }]
  ```

- 🟢 `{id}` fills each array item, so it can't inject shell syntax.
- 🟢 `mt allow` lists an array as `["cargo", "test"]`, and its digest tells it apart from any shell line, so turning one into the other asks again.


== Changes

- `crates/mindtape/src/shell.rs`: a `ConfigCommand` with a `Line` and an `Argv` form; `run` spawns `Argv` with no shell.
- `crates/mindtape/src/config/hooks.rs`: `run` takes either form, and an empty array is a config error.
- `crates/mindtape/src/waits.rs`: a checker's `run` takes either form, `{id}` filling each item.
- `crates/mindtape/src/allow.rs`: list an array in TOML syntax, and digest it apart from a line.
- `crates/mindtape-cli/src/cmd/flip/`: run and name either form.
- `crates/mindtape-cli/tests/it/`: an e2e test spawning `mt` itself as the argv, on every OS, red first.
- `schema/config.schema.json`: regenerate.
- `www/content/hooks.typ`, `www/content/wait-on-upstream.typ`, `www/content/config.typ`: both forms.


== Alternatives

- A config-level shell setting, as npm's `script-shell` and just's `set shell` have: one shell per project still can't serve a Windows and a Linux checkout of the same config, and `{id}` would still reach a shell.
- Keep the string form only: the presets dodge it with filter gates, but a project's own `run` stays per-platform and its `{id}` stays shell-quoted by hand.


== Out of scope

- Placeholders beyond `{id}`. An `[[on.flip]]` command reads the task from `MINDTAPE_TASK_FILE` and its siblings, in either form.
- Splitting a string `run` into words. A string stays a shell line.


== Dogfood

Not applicable. The repo's one `run`, the `gates-fresh` gate, runs `mmz` on Linux only, and changing its form would ask every checkout for `mt allow` again. The e2e test spawning `mt` runs both config kinds on every OS in CI.

