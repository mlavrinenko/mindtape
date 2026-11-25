#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Resolve the sort's key once per task, not once per comparison",
  kind: "dev",
  priority: ice(
    impact: 9,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 3,
  tags: ("stateless",),
  assignee: "agent",
  status: done(2026, 11, 26),
)

== AS-IS

The sort re-derives each task's priority on every comparison.

+ Open a backlog of 2000 tasks and run `mt ls`.
+ 🔴 `crates/mindtape/src/sort.rs`: `compare_default` calls `priority_value` on both sides of every comparison.
+ 🔴 `crates/mindtape/src/framework/eval.rs`: `eval_score` parses the formula string with `build_operator_tree` on every call.
+ 🔴 A sort of 2000 tasks makes about 22,000 comparisons, so one formula parses about 44,000 times.
+ `just bench::suite` reports `sort/default/2000` at 76.7 ms, 3.5 µs per comparison.


== TO-BE

- 🟢 The sort computes each task's key once, then orders on the precomputed keys.
- 🟢 A framework parses its formula once, on first use, and keeps the tree.
- 🟢 `sort/default/2000` falls from 1.176 G instructions to 18.3 M, and the ceiling in `benches/instructions.rs` falls with it.

```
$ just bench::suite
sort/default/2000   time:   [4.51 ms 4.64 ms 4.78 ms]

$ just bench::instructions
filter_match n2000   Instructions:  32,050,572  (was 81,233,386)
sort_default n2000   Instructions:  18,291,624  (was 1,175,742,084)
```


== Changes

- `crates/mindtape/src/sort.rs`: the decorate-sort-undecorate pass over the default chain.
- `crates/mindtape/src/framework/`: the parsed formula cached on `Framework` and `Category`, and the tests that follow it.
- `crates/mindtape/benches/instructions.rs`: the lowered ceilings.
- `www/content/internals.typ` and `CHANGELOG.md`: the warm figure and the entry under `[Unreleased]`.
- `.config/jscpd-baseline.json` and `.config/crap-baseline.json`: the re-minted baselines, which drop the clone this removes and hold the two new functions.


== Out of scope

- The cold path. It spends its time blocked on fsync, which no sort change touches.
- `status_index`'s linear scan over the status legend. It is a handful of string compares against a list of five, and the key pass makes it once per task.
- `sort_by_specs`, the `--sort` path. It still resolves a key per comparison, and the instruction gate does not cover it. The same pass applies there once someone measures it.


== Dogfood

This repo's `mt ls` sorts its own backlog in a fraction of the time, and the instruction gate holds the new floor.


== Goals

+ #Helps(maintainer, to: keep-changes-safe)[the sort's cost falls by an order of magnitude, and the gate keeps it there].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).

