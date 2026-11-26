#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Measure the warm load the way a user runs it",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "tooling", "gating", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 27),
)

== AS-IS

The benchmark fixture keeps an empty dependency list, and the harness cannot see the cost that dominates a warm load.

+ Run `just bench::run 2000`, then read a cache entry under `target/bench/tasks-2000/.mindtape/cache/`.
+ 🔴 `deps` is empty. The fixture's tasks import the installed `@local/mindtape`, so no project-local file is ever read. The per-task re-hashing that is 56% of a real warm `mt ls` then never appears in any number the harness reports.
+ 🔴 `load/warm` is not in the instruction gate, so a change that gives the win back passes `just bench::instructions`.


== TO-BE

- 🟢 Each generated fixture carries this checkout's `lib/` as a local package and points `@local/mindtape` at it. Its entries then record `lib/prelude.typ` and `lib/typst.toml` the way the real backlog does.
- 🟢 The fixture is hermetic: no `mt install` has to have run for it to evaluate.
- 🟢 A prelude edit rebuilds the fixture, and a number measured after one then describes the current prelude.
- 🟢 `load/warm/2000` is a gated case with a ceiling beside it.

```
$ just bench::instructions
instructions::cpu_bound::load_warm n2000:warm_fixture()
  Instructions:                   173112231|N/A
```


== Changes

- `crates/mindtape/benches/fixture.rs`: the fixture writes `lib/prelude.typ` and `lib/typst.toml` from this checkout, and `is_built` rebuilds it when the prelude moved.
- `crates/mindtape/benches/fixture/config.toml`: the `[[typst.local-packages]]` override that resolves `@local/mindtape` to the fixture's own `lib/`.
- `crates/mindtape/benches/instructions.rs`: `load/warm/2000` with a 182 M ceiling, and the `warm_fixture` setup that primes the cache outside the count.


== Alternatives

- A project-local prelude the tasks import directly, without the package machinery. It records one dependency instead of two, so it reproduces the shape but not the magnitude.
- Leaving the fixture alone and measuring the real backlog. The harness would then depend on this repository's own state, and a number would move whenever someone filed a task.


== Out of scope

- The cold path's ceiling. It spends its time blocked, and an instruction count cannot see that.
- A wall-clock gate. A shared runner is too noisy for one.


== Dogfood

This repository runs `just bench::instructions` before a change to the loader, and the fixture measures the path its own backlog takes.


== Goals

+ #Helps(maintainer, to: keep-changes-safe)[because a load-path regression fails a gate instead of waiting for someone to run the wall-clock suite].
+ #Helps(maintainer, to: match-docs-to-code)[because the numbers the internals page quotes come from a fixture that measures the real path].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself).

