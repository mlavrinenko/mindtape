#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Ship named rule packs in the binary",
  kind: "feat",
  priority: ice(
    impact: 9,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 7,
  tags: ("mt", "gating", "tooling"),
  estimate: (files: 5),
  assignee: "agent",
  status: proposed(2026, 9, 14),
)

== Summary

The prose and spec-shape rules a project wants are a dozen-odd `[[tasks.body.content.rules]]` entries with hand-tuned regexes. Copy-pasting them per project means every repo drifts and none of them ever get the next fix. `[dicts]` already solved this exact problem for word sets: two dictionaries ship embedded under the reserved `mindtape-` prefix (crates/mindtape/src/config_dicts.rs), are always compiled whether declared or not, and a project unions them in with `include`. A preset is a one-time copy and cannot update; a pack ships with the binary and does.

== Scope

Do for rules what `BUILTIN_PREFIX` did for dictionaries: embed named rule packs and let a project union them in:

```toml
[tasks.body.content]
include = ["mindtape-plain-prose"]
```

Reserve the prefix against project packs, and give every bundled rule an id (`artifact_gate::Rule::with_id` already carries one) so a project can disable a single rule it disagrees with rather than abandoning the pack. This task is the mechanism only, proven with a test-only pack: the `include` and disable keys on `ContentConfig` in `crates/mindtape/src/body_content.rs`, compiling pack rules ahead of the project's in `crates/mindtape/src/content_gate.rs` with `content_gate_tests.rs`, and a new embedded registry module wired through `crates/mindtape/src/lib.rs`. The first real pack is `write-the-mindtape-plain-prose-rule-pack.typ`; the schema and docs are `document-named-rule-packs.typ`.

Dogfood: include `mindtape-plain-prose` in this repo's `.mindtape/config.toml`, landed in `write-the-mindtape-plain-prose-rule-pack.typ`.
