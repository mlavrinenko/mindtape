#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add a Typst function collector",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("mt", "gating", "stateless"),
  estimate: (src-files: 10),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[a project writes a collector in the same Typst module as its checks],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        sound-codebase: helps[the evaluator reuses CompiledCheck's snapshot and is covered by red tests],
      ),
      ..("docs-match-code",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 4),
  links: depends-on("add-derived-task-fields-and-a-regex-collector.typ"),
)

== AS-IS

+ Read the collector kind in `add-derived-task-fields-and-a-regex-collector.typ`.
+ 🔴 It runs only a regex. A project with a structured Typst extraction has no collector kind and must fall back to a rule `check` that returns breaches instead of a value.
+ Open `crates/mindtape/src/rules/typst.rs`.
+ 🔴 `CompiledCheck` already evaluates a project Typst function over the canonical task, but its result is read only as an array of breach strings.


== TO-BE

- 🟢 `query = { typst = "truth/lib.typ", function = "changes" }` runs a project function over the canonical task or a named field and returns the declared type.
- 🟢 The result lands as `derived.<name>` beside a regex collector; `when`, `returns` and the shared type check stay the same, and `mt rules` shows kind `typst` with the module and function.
- 🟢 A function error or a wrong-typed result is one collector violation naming the module and function.
- 🟢 One `SourceSnapshot` per collector is shared across tasks, as `CompiledCheck` already does it. Collectors run in declaration order, so a later query may read an earlier derived field.

```toml
[[tasks.collectors]]
name = "changes"
query = { typst = "truth/lib.typ", function = "changes", field = "body" }
returns = ["path"]
```


== Changes

- `crates/mindtape/src/config/collect.rs`: add the `typst` query variant, `CollectorSpec.root` and every `returns` type.
- `crates/mindtape/src/config/pack_files.rs`: stamp a pack collector's `root`, as `stamp_check_roots` does for a rule's check.
- `crates/mindtape/src/rules/typst.rs`: add a value-returning evaluation beside the breach call, reusing the probe, world and shared `SourceSnapshot`.
- `crates/mindtape/src/collect/typst.rs` (new): dispatch that evaluation, return the harvested `HVal`, and record a `Problem` naming the module and function.
- `crates/mindtape/src/config/layers.rs`: check the module and function at load, resolving a pack collector's module from its stamped root.
- `crates/mindtape/src/model/mod.rs`, `crates/mindtape-cli/src/cmd/eval.rs`: `Task.collector_problems: Vec<Problem>`, filled by the collector and printed by `mt eval`.
- `crates/mindtape/src/validate/mod.rs`, `violation.rs`: a `Collector` violation at the task marker line, field `derived.<name>`, severity error.
- `crates/mindtape/src/collect/mod.rs`: map an eval error, a panic or a wrong type to that problem.
- `crates/mindtape/src/collect/typst_tests.rs`, `config/collect_tests.rs`: red tests first, over the whole task and a named field, a missing function at load, a panic and a wrong type.
- `schema/config.schema.json`, `www/content/config.typ`, `rules.typ`, `CHANGELOG.md`: the schema and docs.


== Alternatives

- Shelling out to a project command: needs `mt allow` and gives a different result on each host.
- Reusing `check` and parsing its breach strings: a value cannot survive the array-of-strings ABI.
- A separate `mt eval` call per task: loses the loader's cache and the candidate seam.


== Out of scope

- ast-grep queries: the milestone `add-ast-grep-queries-as-a-shared-query-engine.typ`.
- Changing `check` to return values: rules keep the array-of-strings contract.
- Caching Typst results across commands: measure first.


== Dogfood

Not applicable: no current collector needs Typst; the kind exists for a project whose extraction a regex cannot express.

