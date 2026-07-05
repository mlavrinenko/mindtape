#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Repoint mindtape to the published substrate crates and unblock 0.2.0",
  status: proposed(2026, 7, 5)[
    The gate: 0.2.0 does not ship until the substrate crates are published and
    mindtape depends on the crates.io versions, not path deps.
  ],
  priority: ice(impact: 9, confidence: 0.7, ease: 5),
  tags: ("substrate", "tooling", "docs"),
  depends-on: (
    link("extract-swhid-mint.typ"),
    link("extract-artifact-gate.typ"),
    link("extract-typst-cst.typ"),
    link("extract-typst-edit.typ"),
    link("extract-typst-world.typ"),
    link("extract-typst-harvest.typ"),
    link("extract-typst-cleanup.typ"),
  ),
)

== Summary

Once every substrate crate is extracted and published, repoint the mindtape
workspace to the crates.io versions, drop the extracted crates from the workspace,
verify, then tag 0.2.0. This is the release gate the owner set: no 0.2.0 before the
lib crates are published.

== Scope

- Remove the extracted crates from `Cargo.toml` `[workspace] members`
  (typst-world, typst-harvest, typst-cst, typst-edit, typst-cleanup, swhid-mint,
  artifact-gate) and delete their `crates/` dirs. The workspace keeps only
  `mindtape` + `mindtape-cli`.
- In `crates/mindtape/Cargo.toml` and `crates/mindtape-cli/Cargo.toml`, replace the
  `{ path = "../x", version = "0.4.0" }` deps with the PUBLISHED crates.io versions
  (typst-harvest, typst-cst, typst-edit, typst-cleanup, swhid-mint, artifact-gate).
- Keep the `@local/mindtape` dev override working (it lives in mindtape via
  `with_local_package`/typst-world) — verify `mt ls`/`mt check` still resolve the
  in-tree `lib/`.
- Version scheme (the pre-existing blocker, see pre-release-0-2-0-crate-strategy):
  decide tag vs Cargo vs Typst-lib version and fix `just release`'s guard + help
  and the README/`www` version examples so the release string is unambiguous.
- Docs: update CLAUDE.md Workspace section (it says "Four crates" and lists the
  old set) to reflect mindtape + mindtape-cli only, plus the published-substrate
  dependency list; note the substrate repos under Key Docs.

== Tests

- `just check` + `just nix-build` green with the published deps (no path deps to
  extracted crates remain; `cargo tree` shows crates.io sources).
- `mt ls` / `mt check` on `truth/tasks/` unchanged.
- `just release <VERSION>` dry path: guard passes, tag string matches the decided
  scheme.

== Notes

- Only after this is green does 0.2.0 tag. Then remove the CLAUDE.md "Temporary —
  active only on branch" section on merge to main.
- Orchestrator-owned: this is the integration/repoint task, run after all
  extraction agents report their published versions.
