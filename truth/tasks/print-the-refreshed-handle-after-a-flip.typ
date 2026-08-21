#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "print the refreshed handle after a flip",
  priority: ice(
    impact: 4,
    confidence: 0.85,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 3),
  links: related("configure-the-handle-check-length-per-project.typ")[the guard
    that catches a stale ordinal; this one keeps it from going stale unnoticed],
  assignee: "max",
  status: done(
    2026,
    8,
    21,
  )[one pre-batch and one post-batch default view; a moved handle prints, an unmoved one stays silent, and a task leaving the view names its stem],
)

== Summary

A handle's ordinal is the task's position in the default view, recomputed on
every command. A flip re-sorts that view — \`proposed\` to \`wip\` moves a task up
past everything still proposed — so the handle the caller just used is stale
the moment it succeeds, and every handle it displaced is stale too. The caller
finds out on the next command, from a check mismatch that names the task now
occupying the ordinal rather than the one they meant.

Nothing about the flip's own output says the numbering moved. \`flip\_target\`
has the \`Project\`, so recomputing the default view once per batch and printing
each touched task's new handle closes the loop where it opened, at the cost of
one extra view computation on a command that already writes files.


== Scope

- After a flip batch, print the affected task's refreshed handle beside the
  transition it already reports
- Recompute the view once for the whole batch, not once per target
- Say nothing extra when the handle did not move — the common case for a
  same-status re-stamp, and noise there would train the reader to skip the line
- A task the flip filtered OUT of the default view has no handle to print;
  name the filename stem instead, since that is what still addresses it

