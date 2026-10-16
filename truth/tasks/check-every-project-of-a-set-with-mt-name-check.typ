#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Check every project of a set with mt @name check",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[one command runs every project's rules, so a repo that drifted shows up without a visit],
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
  status: proposed(2026, 10, 16),
  links: parent("show-every-tracked-project-from-one-mt-name.typ")
    + depends-on(
      "resolve-mt-name-through-at-toml-and-rename-mt-worktrees-to-mt-at.typ",
      "load-every-project-of-a-set-in-parallel.typ",
    ),
)

== Problem

+ Run `mt check` in each of the 30 projects under `~/projects/home` with today's `mt`.
+ 🔴 Three of them fail before reading a task, and nothing reported it, because `mt check` covers one repo per run. kimmable prints:
```
error: .mindtape/config.toml:107:17: can't parse config: unknown field `max`, expected one of `units`, `required`, `example`
```


== TO-BE

- 🟢 `mt @home check` checks every project in parallel, then prints each project's problems together in set order, each line naming its project.
- 🟢 It ends with one line, such as `ok: 27 projects, 1904 tasks` or `3 of 30 projects failed`, and exits 1 when any project fails.
- 🟢 `--json` prints one object per project with its problems.
- 🟢 `--commit-msg` and `--commits` refuse a set, since each reads one repo's git.


== Changes

- `crates/mindtape-cli/tests/it/check_set.rs`: the red tests first: a clean set, a broken member, `--json`, the refused flags.
- `crates/mindtape-cli/src/cmd/check/set.rs`: check each member from `open_set`, hold each report, and print them in set order.
- `crates/mindtape-cli/src/cmd/check/mod.rs`: send a set to `set.rs`, and refuse the commit flags on one.


== Alternatives

- Print problems as they come: 30 projects in parallel would interleave their lines.
- Stop at the first failed project: that hides the others, as a shell loop with `set -e` does.


== Out of scope

- Fixing the three broken configs: each belongs to its own repo.
- Commit checks across repos: `--commits` names a range in one repo.


== Dogfood

After a release changes the config schema, `mt @home check` finds every repo under `~/projects/home` that needs a fix, this one included.

