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
  status: proposed(2026, 11, 25),
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
filter/match/2000      Instructions:  12,345,678  (no change)
sort/default/2000      Instructions:  98,765,432  (+41.2%)  FAIL
```

== Changes

- `crates/mindtape/benches/instructions.rs`: the `callgrind` target over `filter/match` and `sort/default`.
- `crates/mindtape/Cargo.toml`: the `iai-callgrind` dev-dependency and the `harness = false` target.
- `nix/dev/shell.nix`: `valgrind`, which `callgrind` needs.
- `.github/workflows/ci.yml`: the job, on the Linux runner.
- `CHANGELOG.md`: an entry under `[Unreleased]`.

== Out of scope

- A wall-clock threshold in CI. A shared runner is too noisy for one, and the cold path spends its time blocked, which instruction counts cannot see.
- The cold path's shape. `warm_load_serves_cache_and_skips_eval` and its neighbours already gate it.

== Dogfood

A pull request that makes the sort quadratic fails a gate instead of landing.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[a cost regression fails a gate instead of waiting for someone to run the suite].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
