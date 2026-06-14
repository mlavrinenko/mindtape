#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Dogfood MindTape on its own truth/tasks",
  status: done(2026, 6, 14),
  priority: "high",
  tags: ("tooling", "docs"),
)

== Summary

Make MindTape track its own backlog. Add `.mindtape/config.toml` (roots =
`truth/tasks`) and seed the backlog as per-file task artifacts, so `mindtape
list` and `mindtape check` run against this repo via the in-tree `lib/` prelude.

== Outcome

- `.mindtape/config.toml` constrains status / priority / estimate-hours / tags.
- The deferred items from the per-file rewrite are now backlog tasks here.
- `Refs:` footers point at the driving task file under `truth/tasks/`; ADRs stay
  the reference for architectural decisions. See `CONTRIBUTING.md`.
