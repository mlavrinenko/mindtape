#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Check estimate drift when a task closes",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("mt", "cli", "gating"),
  estimate: (files: 7),
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: related(
    "record-the-artifacts-a-task-expects-to-touch.typ",
  )[the artifacts field is what this check would compare against],
)

== Summary

`check_estimate_drift` runs from exactly one place, crates/mindtape-cli/src/check.rs:81, so the comparison between a declared file budget and the working tree happens on `mt check` and never during a flip. The wip to done flip is the one moment that comparison is complete rather than partial: before it the diff is still growing, after it the tree has already moved on. `[[on.flip]] when` sees the task as it stands before the write, so a task closing out of wip still matches the default `in-flight` filter `status == "wip"` at exactly that moment — the filter needs no new semantics to fire there.

== Scope

Give `[[on.flip]]` actions a built-in check alongside `run`:

```toml
[[on.flip]]
when = 'into("done")'
actions = [
  { check = "estimate-drift", gate = true, message = "the diff outgrew the declared budget" },
]
```

`check` reads the already-loaded project instead of shelling out to reload it, and `gate` alone decides whether drift blocks the close or merely reports — drift is warn-only today, so without it the check prints and lets the flip through.

Exclude the task root from the count at flip time: `mt flip` writes the very task file it is gating and would otherwise inflate its own number by one, the same reason `.mmz/config.yaml` excludes `truth/tasks/`.

Red test: a wip task declaring two files, closed against a four-file tree, reports the drift.
