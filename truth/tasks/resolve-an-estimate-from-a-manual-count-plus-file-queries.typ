#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Resolve an estimate from a manual count plus file queries",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[the implementer reads the fan-out from queries anyone can re-run, not from a guess],
      enforced-practice: helps[a judgment cap measures the code the task will touch, and trips again at `mt wip` when the fan-out grew],
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
  status: proposed(2026, 10, 25),
)

== Problem

+ In `../speconaut`, run ```sh git grep -il refactoring e3ef3b5e -- 'src/*.rs' ':!*test*' | wc -l```.
+ 🔴 It prints `13`. The task filed at that commit said `estimate: (src-files: 4)`, and its commit touched 15 source files. Two more tasks missed the same way (8 against 11, 5 against 20). The estimate is a bare number, so `judgment-stays-small` checked a guess, and the query that would have shown the fan-out had nowhere to live.

== TO-BE

- 🟢 An estimate unit takes a manual count plus queries; the engine resolves it to the count plus the union of files the queries match:
  ```typ
  estimate: (src-files: count(2, grep(`"refactoring"`, in: "src/**/*.rs"))),
  ```
- 🟢 A project opts in per unit and names the path set a match must fall in:
  ```toml
  [tasks.estimate.queries]
  src-files = "@source"
  ```
- 🟢 Rules read `estimate.src-files` as the resolved number, unchanged. A plain number still works, and a unit with no `queries` entry refuses `count(...)`.

== Changes

- `crates/mindtape/src/config/estimate.rs`: the red test first, then `[tasks.estimate.queries]`, unit to path-set name.
- `lib/prelude.typ`: `count(n, ..queries)`, `grep(regex, in: glob)` and `glob(pattern)` constructors, and `_fmt-estimate` rendering a composite.
- `crates/mindtape/src/model/mod.rs`: parse a composite estimate beside a plain number.
- `crates/mindtape/src/estimate_queries.rs`, new: match queries natively over the git-listed files inside the unit's path set, cached per tree.
- `crates/mindtape/src/model/fields.rs`: `estimate.<unit>` reads the resolved count.

== Alternatives

- Shell commands such as `rg -l` as queries. They need `mt allow` and give different results on hosts with different tools.
- A separate `scout:` field beside `estimate`. Every cap rule would have to read two fields.
- A hardcoded rule that every judgment task names a fan-out command. A task that creates files has nothing to grep, so the rule fits only some tasks.

== Out of scope

- Showing the breakdown in `mt eval`, which the follow-up task adds.
- Queries for units that aren't file counts, such as hours.

== Dogfood

This repo's judgment tasks write `src-files: count(...)` with a `grep` per symbol they change, and `judgment-stays-small` reads the resolved count.
