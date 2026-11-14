#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

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

== AS-IS

+ Reading why 21 CLI pages went unrecorded until commit 22b1024e.
+ 🔴 #raw("Justfile"): `outdatty check` passes while a file matched by a glob dependent has no lock entry. A new page under `www/content/cli/*.typ` then stays unrecorded and unwatched with every gate green.

`outdatty.yaml:150` lists `- www/content/cli/*.typ`, and `Justfile:182-183` runs `outdatty check --format paths` under `outdatty-check:`. Commit 22b1024e added 21 lock entries that no gate had asked for. Its message is `docs(outdatty): record a review hash for each CLI command page`.


This fell outside the task that surfaced it. The fix lives here rather than
in a closing report no one rereads. Its home is
#raw("Justfile").

== TO-BE

- 🟢 `just outdatty-check` fails and states each glob-expanded dependent that has no hash in `outdatty.lock`.
```
unrecorded: www/content/cli/new-page.typ
  record it: just outdatty-update cli-docs www/content/cli/new-page.typ
```


== Changes

- `Justfile`: `outdatty-check` runs `.just/scripts/outdatty-check.sh`, which fails on a glob-expanded dependent the lock lacks. Upstream outdatty could own the check later.

== Rollback

- Revert the `Justfile` recipe.

== Out of scope

- Judging whether a recorded hash passed review. The check only looks at presence.

== Dogfood

The gate catches every new page under a glob dependent here.
