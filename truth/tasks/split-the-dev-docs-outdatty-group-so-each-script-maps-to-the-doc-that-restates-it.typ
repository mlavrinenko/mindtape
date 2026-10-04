#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Split the dev-docs outdatty group so each script maps to the doc that restates it",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 4),
  tags: ("found", "tooling"),
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
      docs-match-code: helps[a flagged dependent is one that restates the changed script],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 4),
)

== Problem

+ Reviewing the gate fix commits e551aca4, d228fe5b and c0bd3b46
+ 🔴 #raw("outdatty.yaml"): dev-docs ties 40 sources to 11 dependents, so a script fix is re-recorded unread, and gate-notes.typ is in no group

```
$ git show -s --format=%B <sha> | grep -m1 "dev-docs is re-recorded"   # per commit
e551aca4: user's worktrees.toml. dev-docs is re-recorded: no dependent describes how the gate
d228fe5b: dev-docs is re-recorded. The gate's table row and the gates page stay accurate,
c0bd3b46: dev-docs is re-recorded. No dependent describes how the gate reads a span.
$ grep -c gate-notes outdatty.yaml
0
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("outdatty.yaml").

== TO-BE

- 🟢 Each group pairs the scripts with the docs that restate them, such as the CRAP scripts with `coverage-crap.typ` and `crap-baseline.typ`, and `fork.sh` and `integrate.sh` with `contributing.typ`.
- 🟢 Every gate script is a source of a group whose dependents include `www/utils/gate-notes.typ`.
- 🟢 A script no doc restates sits in a source-only group, so editing it flags nothing.
- 🟢 `just script-scope-check` accepts a script in any group, not only dev-docs.


== Changes

- `outdatty.yaml`: replace dev-docs with per-topic groups and add `www/utils/gate-notes.typ`.
- `outdatty.lock`: record the new groups.
- `.just/scripts/script-scope-check.sh`: require each script to be a source of some group or exempt.
- `docs/src/contributing/gates.typ`: say that a script joins the group of the doc that restates it.


== Rollback

- Revert the commit and `outdatty.lock`; dev-docs returns whole.


== Out of scope

- The other groups, which pair sources and dependents already.


== Dogfood

A gate-script fix in this repo flags only the doc that restates it, so a re-record means someone read that doc.

