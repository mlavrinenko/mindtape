#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "switch-coverage-to-cargo-llvm-cov",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 6,
  ),
  tags: ("tooling",),
  status: done(
    2026,
    7,
    21,
  )[cover runs cargo-llvm-cov: 94.50% lines (tarpaulin read 89.06%), floor raised 70 -> 90. tarpaulin.toml deleted, flake exports LLVM\_COV/LLVM\_PROFDATA from llvmPackages\_21.],
)

== Summary

cargo-tarpaulin cannot attribute coverage of code executed in a child process. This project's CLI tests are overwhelmingly subprocess-based (`assert_cmd` e2e targets plus the self-re-exec `run_child` pattern that isolates environment globals). Every command entry point reads as uncovered even though a test covers it. Measured on this tree, tarpaulin reports 89.06% (822 missed) and cargo-llvm-cov reports 95.30% (530 missed). The remaining misses are thin scattered branches with no cluster above 29 lines in one file. Switch `just cover` to cargo-llvm-cov, which merges per-process `profraw` and sees across exec.

== Scope

- flake.nix: cargo-tarpaulin -> cargo-llvm-cov + llvmPackages_21.llvm (rustc 1.92 is LLVM 21). Export `LLVM_COV`/`LLVM_PROFDATA` in `shellHook`, because the nixpkgs rustc omits llvm-tools-preview.
- Justfile: the cover recipe runs `cargo llvm-cov --workspace --lcov --output-path target/coverage/lcov.info`, keeping the stdout summary
- delete tarpaulin.toml and move fail-under plus the main.rs exclusion onto the recipe (`--fail-under-lines`, `--ignore-filename-regex`)
- just crap keeps consuming target/coverage/lcov.info unchanged
- CI stays as it is: the coverage job already runs `just cover`
- refresh the tarpaulin mentions in CONTRIBUTING.md and AGENTS.md. Run just outdatty-update if the Justfile group drifts.
- raise the coverage floor once the real number lands (70 came from an under-reporting tool)
