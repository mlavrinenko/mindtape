# Mutation testing

Deep dive behind one line of CONTRIBUTING.md's Testing section: `cargo-mutants`.

## What it buys

Coverage proves a line executed. Mutation testing proves a test would fail if
that line were wrong. A surviving `MISSED` mutant is a real assertion gap — code
your suite runs but does not pin.

## How to run it

- `just mutants-diff` before a PR. It sweeps only the mutants your branch's diff
  touches, so it is minutes rather than hours, and covers new files for free.
- `just mutants -f <file>` forces a deliberate full-file sweep.

Neither is a `just check` gate, and there is no CI job for it on purpose. The
reasoning, the `TIMEOUT` caveat, and how to bound a loop a mutant can hang are in
the `mutants` recipes' comments and in `.cargo/mutants.toml`.

## Triage

Every survivor is one of two things:

- A real gap — write a regression test that kills it.
- An equivalent mutant, where the mutated code is genuinely indistinguishable
  from the original. Record it in `.cargo/mutants.toml` `exclude_re`, with its
  reason.

Never leave a survivor untriaged; the value of the sweep is that the list is
empty for a stated reason, not that it is short.
