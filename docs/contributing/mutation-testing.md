<!-- Generated from docs/src/contributing/mutation-testing.typ by `just docs md`. Do not edit; edit the source. -->

# Mutation testing

How to run `cargo-mutants` here, when to trust its verdict, and how to triage a survivor.

## What it finds

Coverage shows that a line ran. Mutation testing shows that a test fails when the line is wrong. A surviving (`MISSED`) mutant is code the suite runs but doesn’t check.

## How to run it

- Before a PR, run `just mutants diff`. It mutates only the code your branch changed, new files included, so it takes minutes.
- Both recipes count the mutants first, at about 13 s each. Over 100, they refuse until the owner reruns them with `MINDTAPE_MUTANTS_APPROVED=1`. A full sweep is always over, so it always needs that approval.
- To sweep one whole file, run `just mutants sweep -f <file>`.
- To sweep the whole workspace, run `just mutants sweep`. It takes hours and writes `mutants.out`.
- A scoped run writes `target/mutants-scoped`, so it never overwrites `mutants.out`. The sweep’s mutants arm reads whichever ran last.
- To see a running sweep’s progress and ETA, run `just mutants status`. It reads `mutants.out` unless you name another directory.

`.cargo/mutants.toml` sets the file set, the excludes and the `mutants` build profile without debug info.

## When to trust a verdict

A `TIMEOUT` is no verdict either way. Under parallel load it can hide a real pass or a real survivor. For a verdict you mean to trust, run with `CARGO_MUTANTS_JOBS=2` or lower and a longer timeout. The default speed settings (three mutant jobs, two build jobs and the mold linker, about 3.5 times faster) are for the pre-PR pass. The sweep runs with no `RUSTC_WRAPPER`, because every mutant changes the source, so a build cache such as kache only misses and fills its store.

A timeout that repeats is a finding: a mutant reached an unbounded loop. Bound such a loop with a `for` over a fixed range. A `while n <= CEILING` loop still hangs once the mutant stops incrementing `n`. See `MAX_CATCH_UP_STEPS` in `recur/dates.rs` for an example.

## Why CI doesn’t run it

Every mutant re-runs a suite with subprocess and PTY tests, and a shared CI runner is where `TIMEOUT` verdicts are least reliable. An advisory check with unreliable verdicts teaches people to ignore it, and a cold build on two runner cores would make it the slowest job. Run it locally, where a timeout means something.

## Triage

Every survivor is a real gap or an equivalent mutant.

- For a real gap, write a test that kills it. If you won’t fix it in this change, leave it to `just ask-refactoring`, which reads the last sweep’s outcomes and files one task per survivor.
- An equivalent mutant changes nothing a test can observe. Record it in `.cargo/mutants.toml` under `exclude_re` or `exclude_globs`, and start its comment with the class name below, so you can grep each class.

The classes:

- type-tautology: a comparison the type already decides, such as `>= 0` on a `usize`. Exclude it. In review, prefer `x != 0` to `x > 0` on an unsigned count, because its mutants stay killable. No lint catches this class, because the equivalence lies between the mutant and the source.
- pure-memoisation: a guard that only skips repeated work, so deleting it changes the running time and never the answer. Exclude it.
- discarded-effect: extra work a later step throws away, such as one more loop pass whose result a final `resize` drops. Exclude it, or rewrite the source so the extra step can’t happen.
- aliased-input: two inputs that name the same thing, such as `""` and `"."` for the current directory. Exclude it.
- same-call: the mutant replaces a body with the call it already is, such as `Default::default()` for `Self::default()`. Exclude it, or delete the function when it isn’t public API.
- no-public-constructor: the branch needs a value the dependency gives no way to build. Exclude it.
- platform-specific: the difference exists on another OS only. Exclude it, or gate a test to that platform.
- entry-point: a thin `main.rs` with no logic to check. Exclude the file.

Leave no survivor untriaged. The sweep is worth running because every name left on its list has a stated reason.
