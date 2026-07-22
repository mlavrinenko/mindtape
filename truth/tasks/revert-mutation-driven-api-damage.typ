#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "revert mutation-driven API damage",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  tags: ("cli", "tests"),
  links: related("adopt-cargo-mutants-mutation-testing.typ")[
    the burn-down that introduced the damage
  ],
  status: done(
    2026,
    7,
    21,
  )[four void signatures restored; 4/4 in-diff mutants caught, no new exclusion],
)

== Summary

Four side-effect functions grew a return value that no caller consumes, purely
so a cheap model could kill a `replace <fn> with ()` mutant with a one-line
assertion. That is test-induced design damage: the mutation score improved by
widening the code under test, and the resulting tests are tautologies. Restore
the void signatures and kill the same mutants through the observable effect.

== Scope

Revert these four signatures to their pre-branch form:

- `crates/mindtape-cli/src/util.rs` `warn_errors(&LoadOutcome) -> usize` — nine
  callers (`ls`, `check`, `flip`, `rm`, `go`, `graph`, `recur`, `review`)
  discard it. The test `warn_errors_returns_error_count` asserts
  `outcome.errors.len() == outcome.errors.len()`; delete it. The function's real
  behaviour (printing to stderr) is still unpinned — either pin it in a
  subprocess test or take an injected `&mut dyn Write`, which is the better
  design anyway.
- `crates/mindtape-cli/src/util.rs` `init_worktree(Option<PathBuf>) -> bool` —
  leaks `OnceLock::set`'s already-set state into the API. Kill the mutant by
  asserting `worktree_override()` after the call instead.
- `crates/mindtape-cli/src/render/color.rs` `init(ColorChoice) -> bool` — same
  shape; observe through `is_color_active()`.
- `crates/mindtape-cli/src/cli.rs` `init_logger(u8) -> LevelFilter` — `main.rs`
  had to be edited to `let _ = cli::init_logger(...)`, which is the tell. Keep
  the extracted pure `verbosity_level(u8) -> LevelFilter` (a genuine
  improvement, directly testable) and drop the return from `init_logger`.

== Verification

Re-sweep each touched file with `cargo mutants -f <file>` *without* `--iterate`.
A mutant that is genuinely unobservable after the revert goes to
`.cargo/mutants.toml` with its reason — an honest exclusion beats a dishonest
signature.
