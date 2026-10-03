#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Say in the skill that a link target is a path from the linking file",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
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
      docs-match-code: helps[an agent writes link targets that resolve],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ Rewriting /concepts/ and /links/, which state that a target is a path.
+ 🔴 #raw("docs/src/skill.typ"): The skill tells agents that \`links:\` edges name tasks by stem, but \`mt\` resolves a target as a path relative to the linking file.

- `skills/mindtape/SKILL.md:14`: "The filename stem is the task’s identity, and `links:` edges name tasks by stem."
- `skills/mindtape/references/hand-editing.md:25`: `links: depends-on("other-task.typ"),  // typed edges by file stem`
- `crates/mindtape/src/model.rs`: `resolve_link_path` joins the target onto the linking task's directory.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("docs/src/skill.typ").

== TO-BE

- 🟢 The skill says a target is the path from the linking file, and that `mt link` writes it from a bare stem.


== Changes

- `docs/src/skill.typ`: correct the identity paragraph.
- `skills/mindtape/references/hand-editing.md`: correct the sketch comment, or slice it from `examples/dsl`.


== Rollback

- Revert the commit.


== Out of scope

- The skill's other references.


== Dogfood

- Agents in this repo read the skill before they hand-edit `links:`.

