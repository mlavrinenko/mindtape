#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Move the sweep templates to the four-section spec",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "tooling"),
  estimate: (files: 5),
  kind: "chore",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on(
    "replace-the-task-section-set-with-a-closed-four-section-spec.typ",
  ),
)

== Summary

`just ask-refactoring` files tasks from four templates that write Summary and Scope. Once the four-section spec is defined, every swept task is filed in the old shape and has to be migrated by hand, and the sweep would block closing the allowlist.


== Scope

Rewrite `.mindtape/templates/sweep-size.typ`, `sweep-mutants.typ`, `sweep-coverage.typ` and `sweep-undertested.typ` into AS-IS, TO-BE, Ruled out and Done when, mapping each finding's measurement to AS-IS and its threshold to Done when. Move the remedy bullets `contrib/ask-refactoring/size.sh` splices into Scope over to TO-BE, and confirm with `mt add --print` per template.

