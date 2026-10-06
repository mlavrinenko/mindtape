#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Point the agents page at the standard skill installers",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("www", "docs"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader installs the skill with a tool they may already have, and mt ships no installer of its own],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[each install command on the page was run against this repo's layout before it was written down],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 6),
)

== Problem

+ Open `/agents/` and read "Give your agent the skill".
+ 🔴 It says "Copy that directory into the directory your agent loads skills from". It names neither the skill format, the directory nor a tool that installs a skill.


== TO-BE

- 🟢 The section names the Agent Skills format and links agentskills.io for what a skill is.
- 🟢 It shows the agent-agnostic installers that read a skill from a GitHub repo, `gh skill install` and `npx skills add`, each run first into a temporary home to confirm it installs `skills/mindtape/` with its `references/`.
- 🟢 It shows the manual copy and links the skills-directory docs of the major agents instead of restating their paths.


== Source of truth

- The Agent Skills spec at agentskills.io.
- The `gh skill` docs and skills.sh.
- The layout of `skills/mindtape/`.


== Changes

- `www/content/agents.typ`: rewrite the section.
- `docs/src/readme.typ`: the same pointer, if the README mentions installing the skill.


== Out of scope

- An `mt` command that installs the skill. `mt` tracks tasks, and installing skills is the installers' job.
- A plugin-marketplace manifest in this repo.


== Dogfood

Not applicable: this repo's agents load the skill from its own tree.

