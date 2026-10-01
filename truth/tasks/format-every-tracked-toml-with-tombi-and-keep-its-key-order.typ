#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Format every tracked TOML with tombi and keep its key order",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 5),
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
      sound-codebase: helps[every tracked TOML is held to one format],
      docs-match-code: neutral,
    ),
  ),
  kind: "dev",
  assignee: "agent",
  status: done(2026, 10, 2),
)

== Problem

+ Run `tombi format --check Cargo.toml clippy.toml crates/mindtape/Cargo.toml crates/mindtape-cli/Cargo.toml crates/mindtape/src/defaults.toml crates/mindtape/src/frameworks.toml examples/self-validating/.mindtape/config.toml` with the dev shell's tombi 0.7.27.
+ 🔴 Every file fails, and `tombi format` re-sorts keys by schema, detaching the `# --- Code Quality ---` groups in `Cargo.toml` from their lints. `TOML_FMT_PATHS` lists six files, so the other tracked TOML is never checked.
+ 0.7.27 has no switch to turn key sorting off short of `[schema] enabled = false`, which also drops editor schema support.


== TO-BE

- 🟢 `just fmt-check` holds every tracked TOML to tombi's format.
- 🟢 tombi never reorders table keys or array values; commented groups stay put.


== Changes

- `flake.nix`: a dedicated nixpkgs input pinned for tombi >= 1.5 only, so typst and tinymist stay put; the tombi comment no longer claims 0.9+.
- `flake.lock`: the new input.
- `tombi.toml`: turns the ordering rules off.
- `Justfile`: `TOML_FMT_PATHS` covers every tracked `*.toml`.
- The TOML files `just fmt` rewrites, and whatever docs or tests show them verbatim.

== Rollback

Revert the commit; the old tombi and the six-path list return.


== Out of scope

- The `check` recipe wiring in the `Justfile`: another change owns it.
- Bumping the main `nixpkgs` input.


== Dogfood

`just fmt` and `just fmt-check` run the one pinned tombi over every tracked TOML.
