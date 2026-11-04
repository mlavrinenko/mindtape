#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Replace estimate.src-files with a collected Changes count",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt", "docs", "gating", "stateless"),
  estimate: (src-files: 10),
  x-goals: (
    product: verdicts(
      (
        enforced-practice: helps[the size cap reads the plan the task already carries, so one declaration is enforced],
        readable-tasks: helps[open tasks stop carrying a duplicate estimate],
      ),
      ..(
        "implementable-tasks",
        "cheap-adoption",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[the templates, packs, examples, docs and skill evals move in one change],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 5),
  links: depends-on("add-derived-task-fields-and-a-regex-collector.typ"),
)

== AS-IS

+ Run `mt ls '!terminal && has(estimate.src-files)'` in this repo.
+ 🔴 Every open task carries `estimate.src-files` and a `== Changes` section that names the same files; the two can disagree.
+ Change a Changes bullet and run `mt check`.
+ 🔴 The size rule still reads the estimate, so `== Changes` is unchecked prose.
+ Open `.mindtape/templates/task.typ` and `crates/mindtape/src/packs/budget.toml`.
+ 🔴 The template asks for an estimate and `.mindtape/config.toml` requires it, while the same files are already listed in Changes.


== TO-BE

- 🟢 `mindtape:budget` ships a `changes` collector over the Changes section, so `derived.changes` is a `["path"]`.
- 🟢 `judgment-stays-small` and `src-budget` read `count_paths(derived.changes, "@source")`; `src-budget` no longer gates on an estimate.
- 🟢 Templates stop asking for an estimate. The budget pack keeps `units`, and `.mindtape/config.toml` sets `required = false`, so old open tasks stay valid and no backfill runs.
- 🟢 Every Changes bullet starts with one backticked path, and Changes is non-empty, so the collector counts the plan.

```toml
[[tasks.collectors]]
name = "changes"
when = "!terminal"
query = { section = "Changes", pattern = "(?m)^\\s*-\\s+`([^`]+)`" }
returns = ["path"]
```


== Changes

- `crates/mindtape/src/packs/budget.toml`: declare the `changes` collector; both rules read `count_paths(derived.changes, "@source")`; `src-budget` gates on `status == "wip"`; keep `units` and drop the obsolete `src-files` query; rewrite the comments and message.
- `crates/mindtape/src/packs/tasks.toml`: require a leading backticked path on every Changes bullet with `must-not-match`, and add a non-empty rule.
- `.mindtape/config.toml`: set `[tasks.estimate] required = false`; rewrite the comment.
- `.mindtape/templates/*.typ`: drop every estimate slot and the literal estimate in `tweak.typ`, and start each Changes example with one backticked path.
- `examples/api-service/.mindtape/config.toml`, `.../templates/*.typ`, `www/generate-api-service.sh`, `www/generate-guides.sh`: declare a local `changes` collector, add `[dicts.source]`, set `required = false`, and stop passing `src-files`.
- `contrib/ask-refactoring/*.sh`: stop passing `estimate=(src-files: ...)`; make each generated Changes bullet start with one path.
- `crates/mindtape/src/config/packs_tests_budget.rs`, `www/content/packs.typ`: update the expected rules and prose.
- `www/content/estimates.typ`, `split-oversized-tasks.typ`, `require-a-task-shape.typ`, `config.typ`, `filters.typ`, `templates.typ`, `www/generate-reference.sh`, `www/generated/*`: describe derived fields and the optional estimate.
- `skills/mindtape/SKILL.md`, `references/*.md`, `evals/evals.json`: drop the estimate slot and teach `derived.changes`.
- `CHANGELOG.md`: the migration.


== Out of scope

- Removing `estimate` from the DSL or `units` from the pack before the open backlog ages out.
- Backfilling closed tasks.
- Expanding a glob in Changes into its matching files; a bullet names one path and a glob counts once.
- `examples/estimates` and `examples/rules`, which keep their estimate examples to document the optional field.
- A Typst function collector: `add-a-typst-function-collector.typ`.
- ast-grep file queries: the milestone `add-ast-grep-queries-as-a-shared-query-engine.typ`.


== Dogfood

This repo files new tasks without an estimate, and its size rules read `derived.changes` from the Changes section the template already asks for.

