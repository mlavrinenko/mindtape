#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Document the estimate-drift flip check",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("mt", "docs"),
  estimate: (files: 5),
  kind: "docs",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on("check-estimate-drift-when-a-task-closes.typ"),
)

== Summary

`check-estimate-drift-when-a-task-closes.typ` adds a built-in `check` action to `[[on.flip]]`, pinned by unit tests only. Nothing drives it through the real binary, and the hooks and estimates pages still describe `show`, `list` and `run` as the only action kinds.


== Scope

Add an end-to-end case to `crates/mindtape-cli/tests/flip_hooks_e2e.rs`: a wip task declaring two files, closed against a four-file tree under a gating `check = "estimate-drift"` action, is refused. Regenerate `schema/config.schema.json`, document the `check` action in `www/content/hooks.typ` and the close-time comparison in `www/content/estimates.typ`, and add a CHANGELOG.md entry.

