#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Add a benchmark harness for the load path",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("mt", "tooling", "gating", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 25),
)

== AS-IS

A backlog of 2000 tasks loads in 13.9 s cold and 0.17 s warm, and nothing in the tree measures either number.

+ Run `just bench::run 2000`.
+ 🔴 The recipe reports 0 of 0 tasks: Git ignores `examples/large/tasks`, and `mt` reads the working tree through Git.
+ Run `cargo bench` in a fresh clone.
+ 🔴 `cargo bench` doesn't build a target and doesn't print a result.

== TO-BE

- 🟢 `just bench::suite` runs a criterion suite over the load path and prints a time per case.
- 🟢 `just bench::save main` then `just bench::compare main` reports the change against a saved baseline.
- 🟢 `just bench::run` times the real binary warm, cold and with the cache off.
- 🟢 `just bench::profile` attributes the warm path's CPU to a function with `callgrind`, and `just bench::syscalls` attributes a cold run to a syscall with `strace`.

```
$ just bench::suite
load/warm/2000          time:   [69.068 ms 70.686 ms 72.669 ms]
load/cold/1000          time:   [6.6756 s 6.8475 s 7.0366 s]
filter/match/2000       time:   [4.9161 ms 4.9281 ms 4.9412 ms]
sort/default/2000       time:   [74.458 ms 74.798 ms 75.190 ms]
```

== Changes

- `crates/mindtape/benches/fixture.rs`: the fixture generator, a deterministic N-task Git repository under `target/bench/`.
- `crates/mindtape/benches/fixture/config.toml`: the benchmark project's config, moved from `examples/large`.
- `crates/mindtape/benches/load.rs`: the criterion suite over load, filter and sort.
- `crates/mindtape/Cargo.toml`: the criterion dev-dependency and the `harness = false` bench target.
- `Cargo.toml`, `.just/bench.just` and `nix/dev/shell.nix`: the bench profile, the recipes, `valgrind` and `strace`.

== Alternatives

- Divan: a nicer API and allocation counters, but no baseline comparison, and the point here is a measurement that moves between runs.
- iai-callgrind: deterministic instruction counts, which suit a CPU-bound hot loop. The cold path is 13.9 s of which 2 s is user CPU, so instruction counting cannot see the cost.
- Keeping the shell generator: it wrote files Git ignores, so the fixture didn't load a single task.
- `samply`: a sampling profiler with a better report, but it needs a privilege a normal user lacks, so a recipe built on it fails where it ships.

== Out of scope

- Fixing the cold path. The suite measures it. The fix is its own task.
- A CI job that gates on a wall-clock threshold. A shared runner is too noisy for one, and the suite is a local tool until a stable runner exists.

== Dogfood

This repo runs `just bench::suite` before a change to the loader, the filter or the sort, and `just bench::compare main` to see what moved.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[because a load-path change that costs 40% shows up before it lands].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
