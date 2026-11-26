#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Mint the filter-match instruction ceiling from its measurement",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  links: (
    (
      task: "make-a-warm-mt-ls-stop-re-hashing-and-re-parsing-every-task.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "tooling", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 27),
)

== AS-IS

A finding records one fact about one path, with the evidence that shows it.

+ Running `just bench::instructions` while measuring the warm load path
+ 🔴 #raw("crates/mindtape/benches/instructions.rs"): the `filter/match/2000` ceiling is 34 M, and the case retires 36.3 M on `main`

```
$ git stash list   # nothing stashed; the tree is `main` at 9d8cf98d
$ cargo bench -p mindtape --bench instructions
instructions::cpu_bound::filter_match n2000:loaded_with_filter()
  Instructions:                    36264684|N/A                  (*********)
Performance has regressed: Instructions (36264684) exceeds limit by 2264684 (>34000000)
instructions::cpu_bound::sort_default n2000:loaded()
  Instructions:                    18257730|N/A                  (*********)
Iai-Callgrind result: Regressed. 1 without regressions; 1 regressed; 2 benchmarks finished in 137.951s
```

`sort/default/2000` still matches its 18.3 M baseline, so the machine and the
toolchain match the ones that minted the ceilings. `0fb64102` set the ceiling
at 34 M against a measured 32.1 M, and the case has since grown 13%.


== TO-BE

- 🟢 `just bench::instructions` passes on `main`.
- 🟢 The ceiling sits about 5% above whatever the case costs. The next regression then fails it.

```
$ just bench::instructions
Iai-Callgrind result: Success. 3 benchmarks finished
```


== Changes

- `crates/mindtape/src/filter/`: whatever the 13% went into, found by bisecting the case against `0fb64102`.
- `crates/mindtape/benches/instructions.rs`: the ceiling, re-minted from the measurement.


== Out of scope

- The `sort/default/2000` ceiling. It still holds.
- The wall-clock suite. This is about the instruction gate.


== Dogfood

`just bench::instructions` is the gate this repository runs before a loader, filter or sort change, and it fails on `main` today.


== Goals

+ #Helps(maintainer, to: keep-changes-safe)[because a red gate on `main` is one nobody reads, and the next real regression passes through it].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).

