#import "@local/mindtape:0.4.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Track the installed skill against its source",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling", "stateless"),
  x-goals: (
    product: verdicts(
      (
        cheap-adoption: helps[the installed skill cannot silently fall behind the CLI],
      ),
      ..(
        "readable-tasks",
        "implementable-tasks",
        "enforced-practice",
        "typst-over-markdown",
        "plain-files",
      ),
    ),
    process: verdicts(
      (
        docs-match-code: helps[a stale copy fails a check instead of a later session],
      ),
      ..("sound-codebase",),
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 7),
  links: depends-on("cut-the-skill-to-an-index-of-what-mt-cannot-say.typ"),
)

== AS-IS

+ Run `just install`, then edit `docs/src/skill.typ` and regenerate `skills/mindtape/SKILL.md`.
+ 🔴 `just check` passes while the harness copy still holds the old text, because `install.sh` copies the tree and writes no stamp.

```
$ rg -n 'mt rules' ~/.config/opencode/skills/mindtape/SKILL.md
36:| List the rules | `mt rules` |
$ rg -n 'mt explain' skills/mindtape/SKILL.md
40:Run `mt explain` first...
```


== TO-BE

- 🟢 `just install` writes a stamp beside each installed skill with the hash of the source tree it copied.
- 🟢 `just check` warns when an installed copy's hash differs from `skills/<name>`, and stays quiet when no copy exists.
- 🟢 Running `just install` after an edit clears the warning.
- 🟢 The check never writes to the harness directories.

```
$ just check
warning: ~/.config/opencode/skills/mindtape differs from skills/mindtape; run `just install`
```


== Changes

- `.just/scripts/install.sh`: write the source hash into each installed skill.
- `.just/scripts/check-skill-drift.sh`: compare the stamp with the source and warn on a mismatch.
- `Justfile`: add `skill-drift` to the check arm.
- `docs/src/contributing.typ`: name the stamp and the warning in the worktrees section.


== Out of scope

- Symlinking instead of copying; the copy stays matched to the installed `mt`.
- The skill content, which `cut-the-skill-to-an-index-of-what-mt-cannot-say` owns.


== Dogfood

This repo's `just install` writes the stamp and `just check` reports a stale copy before a session loads it.

