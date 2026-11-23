#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Install the skill from the checkout under test, not from the machine",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 6,
  ),
  difficulty: 3,
  links: (
    (
      task: "give-the-templates-their-own-command.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "tooling", "gating", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 24),
)

== AS-IS

+ Running `just check` in a fork while a sibling worktree edited `skills/mindtape/SKILL.md`
+ 🔴 #raw(".just/scripts/check-skill-drift.sh"): `skill-drift` compares a worktree against a single machine-wide stamp, so two worktrees that touch the skill fail each other

```
$ just skill-drift
warning: ~/.claude/skills/mindtape differs from skills/mindtape; run `just install`
warning: ~/.config/opencode/skills/mindtape differs from skills/mindtape; run `just install`
```

Four of the thirteen workers in the milestone run hit it, and each fix overwrote the copy the other worktree had just stamped.

== TO-BE

- 🟢 The gate compares the installed copy with the worktree that installed it, so a sibling checkout cannot fail it.
- 🟢 A number of worktrees edited in parallel still leaves one answer per checkout.
- 🟢 The message states which checkout owns the copy.

```
$ just skill-drift
warning: ~/.claude/skills/mindtape was installed from ../mindtape-other, not this worktree
```

== Changes

- `.just/scripts/check-skill-drift.sh`: read the owning checkout from the stamp beside the hash, and compare only when it is this one, or compare against every copy's own stamp.
- `.just/install.just`: write the source path into the stamp beside the hash.
- `docs/src/contributing/gates.typ`: what the stamp holds, and why a fork does not fail on a sibling's install.
- `CHANGELOG.md`: an entry under `[Unreleased]` `Fixed`.

== Out of scope

- Where the copies live. The harness dirs stay as they are.
- `just install`'s scope. It still writes the binary, schema and completions.

== Dogfood

A `just fork` milestone runs many worktrees at once, and the skill is one of the files a feature branch edits.

== Goals

+ #Helps(adopter, to: adopt-in-one-line)[a fork works beside its siblings].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).
