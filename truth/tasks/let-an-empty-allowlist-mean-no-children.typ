#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Let an empty allowlist mean no children",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("substrate", "gating"),
  estimate: (files: 5),
  assignee: "agent",
  status: cancelled(
    2026,
    9,
    15,
  )[Backlog review 2026-09-15: a cross-repo release only to respell a subsection rule that already works.],
)

== Summary

Forbidding subsections under a section is naturally `action = "allowlist"` with the section as `scope` and an empty `allowed`, but artifact-gate refuses it: `Rule::allow` returns `RuleError::EmptyPath` when `allowed` is empty (artifact-gate 0.1.0, `src/lib.rs:208`). This project's config works around it with `action = "forbid"` over a wildcard path pair, which is correct and reads as a trick rather than as the rule it is.

== Scope

Work in the artifact-gate repository (github.com/mlavrinenko/artifact-gate), not this one. Make an empty `allowed` mean "no children permitted": under a `Some` scope no node the path matches may have a child, and under a `None` scope no root node is permitted. A `Some` scope that is itself empty stays `RuleError::EmptyPath`, since a scope matching nothing is a typo rather than an intent.

```toml
# today
action = "forbid"
path = [{ match = ".*" }, { match = ".*" }]

# after
action = "allowlist"
scope = [{ match = ".*" }]
allowed = []
```

Red test first: an empty allowlist under a scope rejects a child and accepts a childless node. Update the doc comment on `Rule::allow`, add a CHANGELOG entry, and bump the crate to 0.1.1 in `Cargo.toml` and `Cargo.lock`, additive because a call that used to fail now succeeds. Commit there following that repository's own conventions, and stop: tagging, pushing and publishing are `publish-artifact-gate-0-1-1.typ`.

== Dogfood

The replacement lands in this repo's `.mindtape/config.toml` in `adopt-the-empty-allowlist-from-artifact-gate-0-1-1.typ`.
