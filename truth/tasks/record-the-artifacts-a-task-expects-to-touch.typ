#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Record the artifacts a task expects to touch",
  kind: "feat",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("mt", "gating"),
  estimate: (files: 3),
  assignee: "agent",
  status: cancelled(
    2026,
    9,
    15,
  )[Backlog review 2026-09-15: git already records the files a task touched, and nothing would consume a declared list.],
)

== Summary

Naming the files is the most-cited field in agent-ticket practice: Anthropic's own guidance is to name the files and interfaces involved, and blustroke's Files section proves the content is data rather than prose, a bare path list with a median of 8 words. Its value is directing the implementer, not validating the spec, so checking that a path exists is the wrong check. A spec legitimately names a file it will create, and a closed task legitimately names one a later refactor deleted.

== Scope

Add a dedicated `artifacts` field to `task`, separate from `links:`, built in the prelude's packager idiom:

```typ
artifacts: created("crates/mindtape/src/a.rs") + changed("crates/mindtape/src/*.rs"),
```

`links:` stays edges between tasks. A path in the working tree is not a task, and folding paths into `links:` would hand every edge consumer (link resolution, `mt rm` tombstones, `topo` and `blocks`) a second kind of target to skip.

Split: this task is the call shape, which is where the decision lives. Touches `lib/prelude.typ` (the `artifacts` parameter, `created` and `changed` packagers that `+` combines the way edge helpers do, and the call-shape checks for a missing or non-string path), the `artifacts` entry in `www/utils/dsl-field-notes.typ` that the doc-facts gate demands as soon as the parameter exists, and red tests in `crates/mindtape/src/loader_tests_prelude.rs`. Reading, typing and the glob rule follow in `read-the-artifacts-field-into-the-task-model.typ`, the drift comparison in `compare-declared-artifacts-against-the-working-tree.typ`, and the docs in `document-the-artifacts-field.typ`.

== Dogfood

This backlog declares `artifacts` from `compare-declared-artifacts-against-the-working-tree.typ` on.
