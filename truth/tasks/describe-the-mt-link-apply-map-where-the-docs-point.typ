#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Describe the mt link --apply map where the docs point",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "docs"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the page a reader is sent to for the adjacency map describes it],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 17),
)

== Problem

+ Reviewing the cli-link-docs dependents while splitting cli-docs
+ 🔴 #raw("www/utils/cli-notes.typ"): The `link` note says the `--apply` map is described on Task DSL, but dsl.typ never mentions it, and links.typ gives it one sentence.


```
$ rg -n "described on" www/utils/cli-notes.typ
114:      #link(u("/concepts/#identity"))[stem] to kind to targets, described on
$ rg -c "apply|adjacency" www/content/dsl.typ
$ echo $?
1
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/content/links.typ").

== TO-BE

- 🟢 One page shows the `--apply` map as a checked YAML block: stem to kind to targets, with a note on an edge that needs one.
- 🟢 The `link` note and the `mt link` page link to that block.


== Changes

- `www/content/links.typ`: a checked example of the map under Writing edges with `mt link`.
- `www/utils/cli-notes.typ`: point the `link` note at that section.


== Rollback

- Revert both files.


== Out of scope

- Generating the map's shape from `cmd/link/input.rs`. A checked example is enough here.


== Dogfood

Not applicable: this repo links its tasks one edge at a time.

