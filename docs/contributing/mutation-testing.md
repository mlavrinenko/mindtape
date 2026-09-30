<!-- Generated from docs/src/contributing/mutation-testing.typ by `just docs md`. Do not edit; edit the source. -->

# Mutation testing

Deep dive behind one line of CONTRIBUTING.md’s Testing section: `cargo-mutants`.

## What it buys

Coverage proves a line executed. Mutation testing proves a test would fail if that line were wrong. A surviving `MISSED` mutant is a real assertion gap — code your suite runs but does not pin.

## How to run it

- `just mutants-diff` before a PR. It sweeps only the mutants your branch’s diff touches, so it is minutes rather than hours, and covers new files for free.
- `just mutants -f <file>` forces a deliberate full-file sweep.
- `just mutants` runs the whole workspace — hours — and is what refreshes the `mutants.out/outcomes.json` the ask-refactoring mutants arm reads.
- `just mutants-status` reads that same file to report a running sweep’s progress and ETA.

Neither is a `just check` gate, and there is no CI job for it on purpose. The file set, the excludes, the `rrule` feature and the debuginfo-free `mutants` profile are `.cargo/mutants.toml`’s subject.

## When a verdict is worth believing

A `TIMEOUT` is not a verdict in either direction: under parallel load it can hide a real pass or a real survivor equally well. So a sweep you intend to BELIEVE runs at `CARGO_MUTANTS_JOBS=2` or lower with an extended timeout, not at the default parallelism. The speed knobs — three parallel mutant jobs, two build jobs and the mold linker, benchmarked at ~3.5x over the naive run, argued in `.just/scripts/_mutants-env.sh` — are for the pre-PR pass, not for a final verdict.

A REPRODUCIBLE timeout is a finding, not noise: it means a mutant reached an unbounded loop. Four found so far — `MAX_CATCH_UP_STEPS` in `recur_dates.rs`, `MAX_DISAMBIGUATE_ATTEMPTS` in `naming.rs`, the string scan in `config_hooks.rs` and `find_root_bounded` in `loader.rs` — were fixed by bounding the loop with a `for` over a fixed range — never by a `while n <=
CEILING`, which still hangs once the mutant stops incrementing `n`.

A survivor inside `#[cfg(not(feature = …))]` code is unreachable under the sweep’s own feature set. Move the twin into its own file and `exclude_globs` it (`recur_rrule_off.rs`) so the live side stays one always-compiled function.

## Why there is no CI mutation job

Every mutant re-runs a suite that includes subprocess and PTY e2e tests, and `TIMEOUT` verdicts are least trustworthy exactly where a shared runner puts you. An advisory check with unreliable verdicts trains people to ignore it, and a cold build on two runner cores turns “advisory” into the slowest job in the pipeline. Run it locally, where a timeout means something.

## Triage

Every survivor is one of three things:

- A real gap — write a regression test that kills it.
- An equivalent mutant, where the mutated code is genuinely indistinguishable from the original. Record it in `.cargo/mutants.toml` `exclude_re`, with its reason.
- A gap not worth pinning now — `just ask-refactoring` reads the same outcomes file and files one scored task per survivor for a later sprint.

Never leave a survivor untriaged; the value of the sweep is that the list is empty for a stated reason, not that it is short.
