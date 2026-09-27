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
    "read-estimate-units-and-kebab-case-names-in-filters.typ",
    "count-changed-files-per-git-state-in-filters.typ",
    "move-open-tasks-and-templates-to-a-src-files-estimate.typ",
  ),
)

== Problem

+ A project declares `[tasks.estimate] units = ["files"]` and one task goes `wip` with `estimate: (files: 3)`.
+ 🔴 `mt check` warns `working tree touches 17 file(s), over the 4 declared by …` although the project wrote no such check: `DriftConfig::default()` hardcodes `unit = "files"`.
+ 🔴 `[[tasks.estimate.max]]`/`min` is a second bespoke checker for what a rule says in one line.


== TO-BE

- 🟢 `[tasks.estimate]` keeps only `units` and `required`; `drift`, `max` and `min` are gone, with no alias (pre-release policy). No estimate check runs unless a rule asks.
- 🟢 This repo's budget and caps are rules whose scope is written in the globs, so `exclude` goes too:

```toml
[[tasks.rules]]
name     = "src-budget"
when     = 'status == "wip"'
assert   = 'changed("staged+worktree", "crates/*/src/**", "lib/**", "!**/*_tests*.rs") <= estimate.src-files'
message  = "split the remaining work into a new task and finish the part in hand"
severity = "warn"

[[tasks.rules]]
name     = "judgment-stays-small"
when     = '!terminal && difficulty >= 3'
assert   = 'estimate.src-files <= 5'
```


== Changes

- `crates/mindtape-cli/tests/estimate_drift_e2e.rs` (red first, renamed `estimate_rules_e2e.rs`): budget and caps as rules; no warning without them.
- `crates/mindtape/src/estimate_drift.rs`, `estimate_drift_tests.rs`: delete.
- `crates/mindtape/src/config_estimate.rs`, `config_estimate_tests.rs`: drop `DriftConfig` and `EstimateBound(s)`.
- `crates/mindtape/src/validate.rs`, `config.rs`, `lib.rs`, `crates/mindtape-cli/src/check.rs`: drop the bounds pass, `report_drift`, re-exports.
- `.mindtape/config.toml`, `examples/learn-piano/.mindtape/config.toml`: the rules.
- `www/content/estimates.typ`, `www/content/config.typ`, `CHANGELOG.md`: rules instead of tables.


== Alternatives

- Make drift opt-in and keep the table: keeps a checker that knows one unit.
- Keep the 200-line-per-file ceiling as a rule: no record of it catching anything.


== Out of scope

- The `src-files` unit itself: `truth/tasks/move-open-tasks-and-templates-to-a-src-files-estimate.typ`.
- `[[tasks.limits]]`: `truth/tasks/retire-task-limits-in-favour-of-count-rules.typ`.
- Retuning the caps for a source-only count: ported as they are.


== Dogfood

This repo's own rules replace the tables in the same commit.
