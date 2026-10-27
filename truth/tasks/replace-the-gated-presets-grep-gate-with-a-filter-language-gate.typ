#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Replace the gated preset's grep gate with a filter-language gate",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt", "gating"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the placeholder gate is a filter condition mt check validates at load, not a shell line],
      cheap-adoption: helps[a fresh gated project closes its first task on any OS, with no mt allow first],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a test holds every shipped preset, answer set and embedded pack to no run],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ Run `mt init --preset gated`, file a task with `mt add`, fill its slots and walk it to `needs-review`.
+ 🔴 On Linux, `mt flip done` refuses until `mt allow` approves `! grep -q "TODO:" "$MINDTAPE_TASK_FILE"`, the preset's own `[[on.flip]]` gate in `crates/mindtape-cli/src/cmd/init/presets/gated/config.toml`.
+ 🔴 On Windows, `cmd /C` can't run that line at all, so `init_e2e` fails there even after `mt allow`.
+ 🔴 `matches(body, "TODO:")` can't replace it: `mt ls` exits 2 with `unknown field: 'body'`, because `matches()` reads only comparable fields.


== Environment

- `mt` built from `main` at `91715b06`, on Linux and on the `windows-latest` GitHub runner.


== TO-BE

- 🟢 `matches()` reads `body`, `note` and `tags`, the text-only fields `contains()` reads.
- 🟢 The gated preset refuses `done` on a task whose body still holds a `- TODO:` placeholder line, with a run-less gate and no shell:

  ```toml
  [[on.flip]]
  when = 'into("done") && matches(body, "(?m)^\\s*- TODO:")'
  actions = [{ gate = true, message = "fill in the template's TODO placeholders" }]
  ```

- 🟢 A fresh gated project closes a filled task with no `mt allow`, on every OS.
- 🟢 A test fails when a shipped preset, a composed `mt init` answer set or an embedded pack declares a `run`. A user's own config and packs may still declare one.


== Changes

- `crates/mindtape/src/filter/strings_tests.rs`: the red test, `matches()` over `body`, `note` and `tags`.
- `crates/mindtape/src/filter/predicates.rs`: pull the text-only field resolver out of `contains_value` so both functions share it.
- `crates/mindtape/src/filter/strings.rs`: `string_field` reads the text-only fields through it.
- `crates/mindtape-cli/src/cmd/init/presets/gated/config.toml`: the run-less gate `into("done") && matches(body, "(?m)^\\s*- TODO:")`.
- `crates/mindtape-cli/tests/it/init_e2e.rs`: walk every preset with no `mt allow`.
- `crates/mindtape-cli/src/cmd/init/preset_tests.rs`, `crates/mindtape-cli/src/cmd/init/wizard/tests.rs`, `crates/mindtape/src/config/packs_tests.rs`: no shipped config declares a `run`.
- `www/content/filters.typ`, `skills/mindtape/references/filters.md`: `matches()` reads the text-only fields.


== Out of scope

- An argv form of `run`. Its own task covers it.
- Restricting `run` in the binary. A user's own packs and configs may run anything.


== Dogfood

Not applicable. This repo's own config gates on `mt check` rules, not on the gated preset.

