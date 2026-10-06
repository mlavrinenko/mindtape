#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Split the Reference sidebar into groups by what the reader looks up",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader finds where to set a thing by scanning a short group, not thirteen links],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 6),
)

== Problem

+ Open any docs page and read the sidebar's Reference group.
+ 🔴 It lists 13 pages in one flat run: CLI reference, Filter language, Priority, Configuration, Statuses, Rules, Rule packs, Commit rules, Named sets, Hooks, Estimates, JSON schema, Import & export.
+ 🔴 Some of them explain one part of the model, such as Statuses, Estimates and Named sets, rather than serve a lookup. A reader looking for where to set something reads all 13.


== TO-BE

- 🟢 Reference holds the lookup pages: the CLI, the filter language, the configuration and the JSON schema.
- 🟢 The rest move into groups named for the reader's job, such as the checks a project sets (rules, rule packs, commit rules, hooks) and the fields a task carries (statuses, priority, estimates, named sets). No group holds more than 6 pages.
- 🟢 No route changes, so no link breaks.


== Source of truth

- `NAV` in `www/utils/site.typ`, and its comment on why groups follow the reader's job.
- Each page's content, which decides whether it is lookup or explanation.


== Changes

- `www/utils/site.typ`: the new groups and their comment.
- Anything that renders the groups, such as `www-groups` in `docs/src/lib.typ`: regenerate its outputs.


== Out of scope

- Page content and routes.
- The CLI pages, which another task splits.


== Dogfood

Not applicable: the sidebar serves readers of the site, not this repo's workflow.

