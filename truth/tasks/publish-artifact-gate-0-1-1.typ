#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Publish artifact-gate 0.1.1",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("substrate",),
  estimate: (files: 1),
  kind: "dev",
  assignee: "max",
  status: proposed(2026, 9, 14),
  links: depends-on("let-an-empty-allowlist-mean-no-children.typ"),
)

== Summary

`let-an-empty-allowlist-mean-no-children.typ` commits the empty-allowlist semantics and the 0.1.1 bump in the artifact-gate repository without releasing them. Pushing and publishing is the maintainer's step, and this repo cannot adopt the version until crates.io serves it.


== Scope

In the artifact-gate repository, push `main`, then tag `v0.1.1` and push the tag. Its `release.yml` workflow publishes on a `v*` tag and refuses a tag that disagrees with `Cargo.toml`. Done when crates.io lists artifact-gate 0.1.1.


== Dogfood

Not applicable: a release step. The new version is dogfooded in `adopt-the-empty-allowlist-from-artifact-gate-0-1-1.typ`.

