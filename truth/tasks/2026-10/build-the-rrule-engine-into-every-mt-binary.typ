#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Build the rrule engine into every mt binary",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt", "tooling"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a calendar rule works in the binary a user installs, with no rebuild.],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[one build and one test run instead of a feature arm, its stub and its CI job.],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    12,
  )[the engine is in every binary.],
)

== AS-IS

+ Install `mt` with `cargo install mindtape-cli` and write this in a task:

  ```typ
  recur: rrule("FREQ=MONTHLY;BYDAY=1MO")
  ```
+ 🔴 `mt check` refuses it, because release builds leave out the `rrule` feature, and the quickstart warns about it.


== TO-BE

- 🟢 Every `mt` binary expands `rrule(...)`, and the release binary grows by about 2 MiB.
- 🟢 The change drops the `rrule` feature, its stub, recipe and CI job. The engine runs under `just check`.
- 🟢 The quickstart and Recurring pages drop the feature.


== Changes

- `crates/mindtape/src/recur/mod.rs`: drop the `cfg` arms and the stub module.
- `crates/mindtape/src/recur/dates.rs`: drop the `cfg` arms.
- `crates/mindtape-cli/src/cmd/init/wizard/axes.rs`: offer the rrule answer always.
- `crates/mindtape/Cargo.toml` and `crates/mindtape-cli/Cargo.toml`: the crates depend on `rrule` directly, and the features go.
- `Justfile`, `.github/workflows/ci.yml`, `flake.nix` and the gate scripts: drop `check-rrule` and the feature flags.


== Alternatives

- Keep the feature and enable it in release builds: two builds to test for one binary anyone installs.


== Out of scope

- The `every(...)` recurrence, which stays the common path.


== Dogfood

This repo's own `mt` gains the engine, and `just check` covers it.
