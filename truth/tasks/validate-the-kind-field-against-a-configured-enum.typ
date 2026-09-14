#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Validate the kind field against a configured enum",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("mt", "gating"),
  estimate: (files: 6),
  assignee: "agent",
  status: done(2026, 9, 14),
)

== Summary

`kind` is a core task field (lib/prelude.typ:646) with no validation: there is no KindConfig in schema/config.schema.json and no [tasks.kind] key in TasksConfig, so any string passes. The generic [tasks.fields.<name>] mechanism already exists and its schema doc names `kind` as its subject, but no config declares it and no shipped template sets the field — so template identity is lost the moment mt add writes the file, and no per-shape header or content rule can ever fire.

== Scope

Declare the vocabulary in `.mindtape/config.toml`:

```toml
[tasks.fields.kind]
values = ["task", "umbrella"]
```

Have every template in `.mindtape/templates/` set `kind:`, and confirm `crates/mindtape/src/fields.rs` and `validate_tests_fields.rs` gate an unknown kind. Cover the enum with a red test before the config lands.
