#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the Gate-Scope trailer where CONTRIBUTING.md offers MINDTAPE_GATE_SCOPE",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
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
      docs-match-code: helps[the contributing guide states the whole scoped-commit rule the hooks enforce],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 17),
)

== Problem

+ Committing the first of two commits in a delegated worktree with `MINDTAPE_GATE_SCOPE`.
+ 🔴 #raw("docs/src/contributing.typ"): CONTRIBUTING.md says `MINDTAPE_GATE_SCOPE=<reason> git commit` skips the freshness check, but the commit-msg hook also requires a matching `Gate-Scope:` trailer, so a commit made as written is refused.

```
commit-msg: MINDTAPE_GATE_SCOPE=... skipped the pre-commit freshness
assertion, but the commit message carries no matching trailer. Add a footer
line reading exactly:

    Gate-Scope: ...

(see CONTRIBUTING.md, Quality Gates) and commit again.
```
`rg Gate-Scope` hits docs/contributing/gates.md, not CONTRIBUTING.md.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("docs/src/contributing.typ").

== TO-BE

- 🟢 The Quality Gates paragraph that offers `MINDTAPE_GATE_SCOPE` names the `Gate-Scope:` trailer it needs, so a scoped commit lands on the first try.


== Changes

- `docs/src/contributing.typ`: add the trailer to the `MINDTAPE_GATE_SCOPE` sentence, then `just docs md`.


== Rollback

- Revert the sentence.


== Out of scope

- The hook itself. It already words the trailer it wants.


== Dogfood

Every delegated agent in this repo scopes its early commits this way.
