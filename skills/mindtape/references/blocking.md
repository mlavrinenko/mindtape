# Recording an external blocker

Reference for the mindtape skill. Read this only when a task waits on
something outside the backlog, such as a release or a review.

`mt ls blocked --columns title,why` shows what each blocked task waits on. A
legend may have no `blocked` status, so check `mt flip --list` before you use
one, and never move a blocked task to `wip`: a WIP cap counts it.

Run `mt config show` and look for `[[tasks.waits]]` checkers. If one fits, add
the condition with `mt set <task> waits-on+=<checker>:<id>`, and the task stays
blocked until that checker exits 0. If none fits, file a task for the blocker
and link to it with `depends-on`, or leave the task in a status before work
starts, with the reason as its note.

A checker runs only once the user has allowed it, so a fresh clone reads
`unknown: not allowed` until then.
