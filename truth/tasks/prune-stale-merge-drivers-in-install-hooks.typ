#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Prune stale merge drivers in install-hooks",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
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
      sound-codebase: helps[a deleted driver leaves no dangling merge config],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ Reading the shared .git/config while landing a rename of a merge driver
+ 🔴 #raw("Justfile"): `just install-hooks` writes `merge.<name>.*` and never removes one, so a renamed or deleted driver stays in the shared `.git/config`

```
$ git config --get-regexp '^merge\..*driver'
merge.outdatty-lock.driver bash .just/scripts/outdatty-lock-merge.sh %O %A %B %P %S %X %Y
merge.outdatty.driver outdatty merge-lock %O %A %B
$ ls .just/scripts/outdatty-lock-merge.sh
ls: cannot access '.just/scripts/outdatty-lock-merge.sh': No such file or directory
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("Justfile").

== TO-BE

- 🟢 After `just install-hooks`, the shared `.git/config` holds a `merge.<name>` section for each driver the recipe registers and a checked-out `.gitattributes` still names with `merge=<name>`.
- 🟢 A driver the recipe once registered that no checked-out worktree's `.gitattributes` names is removed, and a driver added by hand stays.

== Changes

- `Justfile`: `install-hooks` marks each driver it registers with `merge.<name>.mindtape = true`, collects the `merge=<name>` names from every worktree's `.gitattributes`, and removes each marked driver, or unmarked one that runs a `.just/scripts/` script, that none names.
- `crates/mindtape-cli/tests/it/install_hooks_e2e.rs`: a throwaway repo with a stale marked driver, a stale legacy driver, a hand-added one and a second worktree, run through the real recipe.

== Rollback

- Revert the `Justfile` change and delete the `merge.<name>.mindtape` keys from `.git/config`.

== Out of scope

- Per-worktree config: two branches that give one driver name different commands still overwrite each other, as before.
- Drivers neither marked nor running a `.just/scripts/` script: they might be the user's.

== Dogfood

Every dev shell entry runs the recipe, so the stale `merge.outdatty-lock` goes the next time any worktree lands this.
