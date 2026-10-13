#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Merge the jscpd baseline by fingerprint union",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 3),
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
      sound-codebase: helps[parallel worktrees land without hand-resolving the clone baseline],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
)

== Problem

+ Landing seven parallel sweep lanes onto main
+ 🔴 #raw(".jscpd-baseline.json"): every lane that re-minted the clone baseline conflicted with the lanes landed before it, and each rebase needed the conflict taken by hand and `just dry-baseline` re-run

```
error: could not apply 3852b06a... refactor(tests): assert a commit-msg refusal through support::commit_msg_refusal
  Auto-merging .jscpd-baseline.json
  CONFLICT (content): Merge conflict in .jscpd-baseline.json
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".gitattributes").

== TO-BE

- 🟢 A rebase joins both sides' fingerprints, as `outdatty.lock` joins its hashes, and `just check-dry` on the merged tree decides the rest.

== Changes

- `.gitattributes`: route `.jscpd-baseline.json` to a merge driver.
- `.just/scripts/`: a driver that writes the union of both sides' `fingerprints`.
- `flake.nix`: register it beside `outdatty-lock`.

== Rollback

- Delete the `.gitattributes` line; git falls back to its text merge.

== Out of scope

- `crap-baseline.json`, which conflicted once and whose union would hide a dropped function.

== Dogfood

This repo lands parallel `just fork` lanes without touching the clone baseline.
