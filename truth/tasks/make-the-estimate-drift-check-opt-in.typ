#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Make the estimate drift check opt-in",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt",),
  estimate: (files: 7),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the file-budget warning fires only in a project that chose it],
      cheap-adoption: helps[a project that estimates in files gets no warning it never asked for],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the estimates page states one switch instead of an implicit trigger],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 27),
)

== Problem

+ A project declares `[tasks.estimate] units = ["files"]` and nothing under `[tasks.estimate.drift]`.
+ A task with `estimate: (files: 3)` goes `wip`, and the working tree grows to 17 changed files: code, tests, docs pages and `CHANGELOG.md`.
+ 🔴 `mt flip`, `mt check` and every write print `warning: working tree touches 17 file(s), over the 4 declared by …`, although the project never asked for a drift check.

The check is configurable already (`unit`, `in-flight`, `line-multiple`, `exclude` and `message` in `crates/mindtape/src/config_estimate.rs`), but it has no off state: `DriftConfig::default()` sets `unit = "files"`, so it arms itself the moment a task declares a `files` estimate. The only ways out are hacks (`unit` naming an unused unit, `in-flight = 'false'`).


== TO-BE

- 🟢 With no `[tasks.estimate.drift] unit`, the drift check never runs: a `files` estimate is a planning number and nothing more.
- 🟢 Setting `unit = "files"` turns it on; `exclude`, `in-flight`, `line-multiple` and `message` keep their current meaning and defaults.
- 🟢 A `[tasks.estimate.drift]` table carrying other keys but no `unit` is a config warning naming the missing key, not a silently inert block.


== Changes

- `crates/mindtape/src/config_estimate_tests.rs` (red first): the default `DriftConfig` has no unit; a table without `unit` warns.
- `crates/mindtape/src/config_estimate.rs`: `unit: Option<String>`, no `DEFAULT_DRIFT_UNIT`; the warning.
- `crates/mindtape/src/estimate_drift.rs`: return no budget when `unit` is unset.
- `crates/mindtape-cli/tests/estimate_drift_e2e.rs`: no warning without `unit`; the current cases set it.
- `www/content/estimates.typ`: document the switch.
- `.mindtape/config.toml`: opt this repo in explicitly.
- `CHANGELOG.md`: the default change.


== Alternatives

- An `enabled = true` key beside `unit`: two keys for one decision, and `enabled = true` with the default unit hides which estimate it reads.
- Keep it on by default and ship a default `exclude` for docs and changelogs: guesses at every project's layout, and still warns a project that never wanted a budget.
- An `include` glob list instead of `exclude`: a separate question of what counts, not whether the check runs; file it if a project asks.


== Out of scope

- Which paths this repo counts (tests, docs pages, `CHANGELOG.md`): an `exclude` decision, made in `.mindtape/config.toml`.
- Path-level attribution of the diff to tasks: still counts only, per the module doc.
- `[[tasks.estimate.max]]`/`min` bounds: those validate the declared number, not the working tree.


== Dogfood

This repo sets `unit = "files"` in `[tasks.estimate.drift]` and keeps its `exclude` list, so its warning stays as it is today by choice rather than by default.

