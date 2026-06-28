#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rename CLI binary `mindtape` → `mt`; close outdatty's bin-name gap",
  status: proposed(2026, 6, 28),
  priority: ice(impact: 6, confidence: 0.8, ease: 6),
  tags: ("cli", "tooling", "docs"),
)

== Summary

Rename the produced executable from `mindtape` to `mt` (faster to type). `mt`
collides only with the legacy magnetic-tape control program (`mt` from `cpio` /
`mt-st`), which no mainstream modern distro installs in a default desktop or
server image — present only where someone pulls those packages. The typing win
is the whole point, so only the executable name moves: crate names (`mindtape`,
`mindtape-cli`), the Typst package `@local/mindtape`, the MindTape brand, the
`.mindtape/` config dir, and the `MINDTAPE_ADD_*` env prefix all stay.

Real reason this is filed now: use the rename to test outdatty. Today its graph
does not model the binary / invocation name, so a rename slips past
`just outdatty-check`. Close that gap first, then rename, and let outdatty prove
it now flags every stale doc.

== Blast radius (executable name only)

Bin-name source of truth (drives the executable and clap's help/usage/version):
- `crates/mindtape-cli/Cargo.toml` — `[[bin]] name = "mindtape"`.
- `crates/mindtape-cli/src/cli.rs` — `#[command(name = "mindtape", …)]`, plus the
  `mindtape done` / `mindtape flip` examples in the shortcut-verb help comment.

Emitted invocation strings users read:
- `crates/mindtape-cli/src/default-config.toml` — `mindtape ls/rm/add/go/install-lib`
  comments written out by `mindtape init`.

Test harness pinned to the bin name (or the suite cannot find the binary):
- every `assert_cmd::Command::cargo_bin("mindtape")` (add_tests.rs and any test
  helper hardcoding it) → `cargo_bin("mt")`.

Hand-maintained docs carrying `mindtape <subcmd>` invocations:
- README.md, www/index.html (including the `./target/release/mindtape` build
  path), CONTRIBUTING.md, AGENTS.md (= CLAUDE.md symlink — edit AGENTS.md only),
  CHANGELOG.md (add the rename entry).

Internal `///` doc-comments (not emitted; update for consistency, low priority):
- init.rs, util.rs, ls.rs, and siblings.

== outdatty gap (the finding)

What a rename edits vs. what each current group flags:
- `cli-docs` (cli.rs + lib.rs → README, www): fires only because the clap name
  happens to live in cli.rs. A rename done purely in Cargo.toml would not fire
  it, and it omits CONTRIBUTING / AGENTS / CHANGELOG.
- `release-notes` (root Cargo.toml → CHANGELOG): the bin name lives in the crate
  manifest, not root — no link.
- `dev-docs` / `prelude-docs`: flag CONTRIBUTING / AGENTS only on
  flake.nix / Justfile / prelude.typ changes — unrelated to a rename.

Net: editing the bin name flags at most README + www (incidentally) and silently
leaves CONTRIBUTING.md, AGENTS.md, CHANGELOG.md stale. outdatty fails the test.

Mechanism note: outdatty hashes whole files (blake3 per source; sources may be
globs, no line/regex anchors — confirmed via `outdatty schema`). So the fix is a
new group binding the bin-name sources to every invocation doc, accepting
whole-file granularity.

== Scope

1. outdatty fix first. Add a group `cli-name` to `outdatty.yaml`:
   - source: `crates/mindtape-cli/Cargo.toml`, `crates/mindtape-cli/src/cli.rs`,
     `crates/mindtape-cli/src/default-config.toml`.
   - dependents: README.md, www/index.html, CONTRIBUTING.md, AGENTS.md,
     CHANGELOG.md.
   Tradeoff: whole-file hashing means a dependency bump in the crate manifest
   also re-confirms these docs (a cheap `just outdatty-update` no-op). Accepted —
   the alternative (anchor on cli.rs only) lets a manifest-only rename slip,
   exactly the hole being closed. Then `just outdatty-update` to baseline.
2. Rename the executable:
   - Cargo.toml `[[bin]] name = "mt"`.
   - cli.rs `#[command(name = "mt")]` and the help-comment examples.
   - default-config.toml emitted comment strings → `mt …`.
   - every `cargo_bin("mindtape")` → `cargo_bin("mt")`.
   - internal doc-comments → `mt …`.
3. Update docs: README, www (including the build-output path), CONTRIBUTING,
   AGENTS; add a CHANGELOG entry.
4. `just outdatty-update`; commit `outdatty.lock`.
5. `just check` green (fmt, clippy -D, tests, machete, file-size, jscpd,
   outdatty drift).

== Out of scope (keep — flag if you disagree)

- `.mindtape/` config dir and `MINDTAPE_ADD_*` env prefix: renaming them breaks
  existing on-disk projects and is unrelated to typing speed.
- A `mindtape` compat alias / symlink: deliberately not shipped (hard rename).
  Reopen if discoverability complaints surface.
- Crate names, `@local/mindtape` package, MindTape brand: unchanged.

== Acceptance

- `mt --help` / `mt --version` print `mt` as the program name; no `mindtape`
  invocation survives in README / www / CONTRIBUTING / AGENTS or in emitted
  config and help text.
- Proof outdatty now does its job: with `cli-name` added and baselined, re-running
  the rename edits makes `just outdatty-check` flag all five dependents until
  updated — where today it stays silently green.
