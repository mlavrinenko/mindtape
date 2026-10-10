#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Release through cargo-release instead of a hand-rolled script",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[every refusal the release script made still stops a release, now through cargo-release or its hook],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a maintained tool owns the version bump, the CHANGELOG dating, the commit and the tag, where a 120-line shell script did],
      docs-match-code: helps[CONTRIBUTING and the Justfile describe the same release steps],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Read `.just/scripts/release.sh` and the `release` and `release-dry-run` recipes.
+ 🔴 Moving `[Unreleased]` into a dated `CHANGELOG.md` section, and bumping `[workspace.package] version` and `mindtape-cli`'s `mindtape` dependency, are hand steps the script only checks for.
+ 🔴 The script re-implements the bump check, the tag, the push and the remote-behind check in 120 lines of bash that no one else maintains.


== TO-BE

- 🟢 `release.toml` holds the cargo-release config: one `v<version>` tag, a shared workspace version, `publish = false` because `release.yml` publishes on the tag, and `pre-release-replacements` that date `[Unreleased]` and open a fresh one.
- 🟢 `just release X.Y.Z [--dry-run]` bumps, replaces, runs the hook and commits with cargo-release's steps, pushes `main`, waits for `ci.yml` on that commit, then tags and pushes the tag.
- 🟢 Every refusal the script made still stops a release with its message and hints, through cargo-release itself or the pre-release hook.
- 🟢 `.just/scripts/release.sh` is gone, and `cargo-release` is in the dev shell.


== Changes

- `release.toml`: the cargo-release config.
- `Justfile`: `release` on the step subcommands, and `release-dry-run` folded into `--dry-run`.
- `.just/scripts/release.sh`: deleted, with a hook script for the checks cargo-release lacks.
- `flake.nix`: `cargo-release` in the dev shell.
- `docs/src/contributing.typ`, `CONTRIBUTING.md`, `outdatty.yaml`, `.mmz/config.yaml`: the release rules and the gate scopes that named the script.


== Rollback

- Revert the commits. No file format or config key changes.


== Out of scope

- `release.yml`: it still publishes on the tag, and `cargo-release` never publishes here.
- Releasing 0.3.1 or any other version for real.


== Dogfood

The next release of this repo goes through `just release X.Y.Z --dry-run` and `just release X.Y.Z`.

