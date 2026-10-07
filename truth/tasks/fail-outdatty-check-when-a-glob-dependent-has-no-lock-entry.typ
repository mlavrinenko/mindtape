#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fail outdatty-check when a glob dependent has no lock entry",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 1),
  tags: ("found", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a new page under a glob dependent cannot land without a recorded review hash],
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
  status: done(2026, 10, 7),
)

== Problem

+ Reading why 21 CLI pages went unrecorded until commit 22b1024e.
+ 🔴 #raw("Justfile"): \`outdatty check\` passes while a file matched by a glob dependent has no lock entry, so a new page under \`www/content/cli/\*.typ\` stays unrecorded and unwatched with every gate green.

`outdatty.yaml:150`: `- www/content/cli/*.typ`
`Justfile:182-183`: `outdatty-check:` runs `outdatty check --format paths`
Commit 22b1024e (`docs(outdatty): record a review hash for each CLI command page`) added 21 lock entries that no gate had asked for.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("Justfile").

== TO-BE

- 🟢 `just outdatty-check` fails and names each glob-expanded dependent that `outdatty.lock` holds no hash for.
```
unrecorded: www/content/cli/new-page.typ
  record it: just outdatty-update cli-docs www/content/cli/new-page.typ
```


== Changes

- `Justfile`: `outdatty-check` runs `.just/scripts/outdatty-check.sh`, which fails on a glob-expanded dependent the lock lacks. Upstream outdatty could own the check later.

== Rollback

- Revert the `Justfile` recipe.

== Out of scope

- Judging whether a recorded hash was reviewed. Only presence is checked.

== Dogfood

Every new page under a glob dependent here is caught at the gate.
