#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep scoped mutant sweeps out of the full-sweep cache",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling", "tests"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a full sweep stops paying for a scoped run's side effect],
      docs-match-code: helps[both pages name where each run writes],
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    1,
  )[scoped runs write \\\`target/mutants-scoped\\\`; the full sweep keeps \\\`mutants.out\\\`; the arm reads the newer of the two; \\\`just check\\\` green],
)

== Problem

+ Run a full workspace sweep: `just mutants --iterate`.
+ 🔴 The 2026-09-30 run printed `INFO Iteration excludes 46 previously caught or unviable mutants` before 15 hours of testing: earlier scoped runs had overwritten `mutants.out`, the cache a full run reads, so 4240 mutants were tested instead of only re-running the previous misses.
+ Every run wrote there because `cargo mutants` defaults its output to the working directory, and none of the three recipes passed `-o`.

== TO-BE

- 🟢 A run that selects a subset (`-f`/`--file`, `--in-diff`, `--diff`) writes `target/mutants-scoped` and leaves `mutants.out` to the full sweep's `--iterate` cache.
- 🟢 An explicit `-o`/`--output` still writes wherever the caller said.
- 🟢 `just mutants-diff` is always a subset run, so it writes there too.
- 🟢 The ask-refactoring mutants arm reads whichever of the two outcomes files ran last, so a scoped sweep still feeds it.

== Changes

- `.just/scripts/mutants.sh`: a subset selector in `"$@"` appends `--output target/mutants-scoped` unless an output flag was passed; a no-subset full sweep still writes `mutants.out`.
- `.just/scripts/mutants-diff.sh`: the same default for its always-subset run.
- `contrib/ask-refactoring/mutants.sh`: `MUTANTS_JSON` becomes the newer of `mutants.out/outcomes.json` and `target/mutants-scoped/outcomes.json`.
- `Justfile`: the `mutants`/`mutants-diff` comments name both destinations.
- `docs/src/contributing/mutation-testing.typ` and `docs/src/contributing/signal-sweep.typ` (with regenerated `.md`): the same two-destination story.

== Rollback

Remove the two routing blocks and the two-candidate loop in the arm; every run writes `mutants.out` again, and a scoped run once more clobbers the cache the full sweep reads.

== Out of scope

- cargo-mutants' own `--iterate` semantics: it consumes previous outcomes, so a full iterate still shrinks the cache — this stops scoped runs destroying it first, and nothing more.
- CI mutation runs: `mutants-diff` stays the per-branch check.

== Dogfood

The next full `just mutants --iterate` after a scoped run re-tests only the misses instead of rebuilding the whole list; `just ask-refactoring` still sees the scoped sweep's survivors.
