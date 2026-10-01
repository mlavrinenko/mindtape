#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Hold .cargo/mutants.toml to the tombi format check",
  priority: ice(
    impact: 2,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 2),
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
      sound-codebase: helps[every TOML the gate reads holds to one format],
      docs-match-code: neutral,
    ),
  ),
  kind: "dev",
  assignee: "agent",
  status: done(2026, 10, 1),
)

== Problem

+ Read `Justfile`'s `TOML_FMT_PATHS`.
+ 🔴 `.cargo/mutants.toml` is absent, so `tombi format --check .cargo/mutants.toml` fails today and no gate notices.


== TO-BE

- 🟢 `just fmt-check` holds `.cargo/mutants.toml` to tombi's format.


== Changes

- `Justfile`: add `.cargo/mutants.toml` to `TOML_FMT_PATHS`; `just fmt` drops the blank lines between its comment blocks.
- `CONTRIBUTING.md`: regenerated, it renders the list.

== Rollback

Drop the path from `TOML_FMT_PATHS`.


== Out of scope

- The other drifting TOML (`Cargo.toml`, the crate manifests, `defaults.toml`, `frameworks.toml`): tombi re-sorts their keys by schema and detaches the group comments, a separate decision.


== Dogfood

The next exclusion added to `.cargo/mutants.toml` is formatted by `just fmt` like every other config.

