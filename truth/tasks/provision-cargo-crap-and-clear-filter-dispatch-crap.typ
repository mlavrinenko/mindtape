#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "provision-cargo-crap-and-clear-filter-dispatch-crap",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 5,
  ),
  tags: ("tooling",),
  status: done(
    2026,
    7,
    22,
  )[cargo-crap 0.3.1 built from the crates.io tarball via naersk singleStep and added to the dev shell; CI runs `just crap` after `just cover` in the coverage job. The breaching arms of filter.rs `function` are covered (new `filter\_dispatch\_tests.rs`, a child module of `filter\_tests.rs` so `sample()`/`hit()` are shared), so it scores 28.0 at 100% — the CC floor. 972 functions analysed, none above 30.],
)

== Summary

`just crap` (CRAP-metric gate) is documented in the Justfile but cargo-crap is provisioned nowhere — it is absent from flake.nix, from nixpkgs, and from CI, so the recipe has never been runnable in the dev shell. Verified by installing cargo-crap 0.3.1 by hand against the new cargo-llvm-cov lcov: the report parses fine (absolute SF paths resolve, main.rs is already excluded from the lcov), but one function breaches the threshold — filter.rs `function`, CC 28 at 83.7% line coverage, CRAP 31.4 against a limit of 30. It is a flat match dispatcher over filter function names, so the CC is dispatch breadth, not real branching depth; the honest fix is covering the unexercised arms rather than raising the threshold.

== Scope

- decide whether `just crap` is a real gate: either package cargo-crap into flake.nix (not in nixpkgs; naersk `singleStep`, per the crates.io UA block on fetch-cargo-vendor) and wire it after the CI coverage job, or drop the recipe
- if kept: cover the missed arms of filter.rs `function` / its helpers (llvm-cov `--show-missing-lines` names them: 243, 261, 283, 288-291, 294, 313, 339, 346-347) to push CRAP back under 30
- re-run `just cover` then `just crap` to confirm
