<!-- Generated from docs/src/contributing/mutation-testing.typ by `just docs md`. Do not edit; edit the source. -->

# Mutation testing

Deep dive behind one line of CONTRIBUTING.md’s Testing section: `cargo-mutants`.

## What it buys

Coverage proves a line executed. Mutation testing proves a test would fail if that line were wrong. A surviving `MISSED` mutant is a real assertion gap — code your suite runs but does not pin.

## How to run it

- `just mutants-diff` before a PR. It sweeps only the mutants your branch’s diff touches, so it is minutes rather than hours, and covers new files for free.
- `just mutants -f <file>` forces a deliberate full-file sweep.
- `just mutants` runs the whole workspace — hours — and writes `mutants.out`, the cache an iterating full sweep reads back.
- A scoped run — `just mutants-diff` or `just mutants -f <file>` — writes `target/mutants-scoped` instead, so it cannot clobber that cache. The ask-refactoring mutants arm reads whichever of the two ran last.
- `just mutants-status` takes the directory to read (default `mutants.out`) and reports a running sweep’s progress and ETA.

Neither is a `just check` gate, and there is no CI job for it on purpose. The file set, the excludes, the `rrule` feature and the debuginfo-free `mutants` profile are `.cargo/mutants.toml`’s subject.

## When a verdict is worth believing

A `TIMEOUT` is not a verdict in either direction: under parallel load it can hide a real pass or a real survivor equally well. So a sweep you intend to BELIEVE runs at `CARGO_MUTANTS_JOBS=2` or lower with an extended timeout, not at the default parallelism. The speed knobs — three parallel mutant jobs, two build jobs and the mold linker, benchmarked at ~3.5x over the naive run, argued in `.just/scripts/_mutants-env.sh` — are for the pre-PR pass, not for a final verdict. The sweep also runs with no `RUSTC_WRAPPER`: every mutant re-keys, so a build cache such as kache only misses and floods its store.

A REPRODUCIBLE timeout is a finding, not noise: it means a mutant reached an unbounded loop. Four found so far — `MAX_CATCH_UP_STEPS` in `recur_dates.rs`, `MAX_DISAMBIGUATE_ATTEMPTS` in `naming.rs`, the string scan in `config_hooks.rs` and `find_root_bounded` in `loader.rs` — were fixed by bounding the loop with a `for` over a fixed range — never by a `while n <=
CEILING`, which still hangs once the mutant stops incrementing `n`.

A survivor inside `#[cfg(not(feature = …))]` code is unreachable under the sweep’s own feature set. Move the twin into its own file and `exclude_globs` it (`recur_rrule_off.rs`) so the live side stays one always-compiled function.

## Why there is no CI mutation job

Every mutant re-runs a suite that includes subprocess and PTY e2e tests, and `TIMEOUT` verdicts are least trustworthy exactly where a shared runner puts you. An advisory check with unreliable verdicts trains people to ignore it, and a cold build on two runner cores turns “advisory” into the slowest job in the pipeline. Run it locally, where a timeout means something.

## Triage

Every survivor is a real gap or one of the classes below. A real gap gets a regression test that kills it. A gap not worth pinning now goes to `just ask-refactoring`, which reads the last sweep’s outcomes file and files one scored task per survivor for a later sprint. The rest are equivalent mutants, where the mutated code is indistinguishable from the original: record each in `.cargo/mutants.toml` `exclude_re` (or `exclude_globs`), leading the comment with the class name so a recurring class is a grep away.

- type-tautology — a comparison the type already decides, such as `>= 0` on a `usize`, so the mutant flips nothing observable. Exclude it, and in review prefer the source form whose mutants stay killable: `x != 0` rather than `x > 0` on an unsigned count. No source lint can catch the class: the equivalence lies between the mutant and the source, not inside the source.
- pure-memoisation — a guard that only skips repeated work, so deleting it changes the running time, never the answer. Exclude it.
- discarded-effect — extra work a later step throws away, such as one more loop iteration whose result a final `resize` drops. Exclude it, or rewrite the source so the surplus step cannot happen.
- aliased-input — two inputs that name the same thing (`""` and `"."` as the current directory), so the mutated guard picks the same outcome. Exclude it.
- same-call — the mutant replaces a body with the call it already is, such as `Default::default()` for `Self::default()`. Exclude it, or delete the function when it is not public API.
- no-public-constructor — the arm is reachable only by a value the dependency gives no way to build. Exclude it.
- platform-specific — the difference exists on another OS only. Exclude it, or gate a test to that platform.
- cfg-unreachable — code compiled out by the sweep’s own feature set. Move it into its own file and add the file to `exclude_globs`.
- entry-point — a thin `main.rs` with no logic to pin. Exclude the file.

Never leave a survivor untriaged; the value of the sweep is that the list is empty for a stated reason, not that it is short.
