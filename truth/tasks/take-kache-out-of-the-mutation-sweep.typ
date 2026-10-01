#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Take kache out of the mutation sweep",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.6,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("tooling", "tests"),
  estimate: (src-files: 1),
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
      sound-codebase: helps[sweeps stop flushing the shared build cache and stop paying for keys that can never hit],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 1),
)

== Problem

+ Run `just mutants` with kache as `RUSTC_WRAPPER` (the developer's shell sets it session-wide), then split `~/.cache/kache/events.jsonl` by `root` and `result`.
+ 🔴 The 2026-09-30 sweep, summed over parallel rustc processes: hits avoided about 400 s of compile; 1326 misses on `mindtape`, `mt` and `*_e2e` targets cost about 28 min of keying and storing; 2442 adaptive passthroughs still cost about 25 min of keying. The misses wrote 146.5 GB through the 60 GB store, evicting what normal builds hit.
+ The misses are inherent: every mutant edits `crates/mindtape`, so it and its dependents re-key. Only registry deps can hit, and cargo reuses them per scratch copy after the first build.


== TO-BE

- 🟢 A sweep runs plain cargo: no kache keying or storing per mutant, nothing written to `~/.cache/kache`.
- 🟢 Normal builds keep their kache hits across a sweep, because the sweep no longer flushes the store.
- 🟢 The change lands on a measured sample, not on the numbers above alone.


== Changes

- `.just/scripts/_mutants-env.sh`: `export RUSTC_WRAPPER=`, empty, which cargo reads as no wrapper and which overrides config. A comment carries the numbers from Problem. Script-local, like the other knobs there.
- `docs/src/contributing/mutation-testing.typ` and its regenerated `.md`: one line beside the other sweep knobs, if that doc lists them.
- Verify without a full sweep (about 15 h): pick one source file with about 20 mutants (`cargo mutants --list -f <file>`), run that restricted sweep twice from a cold scratch dir, with and without the wrapper. Compare wall time from `mutants.out/outcomes.json`, the `kache stats` entry delta and `du -sh ~/.cache/kache`. Keep the change only if wall time does not regress.


== Rollback

Delete the `RUSTC_WRAPPER=` line and its comment from `.just/scripts/_mutants-env.sh`.


== Out of scope

- `KACHE_INCREMENTAL_CRATES` set to the workspace crate names: it matches rustc's `--crate-name`, so it would have to list all ~55 test targets and would go stale with every new test file.
- kache's `cache.exclude = ["/tmp/cargo-mutants-*/**"]`: right shape, but it is host-wide kache config that lives in the NixOS config, not this repo, so other contributors would not get it.
- `KACHE_DISABLED=1`: makes kache a transparent shim but still strips incremental flags unless `KACHE_PRESERVE_INCREMENTAL=1` is set too, so it takes two knobs to get what an empty wrapper gives.
- `CARGO_MUTANTS_JOBS`, `CARGO_BUILD_JOBS` and the mold flag: tuned and measured separately, so left alone.


== Dogfood

`just mutants` and `just mutants-diff` both source `_mutants-env.sh`, so every sweep in this repo runs without kache from the next one on.

