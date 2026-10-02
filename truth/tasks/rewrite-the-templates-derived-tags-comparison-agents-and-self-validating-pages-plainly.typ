#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the templates, derived-tags, comparison, agents and self-validating pages plainly",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (src-files: 3),
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
      docs-match-code: helps[each example on these pages runs against the project it names],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
  links: depends-on(
    "write-plain-prose-rules-for-the-docs-and-check-for-history-and-shouting.typ",
    "refuse-hand-typed-code-blocks-in-doc-sources.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Run the pages' examples and read their claims.
+ 🔴 `templates.typ:142` sets `estimate.files: 2`; the template on that page has no such slot. `agents.typ:64` sets `priority`, which no template the docs use declares.
+ 🔴 `derived-tags.typ:65` says a closed task keeps the tags its folder derived when filed. Derived tags are recomputed on every load; `validate.rs:375` only skips the allow-list check for closed tasks.
+ 🔴 `comparison.typ:203` links the Recurring row to `/dsl/`, `:163` links Dependencies to `/filters/`, and `:308` says no other tool computes priority from inputs while its own Taskwarrior cell says urgency does. `:311` argues against "a YAML record".
+ `agents.typ:73` uses `--template finding`, which exists only in this repo; `:110` names a local path.


== TO-BE

- 🟢 Every example runs against the project it names; blocks are fixture slices or captures.
- 🟢 No page argues against an imaginary YAML file; /comparison/ compares real tools only.
- 🟢 No page leans on this repo's own templates, paths or backlog.
- 🟢 The prose follows the plain-prose rules.


== Source of truth

- `crates/mindtape-cli/src/add*.rs`: slots, splices, `--into`.
- `crates/mindtape/src/validate.rs`: derived tags.
- The tools' own docs, for /comparison/ cells.


== Changes

- `www/content/{templates,derived-tags,comparison,agents,self-validating-tasks}.typ`: rewrite whole. On /templates/, also list `assignee` among spliced fields (`add_slots.rs:86`), say `id` is injected only when `[tasks.id] format` is set, and drop the `--into .` example (it is the default).
- `examples/self-validating`, `examples/minimal`: slices the pages need.
- `.just/scripts/check-doc-blocks.sh`: drop these pages from the list.


== Out of scope

- Adding tools to /comparison/: fix what is there.


== Dogfood

Agents in this repo follow /agents/; its examples now run as shown.

