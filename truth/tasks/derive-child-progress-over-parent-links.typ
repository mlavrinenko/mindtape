#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Derive child progress over parent links",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("mt", "cli"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent sees which parent is ready to close without walking the graph],
      enforced-practice: helps[a parent cannot close over unfinished children without --force],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 9, 28),
)

== Problem

+ `rg -l 'parent\(' truth/tasks | wc -l` counts 23 tasks with a parent.
+ 🔴 Nothing reports how far a parent's children have got: `mt ls` has no `children` or `progress` field, and `mt done <parent>` succeeds while its children are still open.


== TO-BE

- 🟢 Computed fields `children` (count of tasks whose `parent` edge names this one) and `progress` (terminal children / children, `0..1`), filterable, sortable and available as `ls` columns and in `--json`, beside `topo` and `blocks`:

```
$ mt ls 'children > 0' --columns title,status,progress
#   TITLE                          STATUS   PROGRESS
1a  Ship the import pipeline       🔨 wip   3/5
$ mt ls 'progress == 1 && !terminal'
```

- 🟢 Flipping a parent into a status that requires its work finished is refused while a child is unfinished, naming the children, `--force` to override; the status key is the one `truth/tasks/refuse-a-transition-into-done-while-blocked.typ` introduces, so one legend knob covers both.
- 🟢 `mt check` reports a parent already in such a status with an unfinished child.


== Changes

- `crates/mindtape/src/dep_rank.rs` (tests first): compute `children` and `progress` over `parent` edges.
- `crates/mindtape/src/fields.rs`: expose both as filter/sort fields.
- `crates/mindtape/src/canonical.rs`: carry them in `--json`.
- `crates/mindtape-cli/src/flip.rs`: the parent refusal.
- `www/content/links.typ`, `skills/mindtape` source: document the fields.


== Alternatives

- Progress by estimate weight: needs every child estimated in one unit; the count is honest and total.
- A stored `progress` field: a computed value would drift from the children it summarises.


== Out of scope

- Rollup of estimates or due dates: a separate question.


== Dogfood

Sprint parents in `truth/tasks/` show their progress, and closing one early is refused.

