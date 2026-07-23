#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Print the config JSON schema to stdout",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 3),
  status: done(2026, 7, 23),
)

== Summary

mt install writes config.schema.json to the app config directory, but nothing prints it, so the docs build cannot capture it. Add a stdout emitter reusing the same generator.

== Scope

- Add a config schema subcommand that prints the JSON Schema to stdout
- Reuse the existing generator so the printed schema and the written file cannot diverge
- Cover it with a test
