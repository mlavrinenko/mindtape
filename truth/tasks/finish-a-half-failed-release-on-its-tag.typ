#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a release tag that cannot finish, and finish one on its tag",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a release that cannot finish is refused before its tag exists],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[a failed release run is finished by a re-run, not by a new version],
      docs-match-code: helps[CONTRIBUTING states what `just release` refuses and how to re-run a tag],
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    7,
  )[release-dry-run refuses before tagging; the workflow re-runs a tag and skips published crates],
)

== Problem

+ `mindtape` 0.2.0 was published to crates.io by hand before `v0.2.0` was pushed.
+ 🔴 The tag's four release runs all failed at `cargo publish`, so `v0.2.0` has no GitHub release, and `mindtape-cli` has never reached crates.io.
+ 🔴 `release.yml` runs only on a tag push, so nothing can finish that release on the same tag; outdatty v0.6.0 and two typst crates hit the same wall.
+ 🔴 `just release` tags without checking that crates.io lacks the version, that the tag is new, or that CI passed on the commit.


== TO-BE

- 🟢 `release.yml` skips `cargo publish` for each crate whose version crates.io already has, and still creates the GitHub release.
- 🟢 `gh workflow run release.yml -f tag=vX.Y.Z` re-runs an existing tag, every job checking out that tag; a re-released old tag does not take "Latest".
- 🟢 `just release X.Y.Z` refuses, with an `error:` and a `hint:` line, a dirty tree, a branch other than main, main differing from origin/main, an existing tag, a version on crates.io, no dated CHANGELOG section, a HEAD without green CI, a failing gate or `cargo publish --dry-run`. `just release-dry-run X.Y.Z` runs every check and stops before tagging.


== Changes

- `.github/workflows/release.yml`: `workflow_dispatch` with a `tag` input, one run per tag, a meta job, idempotent publish per crate, an explicit release tag.
- `.just/scripts/release.sh`: the preflight, and a `--dry-run` mode the confirm handshake does not gate.
- `Justfile`: `release-dry-run`, and the release doc comment.
- `docs/src/contributing.typ`, `CONTRIBUTING.md`: the release rules.


== Rollback

- Revert the commit. No file format or config key changes.


== Out of scope

- Repairing `v0.2.0`: its tree has another crate layout and a nixpkgs pin from before the crates.io download fix, so this workflow's check job cannot pass on it.
- Releasing 0.3.0.


== Dogfood

The next release of this repo goes through `just release-dry-run` and `just release`.

