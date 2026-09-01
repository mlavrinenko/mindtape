#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "adopt cargo-nextest as the test runner",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 10),
  assignee: "agent",
  links: related(
    "verify-the-windows-nextest-job-on-github.typ",
  )[the `cargo test` call that survives locally — only a pushed run proves it],
  status: done(
    2026,
    9,
    1,
  )[all gates green locally; only the Windows CI run is unverified],
)

== Summary

`cargo test` prints every test binary's own "running N tests" header and
result block — a green full run is ~2000 lines of noise, and a filtered query
that matches nothing still prints 35 empty result blocks. cargo-nextest runs
each test in its own process and reports from a test-level summary, so the
same run is seven lines. MindTape has been on plain `cargo test` while
cratemplate and kimmable both moved; this adopts the same shape.

`just test`, `just count-tests`, `just check-rrule` and `just cover` all move
to nextest, so one runner is both gated and measured. nextest runs no
doctests; the tree has none (`cargo test --doc` reports 0 in both crates, and
no code-fenced doc comment exists), so no separate pass — the cratemplate
fallback — is wired, and a doctest added later must come with a pass of its
own. CI's Windows job installs the same runner via `taiki-e/install-action`
(pinned to a commit SHA), and the dev shell adds `cargo-nextest`.

== Verification

- `just test`: 2137 tests, 7 lines of output.
- `just count-tests` == the run count (2137), via `nextest list`, no run.
- `just check-rrule`: 1116 tests under the `rrule` feature.
- `just cover`: 94.64% total through `cargo llvm-cov nextest`, stable across two
  runs.
- `just docs::md` + `docs::md-check`: CONTRIBUTING.md gate table re-derived.
- `just check`: all arms green (after the CRAP baseline re-mint, see the
  re-mint commit).
