#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Retire the drift check and estimate bounds in favour of rules",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt",),
  estimate: (files: 14),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[every estimate check is a rule the project can read, override and switch off],
      cheap-adoption: helps[no check fires that a project did not write or inherit from a pack],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[about 1,400 lines of bespoke checker and tests go],
      docs-match-code: helps[the estimates page describes rules, not a special-cased unit],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 27),
  links: depends-on(
    "compare-estimate-units-and-compute-arithmetic-in-filters.typ",
    "add-sum-and-count-aggregates-to-filters.typ",
    "count-changed-files-and-lines-per-git-state-in-filters.typ",
  ),
)

== Problem

+ A project declares `[tasks.estimate] units = ["files"]` and one task goes `wip` with `estimate: (files: 3)`.
+ 🔴 `mt check` warns `working tree touches 17 file(s), over the 4 declared by …` although the project wrote no such check: `DriftConfig::default()` hardcodes `unit = "files"`.
+ 🔴 `[[tasks.estimate.max]]`/`min` is a second bespoke checker for what a rule says in one line.


== TO-BE

- 🟢 `[tasks.estimate]` keeps only `units` and `required`; `drift`, `max` and `min` are gone, with no alias (pre-release policy).
- 🟢 No estimate check runs unless a rule asks: a project copies the rules from `www/content/estimates.typ`.
- 🟢 This repo's config carries its budget and bounds as `[[tasks.rules]]`, keeping today's behaviour and `exclude` list.


== Changes

- `crates/mindtape-cli/tests/estimate_drift_e2e.rs` (red first, renamed `estimate_rules_e2e.rs`): the same scenarios written as rules, and no warning with none.
- `crates/mindtape/src/estimate_drift.rs`, `estimate_drift_tests.rs`: delete.
- `crates/mindtape/src/config_estimate.rs`, `config_estimate_tests.rs`: drop `DriftConfig` and `EstimateBound(s)`.
- `crates/mindtape/src/validate.rs`, `config.rs`, `lib.rs`: drop the bounds pass and re-exports.
- `crates/mindtape-cli/src/check.rs`: drop `report_drift`.
- `.mindtape/config.toml`, `examples/learn-piano/.mindtape/config.toml`: the rules.
- `www/content/estimates.typ`, `www/content/config.typ`: rules instead of tables.
- `CHANGELOG.md`: the removal.


== Alternatives

- Make drift opt-in and keep the table: keeps a checker that knows one unit.
- Ship the rules in the `mindtape-tasks` pack: a pack cannot know a project's units.


== Out of scope

- `[[tasks.limits]]`: `truth/tasks/retire-task-limits-in-favour-of-count-rules.typ`.


== Dogfood

This repo's own rules replace the tables in the same commit.

