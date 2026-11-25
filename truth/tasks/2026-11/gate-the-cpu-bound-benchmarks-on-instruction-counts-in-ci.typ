#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Gate the CPU-bound benchmarks on instruction counts in CI",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling", "gating", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 25),
)

== AS-IS

A load-path change lands with no CI signal on what it costs.

+ Open a pull request that doubles the sort's cost.
+ 🔴 `.github/workflows/ci.yml`: no job runs a benchmark. Every gate passes anyway.
+ Run `just bench::suite` by hand.
+ 🔴 It prints a number and compares it with nothing, so the reader has to remember the last one.

== TO-BE

- 🟢 A CI job runs the CPU-bound cases under `callgrind` and fails on an instruction count that rises past a threshold.
- 🟢 The wall-clock suite stays a local tool, and a nightly job saves a criterion baseline artifact for the cases `callgrind` cannot see.

```
$ just bench::instructions
instructions::cpu_bound::filter_match n2000:loaded_with_filter()
  Instructions:                    81128339|81269623             (-0.17385%)
instructions::cpu_bound::sort_default n2000:loaded()
  Instructions:                  1168167802|1165512108           (+0.22786%)

Iai-Callgrind result: Ok. 2 without regressions; 0 regressed
```

A case over its ceiling prints `Performance has regressed: Instructions (80690451)
exceeds limit by 79690451 (>1000000)`, and `cargo bench` exits non-zero.

== Changes

- `crates/mindtape/benches/instructions.rs`: the `callgrind` target over `filter/match` and `sort/default`, each with an absolute instruction ceiling beside it. A ceiling rather than a percentage against a saved baseline, because `iai-callgrind` stores a baseline as 1.1 MB of `callgrind` profiles and a fresh CI checkout has none.
- `crates/mindtape/Cargo.toml`: the `iai-callgrind` dev-dependency and the `harness = false` target.
- `nix/dev/`: the runner derivation the harness shells out to, which no package carries, and the shell entry that puts it and `valgrind` on PATH.
- `.just/bench.just`: the `instructions` recipe.
- `.github/workflows/`: the push job that gates on the counts, and the nightly job that saves a criterion baseline for the cold path.
- `www/content/internals.typ` and `CHANGELOG.md`: the paragraph that documents the gate, and the entry under `[Unreleased]`.

== Out of scope

- A wall-clock threshold in CI. A shared runner is too noisy for one, and the cold path spends its time blocked, which instruction counts cannot see.
- The cold path's shape. `warm_load_serves_cache_and_skips_eval` and its neighbours already gate it.
- The sort's cost. `sort/default/2000` retires 1.165 G instructions, 582 k per task, which is the expensive half of `mt ls`. The gate bounds it. Making it cheaper is its own task.

== Dogfood

A pull request that makes the sort quadratic fails a gate instead of landing.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a cost regression fails a gate instead of waiting for someone to run the suite].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
