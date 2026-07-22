#import "@local/mindtape:0.2.0": *

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
  )[cover now runs cargo-llvm-cov: 94.50% lines (tarpaulin read 89.06%), floor raised 70 -> 90; tarpaulin.toml deleted, flake exports LLVM\_COV/LLVM\_PROFDATA from llvmPackages\_21.],
)

== Summary

cargo-tarpaulin cannot attribute coverage of code executed in a child process, and MindTape's CLI tests are overwhelmingly subprocess-based (15 assert_cmd e2e targets plus the self-re-exec run_child pattern used to isolate env globals). Every command entry point therefore reads as uncovered even though it is tested. Measured on this tree: tarpaulin reports 89.06% (822 missed), cargo-llvm-cov reports 95.30% (530 missed), and the remaining misses are thin scattered branches with no cluster above 29 lines in one file. Switch just cover to cargo-llvm-cov, which merges per-process profraw and sees across exec.

== Scope

- flake.nix: cargo-tarpaulin -> cargo-llvm-cov + llvmPackages_21.llvm (rustc 1.92 is LLVM 21); export LLVM_COV/LLVM_PROFDATA in shellHook since nixpkgs rustc ships no llvm-tools-preview
- Justfile: cover recipe -> cargo llvm-cov --workspace --lcov --output-path target/coverage/lcov.info, keeping the stdout summary
- delete tarpaulin.toml; move fail-under and the main.rs exclusion onto the recipe (--fail-under-lines, --ignore-filename-regex)
- just crap keeps consuming target/coverage/lcov.info unchanged
- CI needs no edit: the coverage job already runs just cover
- refresh the tarpaulin mentions in CONTRIBUTING.md and AGENTS.md; run just outdatty-update if the Justfile group drifts
- raise the coverage floor once the real number is known (70 was set against an under-reporting tool)
